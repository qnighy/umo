use std::collections::{HashMap, HashSet};

use crate::ast::{Expr, Stmt};
use crate::cctx::Id;
use crate::sir;

#[allow(unused)] // TODO: remove this annotation later
pub fn lower(stmts: &[Stmt]) -> sir::Function {
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
    lower_stmts(stmts, &var_id_map, &mut function, &mut bb_id, result_var);
    function.body[bb_id]
        .insts
        .push(sir::Inst::new(sir::InstKind::Return { rhs: result_var }));
    function
}

fn lower_stmts(
    stmts: &[Stmt],
    var_id_map: &HashMap<Id, usize>,
    function: &mut sir::Function,
    bb_id: &mut usize,
    result_var: usize,
) {
    for (i, stmt) in stmts.iter().enumerate() {
        let is_last = i == stmts.len() - 1;
        let result_var = if is_last { Some(result_var) } else { None };
        lower_stmt(stmt, var_id_map, function, bb_id, result_var);
    }
}

fn lower_stmt(
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
            lower_expr(init, var_id_map, function, bb_id, var_id);
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
            lower_expr(expr, var_id_map, function, bb_id, stmt_result_var);
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
            let cond_var = lower_expr2(cond, var_id_map, function, bb_id);

            let branch_bb_id = *bb_id;

            let then_bb_id = function.body.len();
            function.body.push(sir::BasicBlock { insts: vec![] });
            *bb_id = then_bb_id;
            lower_expr(then, var_id_map, function, bb_id, result_var);

            let else_bb_id = function.body.len();
            function.body.push(sir::BasicBlock { insts: vec![] });
            *bb_id = else_bb_id;
            lower_expr(else_, var_id_map, function, bb_id, result_var);

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
        Expr::IntegerLiteral { value } => {
            function.body[*bb_id]
                .insts
                .push(sir::Inst::new(sir::InstKind::Literal {
                    lhs: result_var,
                    value: sir::Literal::Integer(*value),
                }));
        }
        Expr::Add { lhs, rhs } => {
            let lhs_var = lower_expr2(lhs, var_id_map, function, bb_id);
            let rhs_var = lower_expr2(rhs, var_id_map, function, bb_id);

            let bb = &mut function.body[*bb_id];
            bb.insts.push(sir::Inst::new(sir::InstKind::PushArg {
                value_ref: lhs_var,
            }));
            bb.insts.push(sir::Inst::new(sir::InstKind::PushArg {
                value_ref: rhs_var,
            }));
            bb.insts.push(sir::Inst::new(sir::InstKind::CallBuiltin {
                lhs: Some(result_var),
                builtin: sir::BuiltinKind::Add,
            }));
        }
    }
}

fn lower_expr2(
    expr: &Expr,
    var_id_map: &HashMap<Id, usize>,
    function: &mut sir::Function,
    bb_id: &mut usize,
) -> usize {
    let result_var = function.num_vars;
    function.num_vars += 1;
    lower_expr(expr, var_id_map, function, bb_id, result_var);
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
        Expr::IntegerLiteral { value: _ } => {}
        Expr::Add { lhs, rhs } => {
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

    fn assign_id(cctx: &mut CCtx, mut stmts: Vec<Stmt>) -> Vec<Stmt> {
        let mut scope = Scope::default();
        assign_id_stmts(cctx, &mut scope, &mut stmts);
        stmts
    }

    #[test]
    fn test_lower_add() {
        let mut cctx = CCtx::new();
        let s = assign_id(
            &mut cctx,
            vec![stmts::then_expr(exprs::add(
                exprs::integer_literal(1),
                exprs::integer_literal(2),
            ))],
        );
        let function = lower(&s);
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
        let s = assign_id(
            &mut cctx,
            vec![
                stmts::let_("x", exprs::integer_literal(42)),
                stmts::then_expr(exprs::var("x")),
            ],
        );
        let function = lower(&s);
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
        let s = assign_id(
            &mut cctx,
            vec![
                stmts::let_("x", exprs::integer_literal(42)),
                stmts::then_expr(exprs::branch(
                    exprs::var("x"),
                    exprs::integer_literal(1),
                    exprs::integer_literal(2),
                )),
            ],
        );
        let function = lower(&s);
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
}
