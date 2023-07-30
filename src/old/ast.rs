#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Expr {
    Let(String, Box<Expr>, Box<Expr>),
    Var(String),
    Abs(Vec<String>, Box<Expr>),
    Call(Box<Expr>, Vec<Expr>),
    Cond(
        /** cond */ Box<Expr>,
        /** then */ Box<Expr>,
        /** else */ Box<Expr>,
    ),
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
    pub fn abs(params: &[&str], body: Expr) -> Expr {
        Expr::Abs(
            params.iter().map(|&s| s.to_owned()).collect::<Vec<_>>(),
            Box::new(body),
        )
    }
    pub fn call(callee: Expr, args: &[Expr]) -> Expr {
        Expr::Call(Box::new(callee), args.to_owned())
    }
    pub fn int(x: i32) -> Expr {
        Expr::Int(x)
    }
    pub fn arr(a: &[Expr]) -> Expr {
        Expr::Arr(Vec::from(a))
    }
}
