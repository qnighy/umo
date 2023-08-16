use std::collections::{HashMap, HashSet};

use crate::ast::{BinOp, BuiltinIds, BuiltinKind, Expr, Stmt};
use crate::cctx::Id;
use crate::sir;

#[allow(unused)] // TODO: remove this annotation later
pub fn lower(builtin_ids: &BuiltinIds, stmts: &[Stmt]) -> sir::Function {
    let num_args = 0;
    let mut num_named_vars = num_args;

    let mut vars = HashSet::new();
    collect_vars_stmts(stmts, &mut vars);
    let mut var_ids = vars.into_iter().collect::<Vec<_>>();
    var_ids.sort_unstable();

    let mut var_id_map = HashMap::new();

    for &id in &var_ids {
        var_id_map.insert(id, num_named_vars);
        num_named_vars += 1;
    }

    let mut function = sir::Function::new(
        num_args,
        num_named_vars,
        vec![sir::BasicBlock { insts: vec![] }],
    );
    let mut bb_id = 0;
    let result_var = function.num_vars;
    function.num_vars += 1;
    lower_stmts(
        builtin_ids,
        stmts,
        &var_id_map,
        &mut function,
        &mut bb_id,
        result_var,
    );
    function.body[bb_id]
        .insts
        .push(sir::Inst::new(sir::InstKind::Return { rhs: result_var }));
    function
}

fn lower_stmts(
    builtin_ids: &BuiltinIds,
    stmts: &[Stmt],
    var_id_map: &HashMap<Id, usize>,
    function: &mut sir::Function,
    bb_id: &mut usize,
    result_var: usize,
) {
    for (i, stmt) in stmts.iter().enumerate() {
        let is_last = i == stmts.len() - 1;
        let result_var = if is_last { Some(result_var) } else { None };
        lower_stmt(builtin_ids, stmt, var_id_map, function, bb_id, result_var);
    }
}

fn lower_stmt(
    builtin_ids: &BuiltinIds,
    stmt: &Stmt,
    var_id_map: &HashMap<Id, usize>,
    function: &mut sir::Function,
    bb_id: &mut usize,
    result_var: Option<usize>,
) {
    match stmt {
        Stmt::Let { name: _, id, init } => {
            debug_assert!(!id.is_dummy());

            let var_id = var_id_map[id];
            lower_expr(builtin_ids, init, var_id_map, function, bb_id, var_id);
            if let Some(result_var) = result_var {
                function.body[*bb_id]
                    .insts
                    .push(sir::Inst::new(sir::InstKind::Literal {
                        lhs: result_var,
                        value: sir::Literal::Unit,
                    }))
            }
        }
        Stmt::Expr { expr, use_value } => {
            debug_assert!(result_var.is_some() || !*use_value);
            let stmt_result_var = if *use_value {
                result_var.unwrap()
            } else {
                // Generate dummy temporary
                let result_var = function.num_vars;
                function.num_vars += 1;
                result_var
            };
            lower_expr(
                builtin_ids,
                expr,
                var_id_map,
                function,
                bb_id,
                stmt_result_var,
            );
            if result_var.is_some() && !*use_value {
                // Return unit instead
                function.body[*bb_id]
                    .insts
                    .push(sir::Inst::new(sir::InstKind::Literal {
                        lhs: result_var.unwrap(),
                        value: sir::Literal::Unit,
                    }))
            }
        }
    }
}

