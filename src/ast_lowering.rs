use std::collections::{HashMap, HashSet};

use crate::ast::{BinOp, BuiltinIds, BuiltinKind, Expr, Stmt};
use crate::cctx::Id;
use crate::sir;

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

    let mut function =
        sir::Function::new(num_args, num_named_vars, vec![sir::BasicBlock::default()]);
    let mut fctx = FunctionContext {
        builtin_ids,
        function: &mut function,
        var_id_map: &var_id_map,
    };
    let result_var = fctx.fresh_var();
    lower_stmts(&mut fctx, stmts, result_var);
    fctx.push(sir::Inst::return_(result_var));
    function
}

#[derive(Debug)]
struct FunctionContext<'a> {
    builtin_ids: &'a BuiltinIds,
    function: &'a mut sir::Function,
    var_id_map: &'a HashMap<Id, usize>,
}

impl FunctionContext<'_> {
    fn fresh_var(&mut self) -> usize {
        let var = self.function.num_vars;
        self.function.num_vars += 1;
        var
    }
    fn new_bb(&mut self) -> usize {
        let bb_id = self.function.body.len();
        self.function.body.push(sir::BasicBlock::default());
        bb_id
    }
    fn current_bb_id(&self) -> usize {
        self.function.body.len() - 1
    }
    fn push_at(&mut self, bb_id: usize, inst: sir::Inst) {
        self.function.body[bb_id].insts.push(inst);
    }
    fn push(&mut self, inst: sir::Inst) {
        self.function.body.last_mut().unwrap().insts.push(inst);
    }
}

fn lower_stmts(fctx: &mut FunctionContext<'_>, stmts: &[Stmt], result_var: usize) {
    for (i, stmt) in stmts.iter().enumerate() {
        let is_last = i == stmts.len() - 1;
        let result_var = if is_last { Some(result_var) } else { None };
        lower_stmt(fctx, stmt, result_var);
    }
}

fn lower_stmt(fctx: &mut FunctionContext<'_>, stmt: &Stmt, result_var: Option<usize>) {
    match stmt {
        Stmt::Let { lhs, init } => {
            debug_assert!(!lhs.id.is_dummy());

            let var_id = fctx.var_id_map[&lhs.id];
            lower_expr(fctx, init, var_id);
            if let Some(result_var) = result_var {
                fctx.push(sir::Inst::literal(result_var, ()));
            }
        }
        Stmt::Expr { expr, use_value } => {
            debug_assert!(result_var.is_some() || !*use_value);
            let stmt_result_var = if *use_value {
                result_var.unwrap()
            } else {
                // Generate dummy temporary
                fctx.fresh_var()
            };
            lower_expr(fctx, expr, stmt_result_var);
            if result_var.is_some() && !*use_value {
                // Return unit instead
                fctx.push(sir::Inst::literal(result_var.unwrap(), ()));
            }
        }
    }
}

