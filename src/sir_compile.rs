use std::collections::HashMap;
use std::mem;

use bit_set::BitSet;

use crate::cctx::{CCtx, Id};
use crate::sir::{BasicBlock, Function, Inst, InstKind, ProgramUnit};

pub fn compile(cctx: &CCtx, program_unit: &ProgramUnit) -> ProgramUnit {
    let mut program_unit = program_unit.clone();
    for function in &mut program_unit.functions {
        *function = compile_function(cctx, function);
    }
    program_unit
}

fn compile_function(cctx: &CCtx, function: &Function) -> Function {
    let mut function = function.clone();
    assign_id(cctx, &mut function);
    let live_in = liveness(cctx, &function);
    insert_copy(cctx, &mut function, &live_in);
    unassign_id(&mut function);
    function
}

fn assign_id(cctx: &CCtx, function: &mut Function) {
    for bb in &mut function.body {
        assign_id_bb(cctx, bb);
    }
}
fn assign_id_bb(cctx: &CCtx, bb: &mut BasicBlock) {
    for inst in &mut bb.insts {
        inst.id = cctx.id_gen.fresh();
    }
}

fn unassign_id(function: &mut Function) {
    for bb in &mut function.body {
        unassign_id_bb(bb);
    }
}
fn unassign_id_bb(bb: &mut BasicBlock) {
    for inst in &mut bb.insts {
        inst.id = Id::default();
    }
}