fn lower_expr(
    builtin_ids: &BuiltinIds,
    expr: &Expr,
    var_id_map: &HashMap<Id, usize>,
    function: &mut sir::Function,
    bb_id: &mut usize,
    result_var: usize,
) {
    match expr {
        Expr::Var { name: _, id } => {
            let var_id = var_id_map[id];
            function.body[*bb_id]
                .insts
                .push(sir::Inst::new(sir::InstKind::Copy {
                    lhs: result_var,
                    rhs: var_id,
                }));
        }
        Expr::Branch { cond, then, else_ } => {
            let cond_var = lower_expr2(builtin_ids, cond, var_id_map, function, bb_id);

            let branch_bb_id = *bb_id;

            let then_bb_id = function.body.len();
            function.body.push(sir::BasicBlock { insts: vec![] });
            *bb_id = then_bb_id;
            lower_expr(builtin_ids, then, var_id_map, function, bb_id, result_var);

            let else_bb_id = function.body.len();
            function.body.push(sir::BasicBlock { insts: vec![] });
            *bb_id = else_bb_id;
            lower_expr(builtin_ids, else_, var_id_map, function, bb_id, result_var);

            let cont_bb_id = function.body.len();
            function.body.push(sir::BasicBlock { insts: vec![] });
            *bb_id = cont_bb_id;

            function.body[branch_bb_id]
                .insts
                .push(sir::Inst::new(sir::InstKind::Branch {
                    cond: cond_var,
                    branch_then: then_bb_id,
                    branch_else: else_bb_id,
                }));
            function.body[then_bb_id]
                .insts
                .push(sir::Inst::new(sir::InstKind::Jump { target: cont_bb_id }));
            function.body[else_bb_id]
                .insts
                .push(sir::Inst::new(sir::InstKind::Jump { target: cont_bb_id }));
        }
        Expr::While { cond, body } => {
            let prev_bb_id = *bb_id;

            let cond_bb_id = function.body.len();
            function.body.push(sir::BasicBlock { insts: vec![] });
            *bb_id = cond_bb_id;
            lower_expr(builtin_ids, cond, var_id_map, function, bb_id, result_var);

            let body_bb_id = function.body.len();
            function.body.push(sir::BasicBlock { insts: vec![] });
            *bb_id = body_bb_id;
            lower_expr(builtin_ids, body, var_id_map, function, bb_id, result_var);

            let cont_bb_id = function.body.len();
            function.body.push(sir::BasicBlock { insts: vec![] });
            *bb_id = cont_bb_id;

            function.body[prev_bb_id]
                .insts
                .push(sir::Inst::new(sir::InstKind::Jump { target: cond_bb_id }));
            function.body[cond_bb_id]
                .insts
                .push(sir::Inst::new(sir::InstKind::Branch {
                    cond: result_var,
                    branch_then: body_bb_id,
                    branch_else: cont_bb_id,
                }));
            function.body[body_bb_id]
                .insts
                .push(sir::Inst::new(sir::InstKind::Jump { target: cond_bb_id }));
        }
        Expr::Block { stmts } => {
            lower_stmts(builtin_ids, stmts, var_id_map, function, bb_id, result_var)
        }
        Expr::Assign { name: _, id, rhs } => {
            debug_assert!(!id.is_dummy());
            let var_id = var_id_map[id];
            lower_expr(builtin_ids, rhs, var_id_map, function, bb_id, var_id);
            function.body[*bb_id]
                .insts
                .push(sir::Inst::new(sir::InstKind::Literal {
                    lhs: result_var,
                    value: sir::Literal::Unit,
                }));
        }
        Expr::Call { callee, args } => {
            let builtin = if let Expr::Var { name: _, id } = &**callee {
                builtin_ids.builtins.get(id).copied()
            } else {
                None
            };
            if let Some(builtin) = builtin {
                let arg_vars = args
                    .iter()
                    .map(|arg| lower_expr2(builtin_ids, arg, var_id_map, function, bb_id))
                    .collect::<Vec<_>>();
                for &arg_var in &arg_vars {
                    function.body[*bb_id]
                        .insts
                        .push(sir::Inst::new(sir::InstKind::PushArg {
                            value_ref: arg_var,
                        }));
                }
                function.body[*bb_id]
                    .insts
                    .push(sir::Inst::new(sir::InstKind::CallBuiltin {
                        lhs: result_var,
                        builtin: match builtin {
                            BuiltinKind::Puts => sir::BuiltinKind::Puts,
                            BuiltinKind::Puti => sir::BuiltinKind::Puti,
                        },
                    }));
            } else {
                todo!("non-builtin call");
            }
        }
        Expr::IntegerLiteral { value } => {
            function.body[*bb_id]
                .insts
                .push(sir::Inst::new(sir::InstKind::Literal {
                    lhs: result_var,
                    value: sir::Literal::Integer(*value),
                }));
        }
        Expr::BinOp { op, lhs, rhs } => {
            let lhs_var = lower_expr2(builtin_ids, lhs, var_id_map, function, bb_id);
            let rhs_var = lower_expr2(builtin_ids, rhs, var_id_map, function, bb_id);

            let bb = &mut function.body[*bb_id];
            bb.insts.push(sir::Inst::new(sir::InstKind::PushArg {
                value_ref: lhs_var,
            }));
            bb.insts.push(sir::Inst::new(sir::InstKind::PushArg {
                value_ref: rhs_var,
            }));
            bb.insts.push(sir::Inst::new(sir::InstKind::CallBuiltin {
                lhs: result_var,
                builtin: match op {
                    BinOp::Add => sir::BuiltinKind::Add,
                    BinOp::Lt => sir::BuiltinKind::Lt,
                },
            }));
        }
    }
}

