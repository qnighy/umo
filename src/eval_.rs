use crate::cctx::CCtx;
use crate::rt_ctx::RtCtx;
use crate::sir::ProgramUnit;
use crate::sir_compile::compile;
use crate::sir_eval::eval1;
use crate::sir_typecheck::typecheck;

pub fn eval(ctx: &dyn RtCtx, program_unit: &ProgramUnit) {
    let cctx = CCtx::new();
    typecheck(&cctx, program_unit).unwrap();
    let program_unit = compile(&cctx, program_unit);
    eval1(ctx, &program_unit)
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::sir::testing::{insts, FunctionTestingExt, ProgramUnitTestingExt};
    use crate::sir::Function;
    use crate::testing::MockRtCtx;

    #[test]
    fn test_puts() {
        let ctx = MockRtCtx::new();
        eval(
            &ctx,
            &ProgramUnit::simple(Function::simple(|(x,)| {
                vec![
                    insts::string_literal(x, "Hello, world!"),
                    insts::push_arg(x),
                    insts::puts(),
                    insts::return_(),
                ]
            })),
        );
        assert_eq!(ctx.stdout.lock().unwrap().as_str(), "Hello, world!\n");
    }

    #[test]
    fn test_puts_with_artificial_jump() {
        let ctx = MockRtCtx::new();
        eval(
            &ctx,
            &ProgramUnit::simple(Function::describe(|desc, (x,), (entry, label1)| {
                desc.block(
                    entry,
                    vec![
                        insts::string_literal(x, "Hello, world!"),
                        insts::jump(label1),
                    ],
                );
                desc.block(
                    label1,
                    vec![insts::push_arg(x), insts::puts(), insts::return_()],
                );
            })),
        );
        assert_eq!(ctx.stdout.lock().unwrap().as_str(), "Hello, world!\n");
    }

    #[test]
    fn test_add() {
        let ctx = MockRtCtx::new();
        eval(
            &ctx,
            &ProgramUnit::simple(Function::simple(|(tmp1, tmp2, x)| {
                vec![
                    insts::integer_literal(tmp1, 1),
                    insts::integer_literal(tmp2, 1),
                    insts::push_arg(tmp1),
                    insts::push_arg(tmp2),
                    insts::add(x),
                    insts::push_arg(x),
                    insts::puti(),
                    insts::return_(),
                ]
            })),
        );
        assert_eq!(ctx.stdout.lock().unwrap().as_str(), "2\n");
    }

    #[test]
    fn test_branch_true() {
        let ctx = MockRtCtx::new();
        eval(
            &ctx,
            &ProgramUnit::simple(Function::describe(
                |desc, (x, s), (entry, branch_then, branch_else)| {
                    desc.block(
                        entry,
                        vec![
                            insts::bool_literal(x, true),
                            insts::branch(x, branch_then, branch_else),
                        ],
                    );
                    desc.block(
                        branch_then,
                        vec![
                            insts::string_literal(s, "x is true"),
                            insts::push_arg(s),
                            insts::puts(),
                            insts::return_(),
                        ],
                    );
                    desc.block(
                        branch_else,
                        vec![
                            insts::string_literal(s, "x is false"),
                            insts::push_arg(s),
                            insts::puts(),
                            insts::return_(),
                        ],
                    );
                },
            )),
        );
        assert_eq!(ctx.stdout.lock().unwrap().as_str(), "x is true\n");
    }

    #[test]
    fn test_branch_false() {
        let ctx = MockRtCtx::new();
        eval(
            &ctx,
            &ProgramUnit::simple(Function::describe(
                |desc, (x, s), (entry, branch_then, branch_else)| {
                    desc.block(
                        entry,
                        vec![
                            insts::bool_literal(x, false),
                            insts::branch(x, branch_then, branch_else),
                        ],
                    );
                    desc.block(
                        branch_then,
                        vec![
                            insts::string_literal(s, "x is true"),
                            insts::push_arg(s),
                            insts::puts(),
                            insts::return_(),
                        ],
                    );
                    desc.block(
                        branch_else,
                        vec![
                            insts::string_literal(s, "x is false"),
                            insts::push_arg(s),
                            insts::puts(),
                            insts::return_(),
                        ],
                    );
                },
            )),
        );
        assert_eq!(ctx.stdout.lock().unwrap().as_str(), "x is false\n");
    }

    #[test]
    fn test_sum() {
        // let mut sum = 0;
        // let mut i = 0;
        // while i < 10 {
        //     sum += i;
        //     i += 1;
        // }
        // puti(sum);

        let ctx = MockRtCtx::new();
        eval(
            &ctx,
            &ProgramUnit::simple(Function::describe(
                |desc, (sum, i, tmp1, tmp2, tmp3), (entry, cond, body, end)| {
                    desc.block(
                        entry,
                        vec![
                            // let mut sum = 0;
                            insts::integer_literal(sum, 0),
                            // let mut i = 0;
                            insts::integer_literal(i, 0),
                            // goto cond;
                            insts::jump(1),
                        ],
                    );
                    desc.block(
                        cond,
                        vec![
                            // tmp1 = 10;
                            insts::integer_literal(tmp1, 10),
                            // tmp2 = i < tmp1;
                            insts::push_arg(i),
                            insts::push_arg(tmp1),
                            insts::lt(tmp2),
                            // if tmp2 { goto body; } else { goto end; };
                            insts::branch(tmp2, body, end),
                        ],
                    );
                    desc.block(
                        body,
                        vec![
                            // sum = sum + i;
                            insts::push_arg(sum),
                            insts::push_arg(i),
                            insts::add(sum),
                            // i = i + 1;
                            insts::integer_literal(tmp3, 1),
                            insts::push_arg(i),
                            insts::push_arg(tmp3),
                            insts::add(i),
                            // goto cond;
                            insts::jump(cond),
                        ],
                    );
                    desc.block(
                        end,
                        vec![
                            // puti(sum);
                            insts::push_arg(sum),
                            insts::puti(),
                            // return;
                            insts::return_(),
                        ],
                    );
                },
            )),
        );
        assert_eq!(ctx.stdout.lock().unwrap().as_str(), "45\n");
    }
}
