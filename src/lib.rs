use std::fs;
use std::path::Path;

use ast::BuiltinIds;
use cctx::CCtx;

mod ast;
mod ast_lowering;
mod cctx;
mod eval_;
pub mod ntype;
mod parser;
pub mod rt_ctx;
mod sir;
mod sir_compile;
mod sir_eval;
mod sir_typecheck;
mod sir_validation;
pub mod testing;
mod util;

pub fn run(ctx: &dyn rt_ctx::RtCtx, source_path: &Path) {
    let source = fs::read_to_string(source_path).unwrap();
    let cctx = CCtx::new();
    let builtin_ids = BuiltinIds::new(&cctx);
    let mut program_ast = crate::parser::parse(&source).unwrap();
    let mut scope = crate::ast::Scope::new(&builtin_ids);
    crate::ast::assign_id_stmts(&cctx, &mut scope, &mut program_ast);
    let program_sir = ast_lowering::lower(&builtin_ids, &program_ast);
    let program_unit = sir::ProgramUnit::new(vec![program_sir]);
    crate::eval_::eval(ctx, &program_unit);
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
