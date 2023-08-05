// SIR -- Sequential Intermediate Representation

use crate::cctx::CCtx;
use crate::rt_ctx::RtCtx;

use std::collections::{HashMap, HashSet};
use std::mem;
use std::sync::Arc;

// Define BasicBlock
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BasicBlock {
    // To be hoisted to FunDef
    pub num_vars: usize,
    pub insts: Vec<Inst>,
}

impl BasicBlock {
    pub fn new(num_vars: usize, insts: Vec<Inst>) -> Self {
        Self { num_vars, insts }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Inst {
    pub id: usize,
    pub kind: InstKind,
}

impl Inst {
    pub fn new(kind: InstKind) -> Self {
        Self { id: 0, kind }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum InstKind {
    Copy { lhs: usize, rhs: usize },
    StringLiteral { lhs: usize, value: Arc<String> },
    PushArg { value_ref: usize },
    Puts,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct State {
    vars: Vec<Value>,
    args: Vec<Value>,
}

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
        inst.id = 0;
    }
}

fn liveness(_cctx: &CCtx, bb: &BasicBlock) -> HashMap<usize, HashSet<usize>> {
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
            InstKind::StringLiteral { lhs, .. } => {
                alive.remove(lhs);
            }
            InstKind::PushArg { value_ref } => {
                alive.insert(*value_ref);
            }
            InstKind::Puts => {}
        }
        live_in.insert(bb.insts[i].id, alive.clone());
    }
    live_in
}

fn insert_copy(_cctx: &CCtx, bb: &mut BasicBlock, live_in: &HashMap<usize, HashSet<usize>>) {
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
        InstKind::StringLiteral { .. } => None,
        InstKind::PushArg { value_ref } => Some(*value_ref),
        InstKind::Puts => None,
    }
}

fn replace_moved_rhs(bb: &mut Inst, to: usize) {
    match &mut bb.kind {
        InstKind::Copy { .. } => {
            unreachable!();
        }
        InstKind::StringLiteral { .. } => {
            unreachable!();
        }
        InstKind::PushArg { value_ref } => {
            *value_ref = to;
        }
        InstKind::Puts => {
            unreachable!();
        }
    }
}

pub fn eval(ctx: &dyn RtCtx, bb: &BasicBlock) {
    let cctx = CCtx::new();
    let bb = compile(&cctx, bb);
    eval1(ctx, &bb)
}
fn eval1(ctx: &dyn RtCtx, bb: &BasicBlock) {
    let mut state = State {
        vars: vec![Value::String(Arc::new(String::new())); bb.num_vars],
        args: vec![],
    };
    for inst in &bb.insts {
        match &inst.kind {
            InstKind::Copy { lhs, rhs } => {
                state.vars[*lhs] = state.vars[*rhs].clone();
            }
            InstKind::StringLiteral { lhs, value } => {
                state.vars[*lhs] = Value::String(value.clone());
            }
            InstKind::PushArg { value_ref } => {
                let value = &state.vars[*value_ref];
                state.args.push(value.clone());
            }
            InstKind::Puts => {
                let args = mem::replace(&mut state.args, vec![]);
                assert_eq!(args.len(), 1);
                #[allow(irrefutable_let_patterns)]
                if let Value::String(s) = &args[0] {
                    ctx.puts(s);
                } else {
                    panic!("Expected string");
                }
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum Value {
    String(Arc<String>),
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::testing::MockRtCtx;

    #[test]
    fn test_puts() {
        let ctx = MockRtCtx::new();
        eval(
            &ctx,
            &BasicBlock::new(
                1,
                vec![
                    Inst::new(InstKind::StringLiteral {
                        lhs: 0,
                        value: Arc::new("Hello, world!".to_string()),
                    }),
                    Inst::new(InstKind::PushArg { value_ref: 0 }),
                    Inst::new(InstKind::Puts),
                ],
            ),
        );
        assert_eq!(ctx.stdout.lock().unwrap().as_str(), "Hello, world!\n");
    }

    #[test]
    fn test_compile() {
        let cctx = CCtx::new();
        let bb = BasicBlock::new(
            1,
            vec![
                Inst::new(InstKind::StringLiteral {
                    lhs: 0,
                    value: Arc::new("Hello, world!".to_string()),
                }),
                Inst::new(InstKind::PushArg { value_ref: 0 }),
                Inst::new(InstKind::Puts),
                Inst::new(InstKind::PushArg { value_ref: 0 }),
                Inst::new(InstKind::Puts),
                Inst::new(InstKind::StringLiteral {
                    lhs: 0,
                    value: Arc::new("Hello, world!".to_string()),
                }),
                Inst::new(InstKind::PushArg { value_ref: 0 }),
                Inst::new(InstKind::Puts),
            ],
        );
        let bb = compile(&cctx, &bb);
        assert_eq!(
            bb,
            BasicBlock::new(
                2,
                vec![
                    Inst::new(InstKind::StringLiteral {
                        lhs: 0,
                        value: Arc::new("Hello, world!".to_string()),
                    }),
                    Inst::new(InstKind::Copy { lhs: 1, rhs: 0 }),
                    Inst::new(InstKind::PushArg { value_ref: 1 }),
                    Inst::new(InstKind::Puts),
                    Inst::new(InstKind::PushArg { value_ref: 0 }),
                    Inst::new(InstKind::Puts),
                    Inst::new(InstKind::StringLiteral {
                        lhs: 0,
                        value: Arc::new("Hello, world!".to_string()),
                    }),
                    Inst::new(InstKind::PushArg { value_ref: 0 }),
                    Inst::new(InstKind::Puts),
                ],
            )
        );
    }
}
