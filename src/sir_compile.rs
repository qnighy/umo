use std::collections::{HashMap, HashSet};
use std::mem;

use crate::cctx::{CCtx, Id};
use crate::sir::{BasicBlock, Inst, InstKind};

pub fn compile(cctx: &CCtx, bb: &BasicBlock) -> BasicBlock {
    let mut bb = bb.clone();
    assign_id(cctx, &mut bb);
    let live_in = liveness(cctx, &bb);
    insert_copy(cctx, &mut bb, &live_in);
    unassign_id(&mut bb);
    bb
}

fn assign_id(cctx: &CCtx, bb: &mut BasicBlock) {
    for inst in &mut bb.insts {
        inst.id = cctx.id_gen.fresh();
    }
}

fn unassign_id(bb: &mut BasicBlock) {
    for inst in &mut bb.insts {
        inst.id = Id::default();
    }
}

fn liveness(_cctx: &CCtx, bb: &BasicBlock) -> HashMap<Id, HashSet<usize>> {
    let mut live_in = HashMap::new();
    let mut i = bb.insts.len();
    let mut alive = HashSet::new();
    while i > 0 {
        i -= 1;
        match &bb.insts[i].kind {
            InstKind::Copy { lhs, rhs } => {
                alive.remove(lhs);
                alive.insert(*rhs);
            }
            InstKind::Literal { lhs, .. } => {
                alive.remove(lhs);
            }
            InstKind::PushArg { value_ref } => {
                alive.insert(*value_ref);
            }
            InstKind::CallBuiltin(_) => {}
        }
        live_in.insert(bb.insts[i].id, alive.clone());
    }
    live_in
}

fn insert_copy(_cctx: &CCtx, bb: &mut BasicBlock, live_in: &HashMap<Id, HashSet<usize>>) {
    let mut copied_var_for = HashMap::new();
    for (i, inst) in bb.insts.iter().enumerate() {
        let rhs = moved_rhs_of(inst);
        if let Some(rhs) = rhs {
            let used_next = if i + 1 < bb.insts.len() {
                if let Some(live_in) = live_in.get(&bb.insts[i + 1].id) {
                    live_in.contains(&rhs)
                } else {
                    false
                }
            } else {
                false
            };
            if used_next {
                copied_var_for.insert(inst.id, bb.num_vars);
                bb.num_vars += 1;
            }
        }
    }
    let old_insts = mem::replace(&mut bb.insts, vec![]);
    for mut inst in old_insts {
        if let Some(copied_var) = copied_var_for.get(&inst.id) {
            bb.insts.push(Inst::new(InstKind::Copy {
                lhs: *copied_var,
                rhs: moved_rhs_of(&inst).unwrap(),
            }));
            replace_moved_rhs(&mut inst, *copied_var);
        }
        bb.insts.push(inst);
    }
}

fn moved_rhs_of(bb: &Inst) -> Option<usize> {
    match &bb.kind {
        InstKind::Copy { .. } => None,
        InstKind::Literal { .. } => None,
        InstKind::PushArg { value_ref } => Some(*value_ref),
        InstKind::CallBuiltin(_) => None,
    }
}

fn replace_moved_rhs(bb: &mut Inst, to: usize) {
    match &mut bb.kind {
        InstKind::Copy { .. } => {
            unreachable!();
        }
        InstKind::Literal { .. } => {
            unreachable!();
        }
        InstKind::PushArg { value_ref } => {
            *value_ref = to;
        }
        InstKind::CallBuiltin(_) => {
            unreachable!();
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::sir::testing::{insts, BasicBlockTestingExt};

    use super::*;

    #[test]
    fn test_compile() {
        let cctx = CCtx::new();
        let bb = BasicBlock::describe(|(x,)| {
            vec![
                insts::string_literal(x, "Hello, world!"),
                insts::push_arg(x),
                insts::puts(),
                insts::push_arg(x),
                insts::puts(),
                insts::string_literal(x, "Hello, world!"),
                insts::push_arg(x),
                insts::puts(),
            ]
        });
        let bb = compile(&cctx, &bb);
        assert_eq!(
            bb,
            BasicBlock::describe(|(x, tmp1)| {
                vec![
                    insts::string_literal(x, "Hello, world!"),
                    insts::copy(tmp1, x),
                    insts::push_arg(tmp1),
                    insts::puts(),
                    insts::push_arg(x),
                    insts::puts(),
                    insts::string_literal(x, "Hello, world!"),
                    insts::push_arg(x),
                    insts::puts(),
                ]
            })
        );
    }
}
