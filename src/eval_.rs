use crate::cctx::CCtx;
use crate::rt_ctx::RtCtx;
use crate::sir::Function;
use crate::sir_compile::compile;
use crate::sir_eval::eval1;
use crate::sir_typecheck::typecheck;

pub fn eval(ctx: &dyn RtCtx, bb: &Function) {
    let cctx = CCtx::new();
    typecheck(&cctx, bb).unwrap();
    let bb = compile(&cctx, bb);
    eval1(ctx, &bb)
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::sir::testing::{insts, FunctionTestingExt};
    use crate::sir::BasicBlock;
    use crate::testing::MockRtCtx;

    #[test]
    fn test_puts() {
        let ctx = MockRtCtx::new();
        eval(
            &ctx,
            &Function::describe(|(x,)| {
                vec![BasicBlock::new(vec![
                    insts::string_literal(x, "Hello, world!"),
                    insts::push_arg(x),
                    insts::puts(),
                ])]
            }),
        );
        assert_eq!(ctx.stdout.lock().unwrap().as_str(), "Hello, world!\n");
    }

    #[test]
    fn test_puts_with_artificial_jump() {
        let ctx = MockRtCtx::new();
        eval(
            &ctx,
            &Function::describe(|(x,)| {
                vec![
                    BasicBlock::new(vec![
                        insts::string_literal(x, "Hello, world!"),
                        insts::jump(1),
                    ]),
                    BasicBlock::new(vec![insts::push_arg(x), insts::puts()]),
                ]
            }),
        );
        assert_eq!(ctx.stdout.lock().unwrap().as_str(), "Hello, world!\n");
    }

    #[test]
    fn test_add() {
        let ctx = MockRtCtx::new();
        eval(
            &ctx,
            &Function::describe(|(tmp1, tmp2, x)| {
                vec![BasicBlock::new(vec![
                    insts::integer_literal(tmp1, 1),
                    insts::integer_literal(tmp2, 1),
                    insts::push_arg(tmp1),
                    insts::push_arg(tmp2),
                    insts::add(x),
                    insts::push_arg(x),
                    insts::puti(),
                ])]
            }),
        );
        assert_eq!(ctx.stdout.lock().unwrap().as_str(), "2\n");
    }

    #[test]
    fn test_branch_true() {
        let ctx = MockRtCtx::new();
        eval(
            &ctx,
            &Function::describe(|(x, s)| {
                vec![
                    BasicBlock::new(vec![insts::bool_literal(x, true), insts::branch(x, 1, 2)]),
                    BasicBlock::new(vec![
                        insts::string_literal(s, "x is true"),
                        insts::push_arg(s),
                        insts::puts(),
                    ]),
                    BasicBlock::new(vec![
                        insts::string_literal(s, "x is false"),
                        insts::push_arg(s),
                        insts::puts(),
                    ]),
                ]
            }),
        );
        assert_eq!(ctx.stdout.lock().unwrap().as_str(), "x is true\n");
    }

    #[test]
    fn test_branch_false() {
        let ctx = MockRtCtx::new();
        eval(
            &ctx,
            &Function::describe(|(x, s)| {
                vec![
                    BasicBlock::new(vec![insts::bool_literal(x, false), insts::branch(x, 1, 2)]),
                    BasicBlock::new(vec![
                        insts::string_literal(s, "x is true"),
                        insts::push_arg(s),
                        insts::puts(),
                    ]),
                    BasicBlock::new(vec![
                        insts::string_literal(s, "x is false"),
                        insts::push_arg(s),
                        insts::puts(),
                    ]),
                ]
            }),
        );
        assert_eq!(ctx.stdout.lock().unwrap().as_str(), "x is false\n");
    }
}
