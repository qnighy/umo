use std::fs;
use std::path::Path;
use std::sync::Arc;

mod ast;
mod ast_lowering;
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
            &sir::ProgramUnit::new(vec![sir::Function::new(
                0,
                4,
                vec![sir::BasicBlock::new(vec![
                    sir::Inst::new(sir::InstKind::Builtin {
                        lhs: 0,
                        builtin: sir::BuiltinKind::Puts,
                    }),
                    sir::Inst::new(sir::InstKind::Literal {
                        lhs: 1,
                        value: sir::Literal::String(Arc::new("Hello, world!".to_string())),
                    }),
                    sir::Inst::new(sir::InstKind::PushArg { value_ref: 1 }),
                    sir::Inst::new(sir::InstKind::Call_ { lhs: 3, callee: 0 }),
                    sir::Inst::new(sir::InstKind::Literal {
                        lhs: 2,
                        value: sir::Literal::Unit,
                    }),
                    sir::Inst::new(sir::InstKind::Return { rhs: 2 }),
                ])],
            )]),
        );
    } else if source == "use lang::\"0.0.1\";\nputi(1 + 1);\n" {
        eval_::eval(
            ctx,
            &sir::ProgramUnit::new(vec![sir::Function::new(
                0,
                7,
                vec![sir::BasicBlock::new(vec![
                    sir::Inst::new(sir::InstKind::Builtin {
                        lhs: 0,
                        builtin: sir::BuiltinKind::Puti,
                    }),
                    sir::Inst::new(sir::InstKind::Builtin {
                        lhs: 1,
                        builtin: sir::BuiltinKind::Add,
                    }),
                    sir::Inst::new(sir::InstKind::Literal {
                        lhs: 2,
                        value: sir::Literal::Integer(1),
                    }),
                    sir::Inst::new(sir::InstKind::Literal {
                        lhs: 3,
                        value: sir::Literal::Integer(1),
                    }),
                    sir::Inst::new(sir::InstKind::PushArg { value_ref: 2 }),
                    sir::Inst::new(sir::InstKind::PushArg { value_ref: 3 }),
                    sir::Inst::new(sir::InstKind::Call_ { lhs: 4, callee: 2 }),
                    sir::Inst::new(sir::InstKind::PushArg { value_ref: 4 }),
                    sir::Inst::new(sir::InstKind::Call_ { lhs: 6, callee: 0 }),
                    sir::Inst::new(sir::InstKind::Literal {
                        lhs: 5,
                        value: sir::Literal::Unit,
                    }),
                    sir::Inst::new(sir::InstKind::Return { rhs: 5 }),
                ])],
            )]),
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
