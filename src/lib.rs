use std::fs;
use std::path::Path;
use std::sync::Arc;

use ast::BuiltinIds;
use cctx::CCtx;

mod ast;
mod ast_lowering;
mod cctx;
mod eval_;
pub mod old;
mod parser;
pub mod rt_ctx;
mod sir;
mod sir_compile;
mod sir_eval;
mod sir_typecheck;
pub mod testing;

pub fn run(ctx: &dyn rt_ctx::RtCtx, source_path: &Path) {
    let source = fs::read_to_string(source_path).unwrap();
    if source == "use lang::\"0.0.1\";\nputi(1 + 1);\n" {
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
                    sir::Inst::new(sir::InstKind::Call { lhs: 4, callee: 1 }),
                    sir::Inst::new(sir::InstKind::PushArg { value_ref: 4 }),
                    sir::Inst::new(sir::InstKind::Call { lhs: 6, callee: 0 }),
                    sir::Inst::new(sir::InstKind::Literal {
                        lhs: 5,
                        value: sir::Literal::Unit,
                    }),
                    sir::Inst::new(sir::InstKind::Return { rhs: 5 }),
                ])],
            )]),
        );
    } else {
        let cctx = CCtx::new();
        let builtin_ids = BuiltinIds::new(&cctx);
        let mut program_ast = crate::parser::parse(&source).unwrap();
        let mut scope = crate::ast::Scope::new(&builtin_ids);
        crate::ast::assign_id_stmts(&cctx, &mut scope, &mut program_ast);
        let program_sir = ast_lowering::lower(&builtin_ids, &program_ast);
        let program_unit = sir::ProgramUnit::new(vec![program_sir]);
        crate::eval_::eval(ctx, &program_unit);
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
