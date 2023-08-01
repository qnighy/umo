// SIR -- Sequential Intermediate Representation

use crate::rt_ctx::RtCtx;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Inst {
    Puts,
}

pub fn eval(ctx: &dyn RtCtx, inst: &Inst) {
    match inst {
        Inst::Puts => {
            ctx.puts("Hello, world!");
        }
    }
}
