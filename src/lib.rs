use std::fs;
use std::path::Path;
use std::sync::Arc;

mod cctx;
mod eval_;
pub mod old;
pub mod rt_ctx;
mod sir;
mod sir_compile;
mod sir_eval;
mod sir_typecheck;
pub mod testing;

pub fn run(ctx: &dyn rt_ctx::RtCtx, source_path: &Path) {
    let source = fs::read_to_string(source_path).unwrap();
    if source == "use lang::\"0.0.1\";\nputs(\"Hello, world!\");\n" {
        eval_::eval(
            ctx,
            &sir::BasicBlock::new(
                1,
                vec![
                    sir::Inst::new(sir::InstKind::Literal {
                        lhs: 0,
                        value: sir::Literal::String(Arc::new("Hello, world!".to_string())),
                    }),
                    sir::Inst::new(sir::InstKind::PushArg { value_ref: 0 }),
                    sir::Inst::new(sir::InstKind::CallBuiltin(sir::BuiltinKind::Puts)),
                ],
            ),
        );
    } else if source == "use lang::\"0.0.1\";\nputi(1 + 1);\n" {
        eval_::eval(
            ctx,
            &sir::BasicBlock::new(
                1,
                vec![
                    // TODO: do not reduce 1 + 1
                    sir::Inst::new(sir::InstKind::Literal {
                        lhs: 0,
                        value: sir::Literal::Integer(2),
                    }),
                    sir::Inst::new(sir::InstKind::PushArg { value_ref: 0 }),
                    sir::Inst::new(sir::InstKind::CallBuiltin(sir::BuiltinKind::Puti)),
                ],
            ),
        );
    } else {
        todo!("Proper parsing and execution");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::testing::MockRtCtx;

    #[test]
    fn test_run_hello() {
        let source_path = std::path::Path::new("examples/hello.umo");
        let ctx = MockRtCtx::new();
        run(&ctx, source_path);
        assert_eq!(ctx.stdout.lock().unwrap().as_str(), "Hello, world!\n");
    }
}
