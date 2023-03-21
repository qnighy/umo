use std::str;

use crate::eval::{eval, value_string};
use crate::parsing::parse;

pub mod ast;
mod eval;
mod parsing;

pub fn exec(text: &str) -> String {
    value_string(&eval(&parse(text)))
}

#[test]
fn test_lit() {
    use crate::ast::expr;
    use crate::eval::Value;

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
fn test_let() {
    use crate::ast::expr;
    use crate::eval::Value;

    assert_eq!(
        eval(&expr::let_(
            "foo",
            expr::int(42),
            expr::arr(&[expr::var("foo"), expr::int(50),]),
        )),
        Value::Arr(vec![Value::Int(42), Value::Int(50),])
    );
}

#[test]
fn test_calc() {
    use crate::ast::expr;
    use crate::eval::Value;

    assert_eq!(
        eval(&expr::call(
            expr::var("add"),
            &[expr::int(20), expr::int(22)]
        )),
        Value::Int(42)
    );
}

#[test]
fn test_exec_lit() {
    assert_eq!(exec("[72, 101, 108, 108, 111]"), "Hello");
}

#[test]
fn test_exec_let() {
    assert_eq!(exec("let x = 72 in [x, 101, 108, 108, 111]"), "Hello");
}

#[test]
fn test_exec_calc() {
    assert_eq!(
        exec("let x = 42 in [add(x, 30), 101, 108, 108, 111]"),
        "Hello"
    );
}

#[test]
fn test_exec_lambda() {
    assert_eq!(
        exec(
            "
                let f = (x) => add(x, 30) in
                let g = (y) => f(y) in
                let x = 42 in
                [g(x), 101, 108, 108, 111]
            "
        ),
        "Hello"
    );
}
