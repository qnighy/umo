use std::mem;
use std::sync::Arc;

use crate::rt_ctx::RtCtx;
use crate::sir::{BasicBlock, BuiltinKind, InstKind};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct State {
    vars: Vec<Option<Value>>,
    args: Vec<Value>,
}

pub fn eval1(ctx: &dyn RtCtx, bb: &BasicBlock) {
    let mut state = State {
        vars: vec![None; bb.num_vars],
        args: vec![],
    };
    for inst in &bb.insts {
        match &inst.kind {
            InstKind::Copy { lhs, rhs } => {
                state.vars[*lhs] = Some(state.vars[*rhs].as_ref().unwrap().clone());
            }
            InstKind::StringLiteral { lhs, value } => {
                state.vars[*lhs] = Some(Value::String(value.clone()));
            }
            InstKind::PushArg { value_ref } => {
                let value = state.vars[*value_ref].take().unwrap();
                state.args.push(value);
            }
            InstKind::CallBuiltin(f) => {
                let args = mem::replace(&mut state.args, vec![]);
                eval_builtin(ctx, *f, args);
            }
        }
    }
}

fn eval_builtin(ctx: &dyn RtCtx, f: BuiltinKind, args: Vec<Value>) -> Value {
    match f {
        BuiltinKind::Puts => {
            assert_eq!(args.len(), 1);
            #[allow(irrefutable_let_patterns)]
            if let Value::String(s) = &args[0] {
                ctx.puts(s);
            } else {
                panic!("Expected string");
            }
            // TODO: replace with unit
            Value::String(Arc::new("".to_string()))
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum Value {
    String(Arc<String>),
}
