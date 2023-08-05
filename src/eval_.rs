use crate::cctx::CCtx;
use crate::rt_ctx::RtCtx;
use crate::sir::BasicBlock;
use crate::sir_compile::compile;
use crate::sir_eval::eval1;

pub fn eval(ctx: &dyn RtCtx, bb: &BasicBlock) {
    let cctx = CCtx::new();
    let bb = compile(&cctx, bb);
    eval1(ctx, &bb)
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::sir::testing::{insts, BasicBlockTestingExt};
    use crate::testing::MockRtCtx;

    #[test]
    fn test_puts() {
        let ctx = MockRtCtx::new();
        eval(
            &ctx,
            &BasicBlock::describe(|(x,)| {
                vec![
                    insts::string_literal(x, "Hello, world!"),
                    insts::push_arg(x),
                    insts::puts(),
                ]
            }),
        );
        assert_eq!(ctx.stdout.lock().unwrap().as_str(), "Hello, world!\n");
    }
}
