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

pub fn exec(e: &Expr) -> String {
    let result = eval(e);
    let Value::Arr(result) = result else {
        panic!("Not a string: {:?}", result)
    };
    let result = result
        .iter()
        .map(|elem| {
            let Value::Int(elem) = elem else {
                return None;
            };
            if !(0..=255).contains(elem) {
                return None;
            }
            Some(*elem as u8)
        })
        .collect::<Option<Vec<_>>>()
        .unwrap_or_else(|| panic!("Not a string: {:?}", result));
    String::from_utf8_lossy(&result).into_owned()
}

#[test]
fn test_lit() {
    assert_eq!(eval(&expr::int(42)), Value::Int(42));
    assert_eq!(
        eval(&expr::arr(&[
            expr::int(72),
            expr::int(101),
            expr::int(108),
            expr::int(108),
            expr::int(111),
        ])),
        Value::Arr(vec![
            Value::Int(72),
            Value::Int(101),
            Value::Int(108),
            Value::Int(108),
            Value::Int(111),
        ])
    );
}

#[test]
fn test_exec_lit() {
    assert_eq!(
        exec(&expr::arr(&[
            expr::int(72),
            expr::int(101),
            expr::int(108),
            expr::int(108),
            expr::int(111),
        ])),
        "Hello"
    );
}
