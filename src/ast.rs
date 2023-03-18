#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Expr {
    Let(String, Box<Expr>, Box<Expr>),
    Var(String),
    Int(i32),
    Arr(Vec<Expr>),
}

pub mod expr {
    use super::*;
    pub fn let_(name: &str, init: Expr, cont: Expr) -> Expr {
        Expr::Let(name.to_owned(), Box::new(init), Box::new(cont))
    }
    pub fn var(name: &str) -> Expr {
        Expr::Var(name.to_owned())
    }
    pub fn int(x: i32) -> Expr {
        Expr::Int(x)
    }
    pub fn arr(a: &[Expr]) -> Expr {
        Expr::Arr(Vec::from(a))
    }
}
