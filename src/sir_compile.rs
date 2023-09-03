use std::mem;

use bit_set::BitSet;

use crate::cctx::CCtx;
use crate::sir::{BasicBlock, Function, Inst, InstKind, ProgramUnit};

pub fn compile(cctx: &CCtx, program_unit: &ProgramUnit) -> ProgramUnit {
    if cfg!(debug_assert) {
        program_unit.validate_insts().unwrap();
    }
    let mut program_unit = program_unit.clone();
    for function in &mut program_unit.functions {
        *function = compile_function(cctx, function);
    }
    program_unit
}

fn compile_function(cctx: &CCtx, function: &Function) -> Function {
    let mut function = function.clone();
    liveness(cctx, &mut function);
    insert_copy(cctx, &mut function);
    function
}

fn liveness(cctx: &CCtx, function: &mut Function) {
    let mut updated = true;
    while updated {
        updated = false;
        for bb_id in 0..function.body.len() {
            liveness_bb(cctx, function, bb_id, &mut updated);
        }
    }
}
fn liveness_bb(_cctx: &CCtx, function: &mut Function, bb_id: usize, updated: &mut bool) {
    let mut alive = block_live_out_to_be(function, &function.body[bb_id]);
    let bb = &mut function.body[bb_id];
    for inst in bb.insts.iter_mut().rev() {
        if let Some(live_out) = &inst.live_out {
            if live_out == &alive {
                return;
            }
        }
        inst.live_out = Some(alive.clone());
        update_alive(inst, &mut alive);
    }
    if let Some(live_in) = &mut bb.live_in {
        if live_in == &alive {
            return;
        }
    }
    bb.live_in = Some(alive.clone());
    *updated = true;
}

fn inst_live_in(inst: &Inst) -> BitSet<usize> {
    let mut alive = inst.live_out.clone().unwrap();
    update_alive(inst, &mut alive);
    alive
}

fn update_alive(inst: &Inst, alive: &mut BitSet<usize>) {
    match &inst.kind {
        InstKind::Jump { target: _ } => {}
        InstKind::Branch {
            cond,
            branch_then: _,
            branch_else: _,
        } => {
            alive.insert(*cond);
        }
        InstKind::Return { rhs } => {
            alive.insert(*rhs);
        }
        InstKind::Copy { lhs, rhs } => {
            alive.remove(*lhs);
            alive.insert(*rhs);
        }
        InstKind::Drop { rhs } => {
            alive.insert(*rhs);
        }
        InstKind::Literal { lhs, .. } => {
            alive.remove(*lhs);
        }
        InstKind::Closure { lhs, function_id } => {
            alive.remove(*lhs);
            alive.insert(*function_id);
        }
        InstKind::Builtin { lhs, builtin: _ } => {
            alive.remove(*lhs);
        }
        InstKind::PushArg { value_ref } => {
            alive.insert(*value_ref);
        }
        InstKind::Call { lhs, callee } => {
            alive.remove(*lhs);
            alive.insert(*callee);
        }
    }
}

/// Computes live-out from the successor blocks.
/// (may yield different result from `block_live_out` during the liveness analysis)
fn block_live_out_to_be(function: &Function, bb: &BasicBlock) -> BitSet<usize> {
    let last = bb.insts.last().unwrap();
    assert!(last.kind.is_tail());
    match &last.kind {
        InstKind::Jump { target } => function.body[*target].live_in.clone().unwrap_or_default(),
        InstKind::Branch {
            cond: _,
            branch_then,
            branch_else,
        } => {
            let mut live_out = function.body[*branch_then]
                .live_in
                .clone()
                .unwrap_or_default();
            if let Some(live_in_else) = &function.body[*branch_else].live_in {
                live_out.union_with(&live_in_else);
            }
            live_out
        }
        InstKind::Return { rhs: _ } => BitSet::default(),
        _ => unreachable!(),
    }
}

fn block_live_out(bb: &BasicBlock) -> &BitSet<usize> {
    bb.insts.last().unwrap().live_out.as_ref().unwrap()
}

// Also inserts Drop when necessary
fn insert_copy(cctx: &CCtx, function: &mut Function) {
    // Compute carried over variables
    let mut carried_over = vec![BitSet::<usize>::default(); function.body.len()];
    for arg in 0..function.num_args {
        carried_over[0].insert(arg);
    }
    // Its correctness depends on the absense of multi-in multi-out edges.
    // That means, all the block connections falls into the following cases:
    // 1. An edge that shares its successor with other edges, but not its predecessor. (i.e. Jump)
    // 2. An edge that shares its predecessor with other edges, but not its successor. (i.e. Branch)
    // 3. An edge that does not share its predecessor nor successor with other edges (but its probably useless)
    for bb in function.body.iter() {
        let last = bb.insts.last().unwrap();
        assert!(last.kind.is_tail());
        match &bb.insts.last().unwrap().kind {
            InstKind::Jump { target } => {
                // *target may have multiple writes, but the value is same across predecessors.
                // Therefore it is safe to say that the value is the intersection of the predecessors rather than union.
                carried_over[*target].union_with(block_live_out(bb));
            }
            InstKind::Branch {
                cond: _,
                branch_then,
                branch_else,
            } => {
                // *branch_then is written only once.
                // Therefore it is safe to say that the value is the intersection of the predecessors rather than union.
                // The same applies to *branch_else.
                carried_over[*branch_then].union_with(block_live_out(bb));
                carried_over[*branch_else].union_with(block_live_out(bb));
            }
            InstKind::Return { rhs: _ } => {}
            _ => unreachable!(),
        }
    }

    for (bb, bb_co) in function.body.iter_mut().zip(carried_over) {
        insert_copy_bb(cctx, &mut function.num_vars, bb, bb_co);
    }
}

