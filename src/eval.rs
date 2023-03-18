use std::collections::HashMap;

use crate::ast::Expr;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Value {
    Int(i32),
    Arr(Vec<Value>),
}

#[derive(Debug, Clone, Default)]
pub struct Env {
    locals: HashMap<String, Value>,
}

pub fn eval(e: &Expr, env: &Env) -> Value {
    match e {
        Expr::Let(name, init, cont) => {
            let init_val = eval(init, env);
            let mut new_env = env.clone();
            new_env.locals.insert(name.clone(), init_val);
            eval(cont, &new_env)
        }
        Expr::Var(name) => {
            if let Some(value) = env.locals.get(name) {
                value.clone()
            } else {
                panic!("Undefined variable: {}", name);
            }
        }
        Expr::Int(x) => Value::Int(*x),
        Expr::Arr(a) => Value::Arr(a.iter().map(|elem| eval(elem, env)).collect::<Vec<_>>()),
    }
}

pub fn value_string(v: &Value) -> String {
    let Value::Arr(v) = v else {
        panic!("Not a string: {:?}", v)
    };
    let v = v
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
        .unwrap_or_else(|| panic!("Not a string: {:?}", v));
    String::from_utf8_lossy(&v).into_owned()
}