fn lower_expr2(
    builtin_ids: &BuiltinIds,
    expr: &Expr,
    var_id_map: &HashMap<Id, usize>,
    function: &mut sir::Function,
    bb_id: &mut usize,
) -> usize {
    let result_var = function.num_vars;
    function.num_vars += 1;
    lower_expr(builtin_ids, expr, var_id_map, function, bb_id, result_var);
    result_var
}

fn collect_vars_stmts(stmts: &[Stmt], vars: &mut HashSet<Id>) {
    for stmt in stmts {
        collect_vars_stmt(stmt, vars);
    }
}

fn collect_vars_stmt(stmt: &Stmt, vars: &mut HashSet<Id>) {
    match stmt {
        Stmt::Let { name: _, id, init } => {
            debug_assert!(!id.is_dummy());
            vars.insert(*id);
            collect_vars_expr(init, vars);
        }
        Stmt::Expr { expr, use_value: _ } => {
            collect_vars_expr(expr, vars);
        }
    }
}

fn collect_vars_expr(expr: &Expr, vars: &mut HashSet<Id>) {
    match expr {
        Expr::Var { name: _, id } => {
            debug_assert!(!id.is_dummy());
            vars.insert(*id);
        }
        Expr::Branch { cond, then, else_ } => {
            collect_vars_expr(cond, vars);
            collect_vars_expr(then, vars);
            collect_vars_expr(else_, vars);
        }
        Expr::While { cond, body } => {
            collect_vars_expr(cond, vars);
            collect_vars_expr(body, vars);
        }
        Expr::Block { stmts } => collect_vars_stmts(stmts, vars),
        Expr::Assign { name: _, id, rhs } => {
            debug_assert!(!id.is_dummy());
            vars.insert(*id);
            collect_vars_expr(rhs, vars);
        }
        Expr::Call { callee, args } => {
            collect_vars_expr(callee, vars);
            for arg in args {
                collect_vars_expr(arg, vars);
            }
        }
        Expr::IntegerLiteral { value: _ } => {}
        Expr::BinOp { op: _, lhs, rhs } => {
            collect_vars_expr(lhs, vars);
            collect_vars_expr(rhs, vars);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::testing::{exprs, stmts};
    use crate::ast::{assign_id_stmts, Scope};
    use crate::cctx::CCtx;
    use crate::sir::testing::{insts, FunctionTestingExt};

    fn assign_id(cctx: &mut CCtx, builtin_ids: &BuiltinIds, mut stmts: Vec<Stmt>) -> Vec<Stmt> {
        let mut scope = Scope::new(builtin_ids);
        assign_id_stmts(cctx, &mut scope, &mut stmts);
        stmts
    }

    #[test]
    fn test_lower_add() {
        let mut cctx = CCtx::new();
        let builtin_ids = BuiltinIds::new(&cctx);
        let s = assign_id(
            &mut cctx,
            &builtin_ids,
            vec![stmts::then_expr(exprs::add(
                exprs::integer_literal(1),
                exprs::integer_literal(2),
            ))],
        );
        let function = lower(&builtin_ids, &s);
        assert_eq!(
            function,
            sir::Function::describe(0, |desc, (tmp1, tmp2, tmp3), (entry,)| {
                desc.block(
                    entry,
                    vec![
                        insts::integer_literal(tmp2, 1),
                        insts::integer_literal(tmp3, 2),
                        insts::push_arg(tmp2),
                        insts::push_arg(tmp3),
                        insts::add(tmp1),
                        insts::return_(tmp1),
                    ],
                );
            })
        );
    }

    #[test]
    fn test_lower_simple_var() {
        let mut cctx = CCtx::new();
        let builtin_ids = BuiltinIds::new(&cctx);
        let s = assign_id(
            &mut cctx,
            &builtin_ids,
            vec![
                stmts::let_("x", exprs::integer_literal(42)),
                stmts::then_expr(exprs::var("x")),
            ],
        );
        let function = lower(&builtin_ids, &s);
        assert_eq!(
            function,
            sir::Function::describe(0, |desc, (x, tmp1), (entry,)| {
                desc.block(
                    entry,
                    vec![
                        insts::integer_literal(x, 42),
                        insts::copy(tmp1, x),
                        insts::return_(tmp1),
                    ],
                );
            })
        );
    }

    #[test]
    fn test_lower_branch() {
        let mut cctx = CCtx::new();
        let builtin_ids = BuiltinIds::new(&cctx);
        let s = assign_id(
            &mut cctx,
            &builtin_ids,
            vec![
                stmts::let_("x", exprs::integer_literal(42)),
                stmts::then_expr(exprs::branch(
                    exprs::var("x"),
                    exprs::integer_literal(1),
                    exprs::integer_literal(2),
                )),
            ],
        );
        let function = lower(&builtin_ids, &s);
        assert_eq!(
            function,
            sir::Function::describe(
                0,
                |desc, (x, tmp1, tmp2), (entry, branch_then, branch_else, cont)| {
                    desc.block(
                        entry,
                        vec![
                            insts::integer_literal(x, 42),
                            insts::copy(tmp2, x),
                            insts::branch(tmp2, branch_then, branch_else),
                        ],
                    );
                    desc.block(
                        branch_then,
                        vec![insts::integer_literal(tmp1, 1), insts::jump(cont)],
                    );
                    desc.block(
                        branch_else,
                        vec![insts::integer_literal(tmp1, 2), insts::jump(cont)],
                    );
                    desc.block(cont, vec![insts::return_(tmp1)]);
                }
            )
        );
    }

    #[test]
    fn test_lower_loop() {
        let mut cctx = CCtx::new();
        let builtin_ids = BuiltinIds::new(&cctx);
        let s = assign_id(
            &mut cctx,
            &builtin_ids,
            vec![
                stmts::let_("x", exprs::integer_literal(42)),
                stmts::then_expr(exprs::while_(
                    exprs::lt(exprs::integer_literal(-1), exprs::var("x")),
                    exprs::block(vec![stmts::then_expr(exprs::assign(
                        "x",
                        exprs::add(exprs::var("x"), exprs::integer_literal(-1)),
                    ))]),
                )),
            ],
        );
        let function = lower(&builtin_ids, &s);
        assert_eq!(
            function,
            sir::Function::describe(
                0,
                |desc, (x, tmp1, tmp2, tmp3, tmp4, tmp5), (entry, cond, body, cont)| {
                    desc.block(
                        entry,
                        vec![insts::integer_literal(x, 42), insts::jump(cond)],
                    );
                    desc.block(
                        cond,
                        vec![
                            insts::integer_literal(tmp2, -1),
                            insts::copy(tmp3, x),
                            insts::push_arg(tmp2),
                            insts::push_arg(tmp3),
                            insts::lt(tmp1),
                            insts::branch(tmp1, body, cont),
                        ],
                    );
                    desc.block(
                        body,
                        vec![
                            insts::copy(tmp4, x),
                            insts::integer_literal(tmp5, -1),
                            insts::push_arg(tmp4),
                            insts::push_arg(tmp5),
                            insts::add(x),
                            insts::unit_literal(tmp1),
                            insts::jump(cond),
                        ],
                    );
                    desc.block(cont, vec![insts::return_(tmp1)]);
                }
            )
        );
    }

    #[test]
    fn test_puti() {
        let mut cctx = CCtx::new();
        let builtin_ids = BuiltinIds::new(&cctx);
        let s = assign_id(
            &mut cctx,
            &builtin_ids,
            vec![stmts::then_expr(exprs::call(
                exprs::var("puti"),
                vec![exprs::integer_literal(42)],
            ))],
        );
        let function = lower(&builtin_ids, &s);
        assert_eq!(
            function,
            sir::Function::describe(0, |desc, (_tmp1, tmp2, tmp3), (entry,)| {
                desc.block(
                    entry,
                    vec![
                        insts::integer_literal(tmp3, 42),
                        insts::push_arg(tmp3),
                        insts::puti(tmp2),
                        insts::return_(tmp2),
                    ],
                );
            })
        );
    }
}