fn insert_copy_bb(
    _cctx: &CCtx,
    num_vars: &mut usize,
    bb: &mut BasicBlock,
    mut carried_over: BitSet<usize>,
) {
    let old_insts = mem::replace(&mut bb.insts, Vec::new());

    // Drop unused variables carried over from the last block (caused by branch instructions)
    let mut unused_carried_over = carried_over.clone();
    unused_carried_over.difference_with(bb.live_in.as_ref().unwrap());
    for var in unused_carried_over.iter() {
        carried_over.remove(var);
        bb.insts
            .push(Inst::drop(var).with_live_out(carried_over.clone()));
    }

    // Process block body
    for mut inst in old_insts {
        // Insert copy before the instruction, if necessary
        if let Some(moved_rhs) = moved_rhs_of(&inst) {
            if inst.live_out.as_ref().unwrap().contains(moved_rhs) {
                let new_rhs = fresh_var(num_vars);
                let mut alive = inst_live_in(&inst);
                alive.insert(new_rhs);
                bb.insts
                    .push(Inst::copy(new_rhs, moved_rhs).with_live_out(alive));
                replace_moved_rhs(&mut inst, new_rhs);
            }
        }

        // Insert drop after the instruction, if necessary
        let drop_inst = if let Some(lhs) = lhs_of(&inst) {
            if !inst.live_out.as_ref().unwrap().contains(lhs) {
                let drop_live_out = inst.live_out.clone().unwrap();
                inst.live_out.as_mut().unwrap().insert(lhs);
                Some(Inst::drop(lhs).with_live_out(drop_live_out))
            } else {
                None
            }
        } else {
            None
        };

        bb.insts.push(inst);
        if let Some(drop_inst) = drop_inst {
            bb.insts.push(drop_inst);
        }
    }
}

fn fresh_var(num_vars: &mut usize) -> usize {
    let var = *num_vars;
    *num_vars += 1;
    var
}

fn moved_rhs_of(inst: &Inst) -> Option<usize> {
    match &inst.kind {
        InstKind::Jump { .. } => None,
        InstKind::Branch { cond, .. } => Some(*cond),
        InstKind::Return { rhs } => Some(*rhs),
        InstKind::Copy { .. } => None,
        InstKind::Drop { rhs } => Some(*rhs),
        InstKind::Literal { .. } => None,
        InstKind::Closure {
            lhs: _,
            function_id: _,
        } => None,
        InstKind::Builtin { lhs: _, builtin: _ } => None,
        InstKind::PushArg { value_ref } => Some(*value_ref),
        InstKind::Call { lhs: _, callee } => Some(*callee),
    }
}

fn replace_moved_rhs(inst: &mut Inst, to: usize) {
    match &mut inst.kind {
        InstKind::Jump { .. } => {
            unreachable!();
        }
        InstKind::Branch { cond, .. } => {
            *cond = to;
        }
        InstKind::Return { rhs } => {
            *rhs = to;
        }
        InstKind::Copy { .. } => {
            unreachable!();
        }
        InstKind::Drop { rhs } => {
            *rhs = to;
        }
        InstKind::Literal { .. } => {
            unreachable!();
        }
        InstKind::Closure { .. } => {
            unreachable!();
        }
        InstKind::Builtin { .. } => {
            unreachable!();
        }
        InstKind::PushArg { value_ref } => {
            *value_ref = to;
        }
        InstKind::Call { callee, .. } => {
            *callee = to;
        }
    }
}

