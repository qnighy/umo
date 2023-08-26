use std::collections::HashMap;
use std::fmt;

use crate::cctx::{CCtx, Id};

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Ident {
    pub name: String,
    pub id: Id,
}

impl Ident {
    pub fn with_id(self, id: Id) -> Self {
        Ident { id, ..self }
    }
}

impl fmt::Debug for Ident {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.id.is_dummy() {
            f.debug_tuple("Ident::from").field(&self.name).finish()
        } else {
            f.debug_tuple("Ident::from").field(&self.name).finish()?;
            f.debug_tuple(".with_id").field(&self.id).finish()?;
            Ok(())
        }
    }
}

impl From<&str> for Ident {
    fn from(name: &str) -> Self {
        Ident {
            name: name.to_owned(),
            id: Id::dummy(),
        }
    }
}
impl From<String> for Ident {
    fn from(name: String) -> Self {
        Ident {
            name,
            id: Id::dummy(),
        }
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum Stmt {
    Let { lhs: Ident, init: Expr },
    Expr { expr: Expr, use_value: bool },
}

impl Stmt {
    pub fn let_(lhs: Ident, init: Expr) -> Self {
        Stmt::Let { lhs, init }
    }
    pub fn expr(expr: Expr, use_value: bool) -> Self {
        Stmt::Expr { expr, use_value }
    }
}

impl fmt::Debug for Stmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Stmt::Let { lhs, init } => f.debug_tuple("Stmt:let_").field(lhs).field(init).finish(),
            Stmt::Expr { expr, use_value } => f
                .debug_tuple("Stmt::expr")
                .field(expr)
                .field(use_value)
                .finish(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Expr {
    Var {
        ident: Ident,
    },
    Branch {
        cond: Box<Expr>,
        then: Box<Expr>,
        else_: Box<Expr>,
    },
    While {
        cond: Box<Expr>,
        body: Box<Expr>,
    },
    Block {
        stmts: Vec<Stmt>,
    },
    Assign {
        lhs: Ident,
        rhs: Box<Expr>,
    },
    Call {
        callee: Box<Expr>,
        args: Vec<Expr>,
    },
    // TODO: use BigInt
    IntegerLiteral {
        value: i32,
    },
    StringLiteral {
        value: String,
    },
    BinOp {
        op: BinOp,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BinOp {
    Add,
    Lt,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BuiltinIds {
    pub ids: HashMap<BuiltinKind, Id>,
    pub builtins: HashMap<Id, BuiltinKind>,
}

impl BuiltinIds {
    pub fn new(cctx: &CCtx) -> Self {
        let mut builtin_ids = BuiltinIds::default();
        for builtin_kind in BuiltinKind::iter() {
            let id = cctx.id_gen.fresh();
            builtin_ids.ids.insert(builtin_kind, id);
            builtin_ids.builtins.insert(id, builtin_kind);
        }
        builtin_ids
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BuiltinKind {
    Puts,
    Puti,
}

impl BuiltinKind {
    fn name(self) -> &'static str {
        match self {
            BuiltinKind::Puts => "puts",
            BuiltinKind::Puti => "puti",
        }
    }
    fn iter() -> impl Iterator<Item = Self> {
        static BUILTIN_KINDS: &[BuiltinKind] = &[BuiltinKind::Puts, BuiltinKind::Puti];
        BUILTIN_KINDS.iter().copied()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Scope {
    bindings: HashMap<String, Id>,
    binding_stack: Vec<(String, Option<Id>)>,
}

impl Scope {
    pub fn new(builtin_ids: &BuiltinIds) -> Self {
        let mut scope = Scope {
            bindings: HashMap::default(),
            binding_stack: vec![],
        };
        for (builtin_id, builtin_kind) in &builtin_ids.builtins {
            scope.insert(builtin_kind.name(), *builtin_id);
        }
        scope
    }
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

pub fn assign_id_stmts(cctx: &CCtx, scope: &mut Scope, stmts: &mut Vec<Stmt>) {
    let checkpoint = scope.checkpoint();
    for stmt in stmts {
        assign_id_stmt(cctx, scope, stmt);
    }
    scope.rollback(checkpoint);
}

fn assign_id_stmt(cctx: &CCtx, scope: &mut Scope, stmt: &mut Stmt) {
    match stmt {
        Stmt::Let { lhs, init } => {
            assign_id_expr(cctx, scope, init);
            lhs.id = cctx.id_gen.fresh();
            scope.insert(&lhs.name, lhs.id);
        }
        Stmt::Expr { expr, .. } => {
            assign_id_expr(cctx, scope, expr);
        }
    }
}

fn assign_id_expr(cctx: &CCtx, scope: &mut Scope, expr: &mut Expr) {
    match expr {
        Expr::Var { ident } => {
            if let Some(&found_id) = scope.bindings.get(&ident.name) {
                ident.id = found_id;
            } else {
                // TODO: better error handling
                panic!("undefined variable: {}", ident.name);
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
        Expr::Assign { lhs, rhs } => {
            assign_id_expr(cctx, scope, rhs);
            if let Some(&found_id) = scope.bindings.get(&lhs.name) {
                lhs.id = found_id;
            } else {
                // TODO: better error handling
                panic!("undefined variable: {}", lhs.name);
            }
        }
        Expr::Call { callee, args } => {
            assign_id_expr(cctx, scope, callee);
            for arg in args {
                assign_id_expr(cctx, scope, arg);
            }
        }
        Expr::IntegerLiteral { .. } => {}
        Expr::StringLiteral { .. } => {}
        Expr::BinOp { op: _, lhs, rhs } => {
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
                lhs: Ident::from(name),
                init,
            }
        }

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
                ident: Ident::from(name),
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
                lhs: Ident::from(name),
                rhs: Box::new(rhs),
            }
        }

        pub fn call(callee: Expr, args: Vec<Expr>) -> Expr {
            Expr::Call {
                callee: Box::new(callee),
                args,
            }
        }

        pub fn integer_literal(value: i32) -> Expr {
            Expr::IntegerLiteral { value }
        }

        pub fn string_literal(value: &str) -> Expr {
            Expr::StringLiteral {
                value: value.to_owned(),
            }
        }

        pub fn add(lhs: Expr, rhs: Expr) -> Expr {
            Expr::BinOp {
                op: BinOp::Add,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            }
        }

        pub fn lt(lhs: Expr, rhs: Expr) -> Expr {
            Expr::BinOp {
                op: BinOp::Lt,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            }
        }
    }
}