fn lower_expr(fctx: &mut FunctionContext<'_>, expr: &Expr, result_var: usize) {
    match expr {
        Expr::Var { ident } => {
            let builtin = fctx.builtin_ids.builtins.get(&ident.id).copied();
            if let Some(builtin) = builtin {
                fctx.push(sir::Inst::builtin(
                    result_var,
                    match builtin {
                        BuiltinKind::Puts => sir::BuiltinKind::Puts,
                        BuiltinKind::Puti => sir::BuiltinKind::Puti,
                    },
                ));
            } else {
                let var_id = fctx.var_id_map[&ident.id];
                fctx.push(sir::Inst::copy(result_var, var_id));
            }
        }
        Expr::Branch { cond, then, else_ } => {
            let cond_var = lower_expr2(fctx, cond);

            let branch_bb_id = fctx.current_bb_id();

            let then_bb_id = fctx.new_bb();
            lower_expr(fctx, then, result_var);

            let else_bb_id = fctx.new_bb();
            lower_expr(fctx, else_, result_var);

            let cont_bb_id = fctx.new_bb();

            fctx.push_at(
                branch_bb_id,
                sir::Inst::branch(cond_var, then_bb_id, else_bb_id),
            );
            fctx.push_at(then_bb_id, sir::Inst::jump(cont_bb_id));
            fctx.push_at(else_bb_id, sir::Inst::jump(cont_bb_id));
        }
        Expr::While { cond, body } => {
            let prev_bb_id = fctx.current_bb_id();

            let cond_bb_id = fctx.new_bb();
            let cond_var = lower_expr2(fctx, cond);

            let body_bb_id = fctx.new_bb();
            lower_expr(fctx, body, result_var);

            let cont_bb_id = fctx.new_bb();

            fctx.push_at(prev_bb_id, sir::Inst::jump(cond_bb_id));
            fctx.push_at(
                cond_bb_id,
                sir::Inst::branch(cond_var, body_bb_id, cont_bb_id),
            );
            fctx.push_at(body_bb_id, sir::Inst::jump(cond_bb_id));
            fctx.push(sir::Inst::literal(result_var, ()));
        }
        Expr::Block { stmts } => lower_stmts(fctx, stmts, result_var),
        Expr::Assign { lhs, rhs } => {
            debug_assert!(!lhs.id.is_dummy());
            let var_id = fctx.var_id_map[&lhs.id];
            lower_expr(fctx, rhs, var_id);
            fctx.push(sir::Inst::literal(result_var, ()));
        }
        Expr::Call { callee, args } => {
            let callee_var = lower_expr2(fctx, callee);
            let arg_vars = args
                .iter()
                .map(|arg| lower_expr2(fctx, arg))
                .collect::<Vec<_>>();
            for &arg_var in &arg_vars {
                fctx.push(sir::Inst::push_arg(arg_var));
            }
            fctx.push(sir::Inst::call(result_var, callee_var));
        }
        Expr::IntegerLiteral { value } => {
            fctx.push(sir::Inst::literal(result_var, *value));
        }
        Expr::StringLiteral { value } => {
            fctx.push(sir::Inst::literal(result_var, &**value));
        }
        Expr::BinOp { op, lhs, rhs } => {
            let callee_var = fctx.fresh_var();
            fctx.push(sir::Inst::builtin(
                callee_var,
                match op {
                    BinOp::Add => sir::BuiltinKind::Add,
                    BinOp::Lt => sir::BuiltinKind::Lt,
                },
            ));

            let lhs_var = lower_expr2(fctx, lhs);
            let rhs_var = lower_expr2(fctx, rhs);

            fctx.push(sir::Inst::push_arg(lhs_var));
            fctx.push(sir::Inst::push_arg(rhs_var));
            fctx.push(sir::Inst::call(result_var, callee_var));
        }
    }
}

fn lower_expr2(fctx: &mut FunctionContext<'_>, expr: &Expr) -> usize {
    let result_var = fctx.fresh_var();
    lower_expr(fctx, expr, result_var);
    result_var
}

fn collect_vars_stmts(stmts: &[Stmt], vars: &mut HashSet<Id>) {
    for stmt in stmts {
        collect_vars_stmt(stmt, vars);
    }
}

fn collect_vars_stmt(stmt: &Stmt, vars: &mut HashSet<Id>) {
    match stmt {
        Stmt::Let { lhs, init } => {
            debug_assert!(!lhs.id.is_dummy());
            vars.insert(lhs.id);
            collect_vars_expr(init, vars);
        }
        Stmt::Expr { expr, use_value: _ } => {
            collect_vars_expr(expr, vars);
        }
    }
}

