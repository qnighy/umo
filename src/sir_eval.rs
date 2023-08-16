use std::mem;
use std::sync::Arc;

use crate::rt_ctx::RtCtx;
use crate::sir::{BasicBlock, BuiltinKind, Function, InstKind, Literal, ProgramUnit};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct State {
    vars: Vec<Option<Value>>,
    args: Vec<Value>,
}

pub fn eval1(ctx: &dyn RtCtx, program_unit: &ProgramUnit) {
    eval1_function(ctx, program_unit, &program_unit.functions[0], vec![]);
}
fn eval1_function(
    ctx: &dyn RtCtx,
    program_unit: &ProgramUnit,
    function: &Function,
    received_args: Vec<Value>,
) -> Value {
    assert!(function.num_args <= function.num_vars);
    let mut state = State {
        vars: vec![None; function.num_vars],
        args: vec![],
    };
    for (i, received_arg) in received_args.into_iter().enumerate() {
        state.vars[i] = Some(received_arg);
    }
    let mut current_bb_id = 0;
    loop {
        let bb = &function.body[current_bb_id];
        match eval1_bb(ctx, &mut state, program_unit, bb) {
            BlockResult::Return(value) => {
                return value;
            }
            BlockResult::Jump(next_bb_id) => {
                current_bb_id = next_bb_id;
            }
        }
    }
}
#[derive(Debug)]
enum BlockResult {
    Return(Value),
    Jump(usize),
}
fn eval1_bb(
    ctx: &dyn RtCtx,
    state: &mut State,
    program_unit: &ProgramUnit,
    bb: &BasicBlock,
) -> BlockResult {
    for inst in &bb.insts {
        match &inst.kind {
            InstKind::Jump { target } => {
                return BlockResult::Jump(*target);
            }
            InstKind::Branch {
                cond,
                branch_then,
                branch_else,
            } => {
                let cond = state.vars[*cond].as_ref().unwrap();
                let cond = if let Value::Integer(i) = cond {
                    *i != 0
                } else {
                    panic!("Expected integer");
                };
                return BlockResult::Jump(if cond { *branch_then } else { *branch_else });
            }
            InstKind::Return { rhs } => {
                return BlockResult::Return(state.vars[*rhs].as_ref().unwrap().clone());
            }
            InstKind::Copy { lhs, rhs } => {
                state.vars[*lhs] = Some(state.vars[*rhs].as_ref().unwrap().clone());
            }
            InstKind::Drop { rhs } => {
                state.vars[*rhs] = None;
            }
            InstKind::Literal { lhs, value } => {
                state.vars[*lhs] = Some(Value::from(value.clone()));
            }
            InstKind::PushArg { value_ref } => {
                let value = state.vars[*value_ref].take().unwrap();
                state.args.push(value);
            }
            InstKind::Call { lhs, callee } => {
                let args = mem::replace(&mut state.args, vec![]);
                let return_value =
                    eval1_function(ctx, program_unit, &program_unit.functions[*callee], args);
                state.vars[*lhs] = Some(return_value);
            }
            InstKind::CallBuiltin { lhs, builtin: f } => {
                let args = mem::replace(&mut state.args, vec![]);
                let return_value = eval_builtin(ctx, *f, args);
                state.vars[*lhs] = Some(return_value);
            }
        }
    }
    unreachable!("Missing tail instruction");
}

fn eval_builtin(ctx: &dyn RtCtx, f: BuiltinKind, args: Vec<Value>) -> Value {
    match f {
        BuiltinKind::Add => {
            assert_eq!(args.len(), 2);
            let Value::Integer(i) = &args[0]  else {
                panic!("Expected integer");
            };
            let Value::Integer(j) = &args[1] else {
                panic!("Expected integer");
            };
            Value::Integer(i + j)
        }
        BuiltinKind::Lt => {
            assert_eq!(args.len(), 2);
            let Value::Integer(i) = &args[0]  else {
                panic!("Expected integer");
            };
            let Value::Integer(j) = &args[1] else {
                panic!("Expected integer");
            };
            Value::Integer((i < j) as i32)
        }
        BuiltinKind::Puts => {
            assert_eq!(args.len(), 1);
            if let Value::String(s) = &args[0] {
                ctx.puts(s);
            } else {
                panic!("Expected string");
            }
            Value::Integer(0)
        }
        BuiltinKind::Puti => {
            assert_eq!(args.len(), 1);
            if let Value::Integer(i) = &args[0] {
                ctx.puts(&i.to_string());
            } else {
                panic!("Expected integer");
            }
            Value::Integer(0)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum Value {
    String(Arc<String>),
    Integer(i32),
}

impl From<Literal> for Value {
    fn from(l: Literal) -> Self {
        match l {
            Literal::Unit => Value::Integer(0),
            Literal::String(s) => Value::String(s),
            Literal::Integer(i) => Value::Integer(i),
            Literal::Bool(b) => Value::Integer(b as i32),
        }
    }
}