fn lhs_of(inst: &Inst) -> Option<usize> {
    match &inst.kind {
        InstKind::Jump { .. } => None,
        InstKind::Branch { .. } => None,
        InstKind::Return { .. } => None,
        InstKind::Copy { lhs, .. } => Some(*lhs),
        InstKind::Drop { .. } => None,
        InstKind::Literal { lhs, .. } => Some(*lhs),
        InstKind::Closure { lhs, .. } => Some(*lhs),
        InstKind::Builtin { lhs, .. } => Some(*lhs),
        InstKind::PushArg { .. } => None,
        InstKind::Call { lhs, .. } => Some(*lhs),
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use crate::sir::{BasicBlock, BuiltinKind, Inst, ProgramUnit};

    use super::*;

    #[test]
    fn test_compile() {
        let cctx = CCtx::new();
        let program_unit = ProgramUnit::simple(Function::simple(0, |[x, puts1, tmp1, tmp2]| {
            BasicBlock::new(vec![
                Inst::literal(x, "Hello, world!"),
                Inst::builtin(puts1, BuiltinKind::Puts),
                Inst::push_arg(x),
                Inst::call(tmp2, puts1),
                Inst::builtin(puts1, BuiltinKind::Puts),
                Inst::push_arg(x),
                Inst::call(tmp2, puts1),
                Inst::literal(x, "Hello, world!"),
                Inst::builtin(puts1, BuiltinKind::Puts),
                Inst::push_arg(x),
                Inst::call(tmp2, puts1),
                Inst::literal(tmp1, ()),
                Inst::return_(tmp1),
            ])
        }));
        let program_unit = compile(&cctx, &program_unit);
        assert_eq!(
            program_unit,
            ProgramUnit::simple(Function::simple(0, |[x, puts1, tmp1, tmp2, tmp3]| {
                BasicBlock::new(vec![
                    Inst::literal(x, "Hello, world!").with_live_out([x].into_iter().collect()),
                    Inst::builtin(puts1, BuiltinKind::Puts)
                        .with_live_out([x, puts1].into_iter().collect()),
                    Inst::copy(tmp3, x).with_live_out([x, puts1, tmp3].into_iter().collect()),
                    Inst::push_arg(tmp3).with_live_out([x, puts1].into_iter().collect()),
                    Inst::call(tmp2, puts1).with_live_out([x, tmp2].into_iter().collect()),
                    Inst::drop(tmp2).with_live_out([x].into_iter().collect()),
                    Inst::builtin(puts1, BuiltinKind::Puts)
                        .with_live_out([x, puts1].into_iter().collect()),
                    Inst::push_arg(x).with_live_out([puts1].into_iter().collect()),
                    Inst::call(tmp2, puts1).with_live_out([tmp2].into_iter().collect()),
                    Inst::drop(tmp2).with_live_out([].into_iter().collect()),
                    Inst::literal(x, "Hello, world!").with_live_out([x].into_iter().collect()),
                    Inst::builtin(puts1, BuiltinKind::Puts)
                        .with_live_out([x, puts1].into_iter().collect()),
                    Inst::push_arg(x).with_live_out([puts1].into_iter().collect()),
                    Inst::call(tmp2, puts1).with_live_out([tmp2].into_iter().collect()),
                    Inst::drop(tmp2).with_live_out([].into_iter().collect()),
                    Inst::literal(tmp1, ()).with_live_out([tmp1].into_iter().collect()),
                    Inst::return_(tmp1).with_live_out([].into_iter().collect()),
                ])
                .with_live_in([].into_iter().collect())
            }))
        );
    }

    #[test]
    fn test_compile_drop() {
        let cctx = CCtx::new();
        let program_unit = ProgramUnit::simple(Function::simple(0, |[x, puts1, tmp1, tmp2]| {
            BasicBlock::new(vec![
                Inst::literal(x, "dummy"),
                Inst::literal(x, "Hello, world!"),
                Inst::builtin(puts1, BuiltinKind::Puts),
                Inst::push_arg(x),
                Inst::call(tmp2, puts1),
                Inst::literal(tmp1, ()),
                Inst::return_(tmp1),
            ])
        }));
        let program_unit = compile(&cctx, &program_unit);
        assert_eq!(
            program_unit,
            ProgramUnit::simple(Function::simple(0, |[x, puts1, tmp1, tmp2]| {
                BasicBlock::new(vec![
                    Inst::literal(x, "dummy").with_live_out([x].into_iter().collect()),
                    Inst::drop(x).with_live_out([].into_iter().collect()),
                    Inst::literal(x, "Hello, world!").with_live_out([x].into_iter().collect()),
                    Inst::builtin(puts1, BuiltinKind::Puts)
                        .with_live_out([x, puts1].into_iter().collect()),
                    Inst::push_arg(x).with_live_out([puts1].into_iter().collect()),
                    Inst::call(tmp2, puts1).with_live_out([tmp2].into_iter().collect()),
                    Inst::drop(tmp2).with_live_out([].into_iter().collect()),
                    Inst::literal(tmp1, ()).with_live_out([tmp1].into_iter().collect()),
                    Inst::return_(tmp1).with_live_out([].into_iter().collect()),
                ])
                .with_live_in([].into_iter().collect())
            }))
        );
    }

    #[test]
    fn test_compile_drop_arg() {
        let cctx = CCtx::new();
        let program_unit = ProgramUnit::simple(Function::simple(1, |[_arg1, tmp1]| {
            BasicBlock::new(vec![Inst::literal(tmp1, ()), Inst::return_(tmp1)])
        }));
        let program_unit = compile(&cctx, &program_unit);
        assert_eq!(
            program_unit,
            ProgramUnit::simple(Function::simple(1, |[arg, tmp1]| {
                BasicBlock::new(vec![
                    Inst::drop(arg).with_live_out([].into_iter().collect()),
                    Inst::literal(tmp1, ()).with_live_out([tmp1].into_iter().collect()),
                    Inst::return_(tmp1).with_live_out([].into_iter().collect()),
                ])
                .with_live_in([].into_iter().collect())
            }))
        );
    }
}
