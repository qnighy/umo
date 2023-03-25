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

#[test]
fn test_exec_cond() {
    assert_eq!(
        exec(
            "
                let y = (f) =>
                    ((g) => f((x) => g(g)(x)))
                    ((g) => f((x) => g(g)(x))) in
                let fib = y((fib) => (n) =>
                    if le(n, 1) {
                        n
                    } else {
                        add(fib(sub(n, 1)), fib(sub(n, 2)))
                    }
                ) in
                let x = fib(11) in
                [x]
            "
        ),
        "Y" // 89
    );
}

#[test]
fn test_exec_arr() {
    assert_eq!(
        exec(
            "
                let y2 = (f) =>
                    ((g) => f((x, y) => g(g)(x, y)))
                    ((g) => f((x, y) => g(g)(x, y))) in
                let incr = (a) =>
                    let incr = y2((incr) => (a, i) =>
                        if lt(i, array_len(a)) {
                            incr(array_set(a, i, add(array_get(a, i), 1)), add(i, 1))
                        } else {
                            a
                        }
                    ) in
                    incr(a, 0)
                in
                incr([71, 100, 107, 107, 110])
            "
        ),
        "Hello"
    );
}
