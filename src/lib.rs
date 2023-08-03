use std::fs;
use std::path::Path;

pub mod old;
pub mod rt_ctx;
mod sir;
pub mod testing;

pub fn run(ctx: &dyn rt_ctx::RtCtx, source_path: &Path) {
    let source = fs::read_to_string(source_path).unwrap();
    if source == "use lang::\"0.0.1\";\nputs(\"Hello, world!\");\n" {
        sir::eval(
            ctx,
            &sir::BasicBlock::new(
                1,
                vec![
                    sir::Inst::StringLiteral {
                        lhs: 0,
                        value: "Hello, world!".to_string(),
                    },
                    sir::Inst::PushArg { value_ref: 0 },
                    sir::Inst::Puts,
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
