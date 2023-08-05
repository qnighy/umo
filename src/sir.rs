// SIR -- Sequential Intermediate Representation

use crate::cctx::CCtx;
use crate::rt_ctx::RtCtx;

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
pub enum Inst {
    StringLiteral { lhs: usize, value: Arc<String> },
    PushArg { value_ref: usize },
    Puts,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct State {
    vars: Vec<Value>,
    args: Vec<Value>,
}

pub fn compile(cctx: &CCtx, input: &BasicBlock) -> BasicBlock {
    input.clone()
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
        match inst {
            Inst::StringLiteral { lhs, value } => {
                state.vars[*lhs] = Value::String(value.clone());
            }
            Inst::PushArg { value_ref } => {
                let value = &state.vars[*value_ref];
                state.args.push(value.clone());
            }
            Inst::Puts => {
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
                    Inst::StringLiteral {
                        lhs: 0,
                        value: Arc::new("Hello, world!".to_string()),
                    },
                    Inst::PushArg { value_ref: 0 },
                    Inst::Puts,
                ],
            ),
        );
        assert_eq!(ctx.stdout.lock().unwrap().as_str(), "Hello, world!\n");
    }
}
