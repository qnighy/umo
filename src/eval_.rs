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

    use crate::sir::testing::ProgramUnitTestingExt;
    use crate::sir::{BuiltinKind, Function, Inst};
    use crate::testing::MockRtCtx;

    #[test]
    fn test_puts() {
        let ctx = MockRtCtx::new();
        eval(
            &ctx,
            &ProgramUnit::simple(Function::simple(0, |[x, tmp1, puts1, tmp2]| {
                vec![
                    Inst::literal(x, "Hello, world!"),
                    Inst::builtin(puts1, BuiltinKind::Puts),
                    Inst::push_arg(x),
                    Inst::call(tmp2, puts1),
                    Inst::literal(tmp1, ()),
                    Inst::return_(tmp1),
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
                |[x, tmp1, puts1, tmp2], [entry, label1]| {
                    vec![
                        (
                            entry,
                            vec![Inst::literal(x, "Hello, world!"), Inst::jump(label1)],
                        ),
                        (
                            label1,
                            vec![
                                Inst::builtin(puts1, BuiltinKind::Puts),
                                Inst::push_arg(x),
                                Inst::call(tmp2, puts1),
                                Inst::literal(tmp1, ()),
                                Inst::return_(tmp1),
                            ],
                        ),
                    ]
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
            &ProgramUnit::simple(Function::simple(
                0,
                |[tmp1, tmp2, x, add1, tmp3, puti1, tmp4]| {
                    vec![
                        Inst::literal(tmp1, 1),
                        Inst::literal(tmp2, 1),
                        Inst::builtin(add1, BuiltinKind::Add),
                        Inst::push_arg(tmp1),
                        Inst::push_arg(tmp2),
                        Inst::call(x, add1),
                        Inst::builtin(puti1, BuiltinKind::Puti),
                        Inst::push_arg(x),
                        Inst::call(tmp4, puti1),
                        Inst::literal(tmp3, ()),
                        Inst::return_(tmp3),
                    ]
                },
            )),
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
                |[x, s, tmp1, puts1, tmp2], [entry, branch_then, branch_else]| {
                    vec![
                        (
                            entry,
                            vec![
                                Inst::literal(x, true),
                                Inst::branch(x, branch_then, branch_else),
                            ],
                        ),
                        (
                            branch_then,
                            vec![
                                Inst::literal(s, "x is true"),
                                Inst::builtin(puts1, BuiltinKind::Puts),
                                Inst::push_arg(s),
                                Inst::call(tmp2, puts1),
                                Inst::literal(tmp1, ()),
                                Inst::return_(tmp1),
                            ],
                        ),
                        (
                            branch_else,
                            vec![
                                Inst::literal(s, "x is false"),
                                Inst::builtin(puts1, BuiltinKind::Puts),
                                Inst::push_arg(s),
                                Inst::call(tmp2, puts1),
                                Inst::literal(tmp1, ()),
                                Inst::return_(tmp1),
                            ],
                        ),
                    ]
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
                |[x, s, tmp1, puts1, tmp2], [entry, branch_then, branch_else]| {
                    vec![
                        (
                            entry,
                            vec![
                                Inst::literal(x, false),
                                Inst::branch(x, branch_then, branch_else),
                            ],
                        ),
                        (
                            branch_then,
                            vec![
                                Inst::literal(s, "x is true"),
                                Inst::builtin(puts1, BuiltinKind::Puts),
                                Inst::push_arg(s),
                                Inst::call(tmp2, puts1),
                                Inst::literal(tmp1, ()),
                                Inst::return_(tmp1),
                            ],
                        ),
                        (
                            branch_else,
                            vec![
                                Inst::literal(s, "x is false"),
                                Inst::builtin(puts1, BuiltinKind::Puts),
                                Inst::push_arg(s),
                                Inst::call(tmp2, puts1),
                                Inst::literal(tmp1, ()),
                                Inst::return_(tmp1),
                            ],
                        ),
                    ]
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
                |[sum, i, tmp1, lt1, add1, puti1, tmp2, tmp3, tmp4, tmp5],
                 [entry, cond, body, end]| {
                    vec![
                        (
                            entry,
                            vec![
                                // let mut sum = 0;
                                Inst::literal(sum, 0),
                                // let mut i = 0;
                                Inst::literal(i, 0),
                                // goto cond;
                                Inst::jump(1),
                            ],
                        ),
                        (
                            cond,
                            vec![
                                // tmp1 = 10;
                                Inst::literal(tmp1, 10),
                                // tmp2 = i < tmp1;
                                Inst::builtin(lt1, BuiltinKind::Lt),
                                Inst::push_arg(i),
                                Inst::push_arg(tmp1),
                                Inst::call(tmp2, lt1),
                                // if tmp2 { goto body; } else { goto end; };
                                Inst::branch(tmp2, body, end),
                            ],
                        ),
                        (
                            body,
                            vec![
                                // sum = sum + i;
                                Inst::builtin(add1, BuiltinKind::Add),
                                Inst::push_arg(sum),
                                Inst::push_arg(i),
                                Inst::call(sum, add1),
                                // i = i + 1;
                                Inst::literal(tmp3, 1),
                                Inst::builtin(add1, BuiltinKind::Add),
                                Inst::push_arg(i),
                                Inst::push_arg(tmp3),
                                Inst::call(i, add1),
                                // goto cond;
                                Inst::jump(cond),
                            ],
                        ),
                        (
                            end,
                            vec![
                                // puti(sum);
                                Inst::builtin(puti1, BuiltinKind::Puti),
                                Inst::push_arg(sum),
                                Inst::call(tmp5, puti1),
                                // return;
                                Inst::literal(tmp4, ()),
                                Inst::return_(tmp4),
                            ],
                        ),
                    ]
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
                    Function::simple(0, |[tmp1, fib1, tmp2, tmp3, puti1, tmp4]| {
                        vec![
                            // tmp1 = 10;
                            Inst::literal(tmp1, 10),
                            // tmp2 = fib(tmp1);
                            Inst::closure(fib1, fib),
                            Inst::push_arg(tmp1),
                            Inst::call(tmp2, fib1),
                            // puti(tmp2);
                            Inst::builtin(puti1, BuiltinKind::Puti),
                            Inst::push_arg(tmp2),
                            Inst::call(tmp4, puti1),
                            // return;
                            Inst::literal(tmp3, ()),
                            Inst::return_(tmp3),
                        ]
                    }),
                );
                p.function(
                    fib,
                    Function::describe(
                        1,
                        |[n, tmp1, lt1, tmp2, tmp3, tmp4, add1, tmp5, fib1, tmp6, tmp7, tmp8, tmp9],
                         [entry, branch_then, branch_else]| {
                            vec![
                                (
                                    entry,
                                    vec![
                                        // tmp1 = n < 2;
                                        Inst::literal(tmp2, 2),
                                        Inst::builtin(lt1, BuiltinKind::Lt),
                                        Inst::push_arg(n),
                                        Inst::push_arg(tmp2),
                                        Inst::call(tmp1, lt1),
                                        // if tmp1 { goto branch_then; } else { goto branch_else; };
                                        Inst::branch(tmp1, branch_then, branch_else),
                                    ],
                                ),
                                (
                                    branch_then,
                                    vec![
                                        // return n;
                                        Inst::return_(n),
                                    ],
                                ),
                                (
                                    branch_else,
                                    vec![
                                        // tmp4 = n - 1;
                                        Inst::literal(tmp5, -1),
                                        Inst::builtin(add1, BuiltinKind::Add),
                                        Inst::push_arg(n),
                                        Inst::push_arg(tmp5),
                                        Inst::call(tmp4, add1),
                                        // tmp6 = fib(tmp4);
                                        Inst::closure(fib1, fib),
                                        Inst::push_arg(tmp4),
                                        Inst::call(tmp6, fib1),
                                        // tmp7 = n - 2;
                                        Inst::literal(tmp8, -2),
                                        Inst::builtin(add1, BuiltinKind::Add),
                                        Inst::push_arg(n),
                                        Inst::push_arg(tmp8),
                                        Inst::call(tmp7, add1),
                                        // tmp9 = fib(tmp7);
                                        Inst::closure(fib1, fib),
                                        Inst::push_arg(tmp7),
                                        Inst::call(tmp9, fib1),
                                        // tmp3 = tmp6 + tmp9;
                                        Inst::builtin(add1, BuiltinKind::Add),
                                        Inst::push_arg(tmp6),
                                        Inst::push_arg(tmp9),
                                        Inst::call(tmp3, add1),
                                        // return tmp3;
                                        Inst::return_(tmp3),
                                    ],
                                ),
                            ]
                        },
                    ),
                );
            }),
        );
        assert_eq!(ctx.stdout.lock().unwrap().as_str(), "55\n");
    }
}
