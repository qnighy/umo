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
    use std::sync::Arc;

    use super::*;

    use crate::sir::{Inst, InstKind};
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
}