fn collect_vars_expr(expr: &Expr, vars: &mut HashSet<Id>) {
    match expr {
        Expr::Var { ident } => {
            debug_assert!(!ident.id.is_dummy());
            vars.insert(ident.id);
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
        Expr::Assign { lhs, rhs } => {
            debug_assert!(!lhs.id.is_dummy());
            vars.insert(lhs.id);
            collect_vars_expr(rhs, vars);
        }
        Expr::Call { callee, args } => {
            collect_vars_expr(callee, vars);
            for arg in args {
                collect_vars_expr(arg, vars);
            }
        }
        Expr::IntegerLiteral { value: _ } => {}
        Expr::StringLiteral { value: _ } => {}
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
    use crate::sir::testing::FunctionTestingExt;
    use crate::sir::Inst;

    fn assign_id(cctx: &mut CCtx, builtin_ids: &BuiltinIds, mut stmts: Vec<Stmt>) -> Vec<Stmt> {
        let mut scope = Scope::new(builtin_ids);
        assign_id_stmts(cctx, &mut scope, &mut stmts);
        stmts
    }

    #[test]
    fn test_lower_hello() {
        let mut cctx = CCtx::new();
        let builtin_ids = BuiltinIds::new(&cctx);
        let s = assign_id(
            &mut cctx,
            &builtin_ids,
            vec![stmts::expr(exprs::call(
                exprs::var("puts"),
                vec![exprs::string_literal("Hello, world!")],
            ))],
        );
        let function = lower(&builtin_ids, &s);
        assert_eq!(
            function,
            sir::Function::describe(0, |desc, (_tmp1, tmp2, tmp3, puts1, tmp4), (entry,)| {
                desc.block(
                    entry,
                    vec![
                        Inst::builtin(puts1, sir::BuiltinKind::Puts),
                        Inst::literal(tmp4, "Hello, world!"),
                        Inst::push_arg(tmp4),
                        Inst::call(tmp3, puts1),
                        Inst::literal(tmp2, ()),
                        Inst::return_(tmp2),
                    ],
                );
            })
        );
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
            sir::Function::describe(0, |desc, (tmp1, add1, tmp2, tmp3), (entry,)| {
                desc.block(
                    entry,
                    vec![
                        Inst::builtin(add1, sir::BuiltinKind::Add),
                        Inst::literal(tmp2, 1),
                        Inst::literal(tmp3, 2),
                        Inst::push_arg(tmp2),
                        Inst::push_arg(tmp3),
                        Inst::call(tmp1, add1),
                        Inst::return_(tmp1),
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
                        Inst::literal(x, 42),
                        Inst::copy(tmp1, x),
                        Inst::return_(tmp1),
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
                            Inst::literal(x, 42),
                            Inst::copy(tmp2, x),
                            Inst::branch(tmp2, branch_then, branch_else),
                        ],
                    );
                    desc.block(branch_then, vec![Inst::literal(tmp1, 1), Inst::jump(cont)]);
                    desc.block(branch_else, vec![Inst::literal(tmp1, 2), Inst::jump(cont)]);
                    desc.block(cont, vec![Inst::return_(tmp1)]);
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
                |desc,
                 (x, tmp1, cond1, lt1, tmp2, tmp3, add1, tmp4, tmp5),
                 (entry, cond, body, cont)| {
                    desc.block(entry, vec![Inst::literal(x, 42), Inst::jump(cond)]);
                    desc.block(
                        cond,
                        vec![
                            Inst::builtin(lt1, sir::BuiltinKind::Lt),
                            Inst::literal(tmp2, -1),
                            Inst::copy(tmp3, x),
                            Inst::push_arg(tmp2),
                            Inst::push_arg(tmp3),
                            Inst::call(cond1, lt1),
                            Inst::branch(cond1, body, cont),
                        ],
                    );
                    desc.block(
                        body,
                        vec![
                            Inst::builtin(add1, sir::BuiltinKind::Add),
                            Inst::copy(tmp4, x),
                            Inst::literal(tmp5, -1),
                            Inst::push_arg(tmp4),
                            Inst::push_arg(tmp5),
                            Inst::call(x, add1),
                            Inst::literal(tmp1, ()),
                            Inst::jump(cond),
                        ],
                    );
                    desc.block(cont, vec![Inst::literal(tmp1, ()), Inst::return_(tmp1)]);
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
            sir::Function::describe(0, |desc, (_tmp1, tmp2, puti1, tmp3), (entry,)| {
                desc.block(
                    entry,
                    vec![
                        Inst::builtin(puti1, sir::BuiltinKind::Puti),
                        Inst::literal(tmp3, 42),
                        Inst::push_arg(tmp3),
                        Inst::call(tmp2, puti1),
                        Inst::return_(tmp2),
                    ],
                );
            })
        );
    }
}
