use std::{
    collections::{HashMap, HashSet},
    mem,
};

use crate::ast::Expr;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum CExpr {
    Let(Box<CExpr>, Box<CExpr>),
    Var(usize, /** movable? */ bool),
    Int(i32),
    Arr(Vec<CExpr>),
}

fn compile(e: &Expr) -> CExpr {
    let mut e = compile1(e, &mut Compile1Env::default());
    let mut used = UsedSet {
        parent: None,
        used: HashSet::new(),
    };
    compile2(&mut e, &mut Compile2Env::default(), &mut used);
    e
}

#[derive(Debug, Clone, Default)]
struct Compile1Env {
    index: usize,
    locals: HashMap<String, usize>,
}

fn compile1(e: &Expr, env: &mut Compile1Env) -> CExpr {
    match e {
        Expr::Let(name, init, cont) => {
            let init = compile1(init, env);
            let local_idx = env.index;
            env.index += 1;
            let old_binding = env.locals.insert(name.to_owned(), local_idx);
            let cont = compile1(cont, env);
            if let Some(old_binding) = old_binding {
                env.locals.insert(name.to_owned(), old_binding);
            } else {
                env.locals.remove(name);
            }
            env.index -= 1;
            CExpr::Let(Box::new(init), Box::new(cont))
        }
        Expr::Var(name) => {
            if let Some(&idx) = env.locals.get(name) {
                CExpr::Var(env.index - idx - 1, false)
            } else {
                panic!("Undefined variable: {}", name);
            }
        }
        Expr::Int(x) => CExpr::Int(*x),
        Expr::Arr(elems) => CExpr::Arr(
            elems
                .iter()
                .map(|elem| compile1(elem, env))
                .collect::<Vec<_>>(),
        ),
    }
}

#[derive(Debug, Clone, Default)]
struct Compile2Env {
    index: usize,
}

#[derive(Debug)]
struct UsedSet<'a> {
    parent: Option<&'a UsedSet<'a>>,
    used: HashSet<usize>,
}

impl UsedSet<'_> {
    fn has(&self, level: usize) -> bool {
        self.used.contains(&level) || self.parent.map(|parent| parent.has(level)).unwrap_or(false)
    }
}

fn compile2(e: &mut CExpr, env: &mut Compile2Env, used: &mut UsedSet<'_>) {
    match e {
        CExpr::Let(init, cont) => {
            env.index += 1;
            compile2(cont, env, used);
            env.index -= 1;
            compile2(init, env, used);
        }
        CExpr::Var(idx, movable) => {
            let level = env.index - *idx - 1;
            if !used.has(level) {
                used.used.insert(level);
                *movable = true;
            }
        }
        CExpr::Int(_) => {}
        CExpr::Arr(elems) => {
            for elem in elems {
                compile2(elem, env, used);
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Value {
    Int(i32),
    Arr(Vec<Value>),
}

#[derive(Debug, Clone, Default)]
struct Env {
    locals: Vec<Value>,
}

pub fn eval(e: &Expr) -> Value {
    let e = compile(e);
    eval_c(&e, &mut Env::default())
}

fn eval_c(e: &CExpr, env: &mut Env) -> Value {
    match e {
        CExpr::Let(init, cont) => {
            let init_val = eval_c(init, env);
            env.locals.push(init_val);
            let ret = eval_c(cont, env);
            env.locals.pop().unwrap();
            ret
        }
        CExpr::Var(idx, movable) => {
            let level = env.locals.len() - *idx - 1;
            if *movable {
                mem::replace(&mut env.locals[level], Value::Int(0))
            } else {
                env.locals[level].clone()
            }
        }
        CExpr::Int(x) => Value::Int(*x),
        CExpr::Arr(a) => Value::Arr(a.iter().map(|elem| eval_c(elem, env)).collect::<Vec<_>>()),
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
