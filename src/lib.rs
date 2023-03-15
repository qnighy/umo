#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Value {
    Int(i32),
    Arr(Vec<Value>),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Expr {
    Int(i32),
    Arr(Vec<Expr>),
}

pub mod expr {
    use super::*;
    pub fn int(x: i32) -> Expr {
        Expr::Int(x)
    }
    pub fn arr(a: &[Expr]) -> Expr {
        Expr::Arr(Vec::from(a))
    }
}

pub fn eval(e: &Expr) -> Value {
    match e {
        Expr::Int(x) => Value::Int(*x),
        Expr::Arr(a) => Value::Arr(a.iter().map(|elem| eval(elem)).collect::<Vec<_>>()),
    }
}

#[test]
fn test_lit() {
    assert_eq!(eval(&expr::int(42)), Value::Int(42));
    assert_eq!(
        eval(&expr::arr(&[
            expr::int(71),
            expr::int(101),
            expr::int(108),
            expr::int(108),
            expr::int(111),
        ])),
        Value::Arr(vec![
            Value::Int(71),
            Value::Int(101),
            Value::Int(108),
            Value::Int(108),
            Value::Int(111),
        ])
    );
}
