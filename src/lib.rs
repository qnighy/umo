#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Expr {
    Lit(i32),
}

pub fn eval(e: &Expr) -> i32 {
    match e {
        Expr::Lit(x) => *x,
    }
}

#[test]
fn test_lit() {
    assert_eq!(eval(&Expr::Lit(42)), 42);
}
