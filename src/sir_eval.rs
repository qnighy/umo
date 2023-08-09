use std::mem;
use std::sync::Arc;

use crate::rt_ctx::RtCtx;
use crate::sir::{BasicBlock, BuiltinKind, Function, InstKind, Literal};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct State {
    vars: Vec<Option<Value>>,
    args: Vec<Value>,
}

pub fn eval1(ctx: &dyn RtCtx, function: &Function) {
    let mut state = State {
        vars: vec![None; function.num_vars],
        args: vec![],
    };
    eval1_bb(ctx, &mut state, &function.body[0]);
}
fn eval1_bb(ctx: &dyn RtCtx, state: &mut State, bb: &BasicBlock) {
    for inst in &bb.insts {
        match &inst.kind {
            InstKind::Copy { lhs, rhs } => {
                state.vars[*lhs] = Some(state.vars[*rhs].as_ref().unwrap().clone());
            }
            InstKind::Literal { lhs, value } => {
                state.vars[*lhs] = Some(Value::from(value.clone()));
            }
            InstKind::PushArg { value_ref } => {
                let value = state.vars[*value_ref].take().unwrap();
                state.args.push(value);
            }
            InstKind::CallBuiltin { lhs, builtin: f } => {
                let args = mem::replace(&mut state.args, vec![]);
                let return_value = eval_builtin(ctx, *f, args);
                if let Some(lhs) = lhs {
                    state.vars[*lhs] = Some(return_value);
                }
            }
        }
    }
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
            Literal::String(s) => Value::String(s),
            Literal::Integer(i) => Value::Integer(i),
        }
    }
}
