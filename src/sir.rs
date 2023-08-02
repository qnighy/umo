// SIR -- Sequential Intermediate Representation

use crate::rt_ctx::RtCtx;

// Define BasicBlock
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BasicBlock {
    pub insts: Vec<Inst>,
}

impl BasicBlock {
    pub fn new(insts: Vec<Inst>) -> Self {
        Self { insts }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Inst {
    Puts,
}

pub fn eval(ctx: &dyn RtCtx, bb: &BasicBlock) {
    for inst in &bb.insts {
        match inst {
            Inst::Puts => {
                ctx.puts("Hello, world!");
            }
        }
    }
}