fn liveness(cctx: &CCtx, function: &Function) -> HashMap<Id, BitSet<usize>> {
    let mut live_in = HashMap::new();
    let mut updated = true;
    while updated {
        updated = false;
        for bb in function.body.iter().rev() {
            liveness_bb(cctx, function, bb, &mut live_in, &mut updated);
        }
    }
    live_in
}
fn liveness_bb(
    _cctx: &CCtx,
    function: &Function,
    bb: &BasicBlock,
    live_in: &mut HashMap<Id, BitSet<usize>>,
    updated: &mut bool,
) {
    let mut i = bb.insts.len();
    let mut alive = BitSet::<usize>::default();
    while i > 0 {
        i -= 1;
        match &bb.insts[i].kind {
            InstKind::Jump { target } => {
                alive = get_block_liveness(function, live_in, target);
            }
            InstKind::Branch {
                cond,
                branch_then,
                branch_else,
            } => {
                alive = get_block_liveness(function, live_in, branch_then);
                alive.union_with(&get_block_liveness(function, live_in, branch_else));
                alive.insert(*cond);
            }
            InstKind::Return { rhs } => {
                alive = BitSet::default();
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
            InstKind::Call_ { lhs, callee } => {
                alive.remove(*lhs);
                alive.insert(*callee);
            }
        }
        *updated = *updated
            || if let Some(old_alive) = live_in.get(&bb.insts[i].id) {
                *old_alive != alive
            } else {
                true
            };
        live_in.insert(bb.insts[i].id, alive.clone());
    }
}

fn get_block_liveness(
    function: &Function,
    live_in: &HashMap<Id, BitSet<usize>>,
    target: &usize,
) -> BitSet<usize> {
    live_in
        .get(&function.body[*target].insts[0].id)
        .cloned()
        .unwrap_or_else(|| BitSet::default())
}

// Also inserts Drop when necessary
fn insert_copy(cctx: &CCtx, function: &mut Function, live_in: &HashMap<Id, BitSet<usize>>) {
    for (i, bb) in function.body.iter_mut().enumerate() {
        let num_args = if i == 0 {
            Some(function.num_args)
        } else {
            None
        };
        insert_copy_bb(cctx, num_args, &mut function.num_vars, bb, live_in);
    }
}
fn insert_copy_bb(
    _cctx: &CCtx,
    num_args: Option<usize>,
    num_vars: &mut usize,
    bb: &mut BasicBlock,
    live_in: &HashMap<Id, BitSet<usize>>,
) {
    let mut copied_var_for = HashMap::new();
    let mut dropped_var_for = HashMap::new();
    for (i, inst) in bb.insts.iter().enumerate() {
        let rhs = moved_rhs_of(inst);
        if let Some(rhs) = rhs {
            let used_next = if i + 1 < bb.insts.len() {
                if let Some(live_in) = live_in.get(&bb.insts[i + 1].id) {
                    live_in.contains(rhs)
                } else {
                    false
                }
            } else {
                false
            };
            if used_next {
                copied_var_for.insert(inst.id, *num_vars);
                *num_vars += 1;
            }
        }
        let lhs = lhs_of(inst);
        if let Some(lhs) = lhs {
            let used_next = if i + 1 < bb.insts.len() {
                if let Some(live_in) = live_in.get(&bb.insts[i + 1].id) {
                    live_in.contains(lhs)
                } else {
                    false
                }
            } else {
                false
            };
            if !used_next {
                dropped_var_for.insert(inst.id, lhs);
            }
        }
    }
    let old_insts = mem::replace(&mut bb.insts, vec![]);
    if let Some(num_args) = num_args {
        // Drop arguments if it is unused
        for arg_id in 0..num_args {
            if !live_in.get(&old_insts[0].id).unwrap().contains(arg_id) {
                bb.insts.push(Inst::new(InstKind::Drop { rhs: arg_id }));
            }
        }
    }
    for mut inst in old_insts {
        if let Some(copied_var) = copied_var_for.get(&inst.id) {
            bb.insts.push(Inst::new(InstKind::Copy {
                lhs: *copied_var,
                rhs: moved_rhs_of(&inst).unwrap(),
            }));
            replace_moved_rhs(&mut inst, *copied_var);
        }
        let id = inst.id;
        bb.insts.push(inst);
        if let Some(dropped_var) = dropped_var_for.get(&id) {
            bb.insts
                .push(Inst::new(InstKind::Drop { rhs: *dropped_var }));
        }
    }
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
        InstKind::Call_ { lhs: _, callee } => Some(*callee),
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
        InstKind::Call_ { callee, .. } => {
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
        InstKind::Call_ { lhs, .. } => Some(*lhs),
    }
}

#[cfg(test)]
mod tests {
    use crate::sir::{
        testing::{insts, FunctionTestingExt, ProgramUnitTestingExt},
        BuiltinKind,
    };

    use super::*;

    #[test]
    fn test_compile() {
        let cctx = CCtx::new();
        let program_unit = ProgramUnit::simple(Function::simple(0, |(x, puts1, tmp1, tmp2)| {
            vec![
                insts::string_literal(x, "Hello, world!"),
                insts::builtin(puts1, BuiltinKind::Puts),
                insts::push_arg(x),
                insts::call(tmp2, puts1),
                insts::builtin(puts1, BuiltinKind::Puts),
                insts::push_arg(x),
                insts::call(tmp2, puts1),
                insts::string_literal(x, "Hello, world!"),
                insts::builtin(puts1, BuiltinKind::Puts),
                insts::push_arg(x),
                insts::call(tmp2, puts1),
                insts::unit_literal(tmp1),
                insts::return_(tmp1),
            ]
        }));
        let program_unit = compile(&cctx, &program_unit);
        assert_eq!(
            program_unit,
            ProgramUnit::simple(Function::simple(0, |(x, puts1, tmp1, tmp2, tmp3)| {
                vec![
                    insts::string_literal(x, "Hello, world!"),
                    insts::builtin(puts1, BuiltinKind::Puts),
                    insts::copy(tmp3, x),
                    insts::push_arg(tmp3),
                    insts::call(tmp2, puts1),
                    insts::drop(tmp2),
                    insts::builtin(puts1, BuiltinKind::Puts),
                    insts::push_arg(x),
                    insts::call(tmp2, puts1),
                    insts::drop(tmp2),
                    insts::string_literal(x, "Hello, world!"),
                    insts::builtin(puts1, BuiltinKind::Puts),
                    insts::push_arg(x),
                    insts::call(tmp2, puts1),
                    insts::drop(tmp2),
                    insts::unit_literal(tmp1),
                    insts::return_(tmp1),
                ]
            }))
        );
    }

    #[test]
    fn test_compile_drop() {
        let cctx = CCtx::new();
        let program_unit = ProgramUnit::simple(Function::simple(0, |(x, puts1, tmp1, tmp2)| {
            vec![
                insts::string_literal(x, "dummy"),
                insts::string_literal(x, "Hello, world!"),
                insts::builtin(puts1, BuiltinKind::Puts),
                insts::push_arg(x),
                insts::call(tmp2, puts1),
                insts::unit_literal(tmp1),
                insts::return_(tmp1),
            ]
        }));
        let program_unit = compile(&cctx, &program_unit);
        assert_eq!(
            program_unit,
            ProgramUnit::simple(Function::simple(0, |(x, puts1, tmp1, tmp2)| {
                vec![
                    insts::string_literal(x, "dummy"),
                    insts::drop(x),
                    insts::string_literal(x, "Hello, world!"),
                    insts::builtin(puts1, BuiltinKind::Puts),
                    insts::push_arg(x),
                    insts::call(tmp2, puts1),
                    insts::drop(tmp2),
                    insts::unit_literal(tmp1),
                    insts::return_(tmp1),
                ]
            }))
        );
    }

    #[test]
    fn test_compile_drop_arg() {
        let cctx = CCtx::new();
        let program_unit = ProgramUnit::simple(Function::simple(1, |(_arg1, tmp1)| {
            vec![insts::unit_literal(tmp1), insts::return_(tmp1)]
        }));
        let program_unit = compile(&cctx, &program_unit);
        assert_eq!(
            program_unit,
            ProgramUnit::simple(Function::simple(1, |(arg, tmp1)| {
                vec![
                    insts::drop(arg),
                    insts::unit_literal(tmp1),
                    insts::return_(tmp1),
                ]
            }))
        );
    }
}
