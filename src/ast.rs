use std::collections::HashMap;

use crate::cctx::{CCtx, Id};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Stmt {
    #[allow(unused)] // TODO: remove this annotation later
    Let { name: String, id: Id, init: Expr },
    #[allow(unused)] // TODO: remove this annotation later
    Expr { expr: Expr, use_value: bool },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Expr {
    #[allow(unused)] // TODO: remove this annotation later
    Var { name: String, id: Id },
    #[allow(unused)] // TODO: remove this annotation later
    Branch {
        cond: Box<Expr>,
        then: Box<Expr>,
        else_: Box<Expr>,
    },
    #[allow(unused)] // TODO: remove this annotation later
    While { cond: Box<Expr>, body: Box<Expr> },
    #[allow(unused)] // TODO: remove this annotation later
    Block { stmts: Vec<Stmt> },
    #[allow(unused)] // TODO: remove this annotation later
    Assign {
        name: String,
        id: Id,
        rhs: Box<Expr>,
    },
    #[allow(unused)] // TODO: remove this annotation later
    // TODO: use BigInt
    IntegerLiteral { value: i32 },
    #[allow(unused)] // TODO: remove this annotation later
    Add { lhs: Box<Expr>, rhs: Box<Expr> },
    #[allow(unused)] // TODO: remove this annotation later
    Lt { lhs: Box<Expr>, rhs: Box<Expr> },
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Scope {
    bindings: HashMap<String, Id>,
    binding_stack: Vec<(String, Option<Id>)>,
}

impl Scope {
    fn insert(&mut self, name: &str, id: Id) {
        self.binding_stack
            .push((name.to_owned(), self.bindings.insert(name.to_owned(), id)));
    }

    fn checkpoint(&self) -> usize {
        self.binding_stack.len()
    }

    fn rollback(&mut self, checkpoint: usize) {
        for (key, old_id) in self.binding_stack.drain(checkpoint..).rev() {
            if let Some(old_id) = old_id {
                self.bindings.insert(key, old_id);
            } else {
                self.bindings.remove(&key);
            }
        }
    }
}

#[allow(unused)] // TODO: remove this annotation later
pub fn assign_id_stmts(cctx: &CCtx, scope: &mut Scope, stmts: &mut Vec<Stmt>) {
    let checkpoint = scope.checkpoint();
    for stmt in stmts {
        assign_id_stmt(cctx, scope, stmt);
    }
    scope.rollback(checkpoint);
}

fn assign_id_stmt(cctx: &CCtx, scope: &mut Scope, stmt: &mut Stmt) {
    match stmt {
        Stmt::Let { name, id, init } => {
            assign_id_expr(cctx, scope, init);
            *id = cctx.id_gen.fresh();
            scope.insert(name, *id);
        }
        Stmt::Expr { expr, .. } => {
            assign_id_expr(cctx, scope, expr);
        }
    }
}

fn assign_id_expr(cctx: &CCtx, scope: &mut Scope, expr: &mut Expr) {
    match expr {
        Expr::Var { name, id } => {
            if let Some(&found_id) = scope.bindings.get(name) {
                *id = found_id;
            } else {
                // TODO: better error handling
                panic!("undefined variable: {}", name);
            }
        }
        Expr::Branch { cond, then, else_ } => {
            assign_id_expr(cctx, scope, cond);
            assign_id_expr(cctx, scope, then);
            assign_id_expr(cctx, scope, else_);
        }
        Expr::While { cond, body } => {
            assign_id_expr(cctx, scope, cond);
            assign_id_expr(cctx, scope, body);
        }
        Expr::Block { stmts } => {
            assign_id_stmts(cctx, scope, stmts);
        }
        Expr::Assign { name, id, rhs } => {
            assign_id_expr(cctx, scope, rhs);
            if let Some(&found_id) = scope.bindings.get(name) {
                *id = found_id;
            } else {
                // TODO: better error handling
                panic!("undefined variable: {}", name);
            }
        }
        Expr::IntegerLiteral { .. } => {}
        Expr::Add { lhs, rhs } => {
            assign_id_expr(cctx, scope, lhs);
            assign_id_expr(cctx, scope, rhs);
        }
        Expr::Lt { lhs, rhs } => {
            assign_id_expr(cctx, scope, lhs);
            assign_id_expr(cctx, scope, rhs);
        }
    }
}

#[cfg(test)]
pub mod testing {
    pub mod stmts {
        use super::super::*;

        pub fn let_(name: &str, init: Expr) -> Stmt {
            Stmt::Let {
                name: name.to_string(),
                id: Id::dummy(),
                init,
            }
        }

        #[allow(unused)] // TODO: remove this annotation later
        pub fn expr(expr: Expr) -> Stmt {
            Stmt::Expr {
                expr,
                use_value: false,
            }
        }

        pub fn then_expr(expr: Expr) -> Stmt {
            Stmt::Expr {
                expr,
                use_value: true,
            }
        }
    }
    pub mod exprs {
        use super::super::*;

        pub fn var(name: &str) -> Expr {
            Expr::Var {
                name: name.to_string(),
                id: Id::dummy(),
            }
        }

        pub fn branch(cond: Expr, then: Expr, else_: Expr) -> Expr {
            Expr::Branch {
                cond: Box::new(cond),
                then: Box::new(then),
                else_: Box::new(else_),
            }
        }

        pub fn while_(cond: Expr, body: Expr) -> Expr {
            Expr::While {
                cond: Box::new(cond),
                body: Box::new(body),
            }
        }

        pub fn block(stmts: Vec<Stmt>) -> Expr {
            Expr::Block { stmts }
        }

        pub fn assign(name: &str, rhs: Expr) -> Expr {
            Expr::Assign {
                name: name.to_string(),
                id: Id::dummy(),
                rhs: Box::new(rhs),
            }
        }

        pub fn integer_literal(value: i32) -> Expr {
            Expr::IntegerLiteral { value }
        }

        pub fn add(lhs: Expr, rhs: Expr) -> Expr {
            Expr::Add {
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            }
        }

        pub fn lt(lhs: Expr, rhs: Expr) -> Expr {
            Expr::Lt {
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            }
        }
    }
}
