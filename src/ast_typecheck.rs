use std::collections::HashMap;

use crate::ast::{Expr, Ident, Stmt};
use crate::cctx::Id;
use crate::ntype::{TyCtx, Type, UnificationFailure};

pub fn typecheck(program: &[Stmt], ty_ctx: &mut TyCtx) -> Result<(), UnificationFailure> {
    TypeChecker::new(ty_ctx).typecheck_program(program)
}

#[derive(Debug)]
struct TypeChecker<'a> {
    ty_ctx: &'a mut TyCtx,
    var_types: HashMap<Id, Type>,
}

impl<'a> TypeChecker<'a> {
    fn new(ty_ctx: &'a mut TyCtx) -> Self {
        Self {
            ty_ctx,
            var_types: HashMap::new(),
        }
    }
    fn typecheck_program(&mut self, program: &[Stmt]) -> Result<(), UnificationFailure> {
        let ty = self.typecheck_stmts(program)?;
        ty.unify(&Type::Unit, self.ty_ctx)?;
        Ok(())
    }
    fn typecheck_stmts(&mut self, stmts: &[Stmt]) -> Result<Type, UnificationFailure> {
        let mut final_type = Type::Unit;
        for stmt in stmts {
            final_type = self.typecheck_stmt(stmt)?;
        }
        Ok(final_type)
    }

    fn typecheck_stmt(&mut self, stmt: &Stmt) -> Result<Type, UnificationFailure> {
        match stmt {
            Stmt::Expr { expr, use_value } => {
                let ty = self.typecheck_expr(expr)?;
                if *use_value {
                    Ok(ty)
                } else {
                    Ok(Type::Unit)
                }
            }
            Stmt::Let { lhs, init } => {
                let init_ty = self.typecheck_expr(init)?;
                debug_assert!(!self.var_types.contains_key(&lhs.id));
                self.var_types.insert(lhs.id, init_ty);
                Ok(Type::Unit)
            }
        }
    }

    fn typecheck_expr(&mut self, expr: &Expr) -> Result<Type, UnificationFailure> {
        match expr {
            Expr::Var { ident } => {
                let ty = self.typecheck_ident(ident)?;
                Ok(ty)
            }
            Expr::Branch { cond, then, else_ } => {
                let cond_ty = self.typecheck_expr(cond)?;
                cond_ty.unify(&Type::Bool, self.ty_ctx)?;
                let then_ty = self.typecheck_expr(then)?;
                let else_ty = self.typecheck_expr(else_)?;
                then_ty.unify(&else_ty, self.ty_ctx)?;
                Ok(then_ty)
            }
            Expr::While { cond, body } => {
                let cond_ty = self.typecheck_expr(cond)?;
                cond_ty.unify(&Type::Bool, self.ty_ctx)?;
                let body_ty = self.typecheck_expr(body)?;
                body_ty.unify(&Type::Unit, self.ty_ctx)?;
                Ok(Type::Unit)
            }
            Expr::Block { stmts } => self.typecheck_stmts(stmts),
            Expr::Assign { lhs, rhs } => {
                let lhs_ty = self.typecheck_ident(lhs)?;
                let rhs_ty = self.typecheck_expr(rhs)?;
                lhs_ty.unify(&rhs_ty, self.ty_ctx)?;
                Ok(Type::Unit)
            }
            Expr::Call { callee, args } => {
                let callee_ty = self.typecheck_expr(callee)?;
                let mut arg_tys = Vec::new();
                for arg in args {
                    let arg_ty = self.typecheck_expr(arg)?;
                    arg_tys.push(arg_ty);
                }
                let ret_ty = Type::fresh(&mut self.ty_ctx);
                let func_ty = Type::function(arg_tys, ret_ty.clone());
                callee_ty.unify(&func_ty, self.ty_ctx)?;
                Ok(ret_ty)
            }
            Expr::IntegerLiteral { value: _ } => Ok(Type::Integer),
            Expr::StringLiteral { value: _ } => Ok(Type::String),
            Expr::BinOp { op, lhs, rhs } => {
                let op_ty = match op {
                    crate::ast::BinOp::Add => {
                        Type::function(vec![Type::Integer, Type::Integer], Type::Integer)
                    }
                    crate::ast::BinOp::Lt => {
                        Type::function(vec![Type::Integer, Type::Integer], Type::Bool)
                    }
                };
                let lhs_ty = self.typecheck_expr(lhs)?;
                let rhs_ty = self.typecheck_expr(rhs)?;
                let ret_ty = Type::fresh(&mut self.ty_ctx);
                op_ty.unify(
                    &Type::function(vec![lhs_ty, rhs_ty], ret_ty.clone()),
                    self.ty_ctx,
                )?;
                Ok(ret_ty)
            }
        }
    }

    fn typecheck_ident(&mut self, ident: &Ident) -> Result<Type, UnificationFailure> {
        debug_assert!(!ident.id.is_dummy());
        let ty = self.var_types.get(&ident.id).unwrap();
        Ok(ty.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::ast::{assign_id_stmt, BinOp, BuiltinIds, Expr, Scope, Stmt};
    use crate::cctx::CCtx;
    use crate::ntype::Type;

    fn with_typechecker<R, F>(f: F) -> R
    where
        F: FnOnce(&CCtx, &mut Scope, &mut TypeChecker) -> R,
    {
        let cctx = CCtx::new();
        let builtin_ids = BuiltinIds::new(&cctx);
        let mut scope = Scope::new(&builtin_ids);
        let mut ty_ctx = TyCtx::default();
        let mut typechecker = TypeChecker::new(&mut ty_ctx);
        f(&cctx, &mut scope, &mut typechecker)
    }

    #[test]
    fn test_typecheck_stmt() {
        with_typechecker(|cctx, scope, typechecker| {
            let mut stmt = Stmt::expr(
                Expr::bin_op(
                    BinOp::Add,
                    Expr::integer_literal(1),
                    Expr::integer_literal(2),
                ),
                false,
            );
            assign_id_stmt(cctx, scope, &mut stmt);
            let ty = typechecker.typecheck_stmt(&stmt).unwrap();
            assert_eq!(ty, Type::Unit);
        });
    }

    #[test]
    fn test_typecheck_stmts() {
        with_typechecker(|cctx, scope, typechecker| {
            let mut stmts = vec![
                Stmt::expr(
                    Expr::bin_op(
                        BinOp::Add,
                        Expr::integer_literal(1),
                        Expr::integer_literal(2),
                    ),
                    false,
                ),
                Stmt::expr(
                    Expr::bin_op(
                        BinOp::Add,
                        Expr::integer_literal(1),
                        Expr::integer_literal(2),
                    ),
                    false,
                ),
            ];
            assign_id_stmt(cctx, scope, &mut stmts[0]);
            assign_id_stmt(cctx, scope, &mut stmts[1]);
            let ty = typechecker.typecheck_stmts(&stmts).unwrap();
            assert_eq!(ty, Type::Unit);
        });
    }
}
