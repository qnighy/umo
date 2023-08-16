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
            &ProgramUnit::simple(Function::simple(0, |(x, tmp1, tmp2)| {
                vec![
                    insts::string_literal(x, "Hello, world!"),
                    insts::push_arg(x),
                    insts::puts(tmp2),
                    insts::unit_literal(tmp1),
                    insts::return_(tmp1),
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
            &ProgramUnit::simple(Function::describe(
                0,
                |desc, (x, tmp1, tmp2), (entry, label1)| {
                    desc.block(
                        entry,
                        vec![
                            insts::string_literal(x, "Hello, world!"),
                            insts::jump(label1),
                        ],
                    );
                    desc.block(
                        label1,
                        vec![
                            insts::push_arg(x),
                            insts::puts(tmp2),
                            insts::unit_literal(tmp1),
                            insts::return_(tmp1),
                        ],
                    );
                },
            )),
        );
        assert_eq!(ctx.stdout.lock().unwrap().as_str(), "Hello, world!\n");
    }

    #[test]
    fn test_add() {
        let ctx = MockRtCtx::new();
        eval(
            &ctx,
            &ProgramUnit::simple(Function::simple(0, |(tmp1, tmp2, x, tmp3, tmp4)| {
                vec![
                    insts::integer_literal(tmp1, 1),
                    insts::integer_literal(tmp2, 1),
                    insts::push_arg(tmp1),
                    insts::push_arg(tmp2),
                    insts::add(x),
                    insts::push_arg(x),
                    insts::puti(tmp4),
                    insts::unit_literal(tmp3),
                    insts::return_(tmp3),
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
                0,
                |desc, (x, s, tmp1, tmp2), (entry, branch_then, branch_else)| {
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
                            insts::puts(tmp2),
                            insts::unit_literal(tmp1),
                            insts::return_(tmp1),
                        ],
                    );
                    desc.block(
                        branch_else,
                        vec![
                            insts::string_literal(s, "x is false"),
                            insts::push_arg(s),
                            insts::puts(tmp2),
                            insts::unit_literal(tmp1),
                            insts::return_(tmp1),
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
                0,
                |desc, (x, s, tmp1, tmp2), (entry, branch_then, branch_else)| {
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
                            insts::puts(tmp2),
                            insts::unit_literal(tmp1),
                            insts::return_(tmp1),
                        ],
                    );
                    desc.block(
                        branch_else,
                        vec![
                            insts::string_literal(s, "x is false"),
                            insts::push_arg(s),
                            insts::puts(tmp2),
                            insts::unit_literal(tmp1),
                            insts::return_(tmp1),
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
                0,
                |desc, (sum, i, tmp1, tmp2, tmp3, tmp4, tmp5), (entry, cond, body, end)| {
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
                            insts::puti(tmp5),
                            // return;
                            insts::unit_literal(tmp4),
                            insts::return_(tmp4),
                        ],
                    );
                },
            )),
        );
        assert_eq!(ctx.stdout.lock().unwrap().as_str(), "45\n");
    }

    #[test]
    fn test_fib() {
        // let fib = fn(n) {
        //     if n < 2 {
        //         n
        //     } else {
        //         fib(n - 1) + fib(n - 2)
        //     }
        // };
        // puti(fib(10));

        let ctx = MockRtCtx::new();
        eval(
            &ctx,
            &ProgramUnit::describe(|p, (entry, fib)| {
                p.function(
                    entry,
                    Function::simple(0, |(tmp1, tmp2, tmp3, tmp4)| {
                        vec![
                            // tmp1 = 10;
                            insts::integer_literal(tmp1, 10),
                            // tmp2 = fib(tmp1);
                            insts::push_arg(tmp1),
                            insts::call(tmp2, fib),
                            // puti(tmp2);
                            insts::push_arg(tmp2),
                            insts::puti(tmp4),
                            // return;
                            insts::unit_literal(tmp3),
                            insts::return_(tmp3),
                        ]
                    }),
                );
                p.function(
                    fib,
                    Function::describe(
                        1,
                        |desc,
                         (n, tmp1, tmp2, tmp3, tmp4, tmp5, tmp6, tmp7, tmp8, tmp9),
                         (entry, branch_then, branch_else)| {
                            desc.block(
                                entry,
                                vec![
                                    // tmp1 = n < 2;
                                    insts::integer_literal(tmp2, 2),
                                    insts::push_arg(n),
                                    insts::push_arg(tmp2),
                                    insts::lt(tmp1),
                                    // if tmp1 { goto branch_then; } else { goto branch_else; };
                                    insts::branch(tmp1, branch_then, branch_else),
                                ],
                            );
                            desc.block(
                                branch_then,
                                vec![
                                    // return n;
                                    insts::return_(n),
                                ],
                            );
                            desc.block(
                                branch_else,
                                vec![
                                    // tmp4 = n - 1;
                                    insts::integer_literal(tmp5, -1),
                                    insts::push_arg(n),
                                    insts::push_arg(tmp5),
                                    insts::add(tmp4),
                                    // tmp6 = fib(tmp4);
                                    insts::push_arg(tmp4),
                                    insts::call(tmp6, fib),
                                    // tmp7 = n - 2;
                                    insts::integer_literal(tmp8, -2),
                                    insts::push_arg(n),
                                    insts::push_arg(tmp8),
                                    insts::add(tmp7),
                                    // tmp9 = fib(tmp7);
                                    insts::push_arg(tmp7),
                                    insts::call(tmp9, fib),
                                    // tmp3 = tmp6 + tmp9;
                                    insts::push_arg(tmp6),
                                    insts::push_arg(tmp9),
                                    insts::add(tmp3),
                                    // return tmp3;
                                    insts::return_(tmp3),
                                ],
                            );
                        },
                    ),
                );
            }),
        );
        assert_eq!(ctx.stdout.lock().unwrap().as_str(), "55\n");
    }
}
