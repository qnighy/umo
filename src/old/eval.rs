use std::collections::{HashMap, HashSet};
use std::mem;

use crate::old::ast::Expr;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum CExpr {
    Let(Box<CExpr>, Box<CExpr>),
    Var(usize, /** movable? */ bool),
    Abs(usize, Box<CExpr>),
    Call(Box<CExpr>, Vec<CExpr>),
    Cond(
        /** cond */ Box<CExpr>,
        /** then */ Box<CExpr>,
        /** else */ Box<CExpr>,
    ),
    Int(i32),
    Arr(Vec<CExpr>),
    Builtin(BuiltinKind),
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
                let builtin = match name.as_str() {
                    "add" => BuiltinKind::Add,
                    "sub" => BuiltinKind::Sub,
                    "mul" => BuiltinKind::Mul,
                    "div" => BuiltinKind::Div,
                    "lt" => BuiltinKind::Lt,
                    "gt" => BuiltinKind::Gt,
                    "le" => BuiltinKind::Le,
                    "ge" => BuiltinKind::Ge,
                    "eq" => BuiltinKind::Eq,
                    "ne" => BuiltinKind::Ne,
                    "array_len" => BuiltinKind::ArrayLen,
                    "array_init" => BuiltinKind::ArrayInit,
                    "array_get" => BuiltinKind::ArrayGet,
                    "array_set" => BuiltinKind::ArraySet,
                    _ => panic!("Undefined variable: {}", name),
                };
                CExpr::Builtin(builtin)
            }
        }
        Expr::Abs(params, body) => {
            let mut stack = Vec::new();
            for name in params {
                let local_idx = env.index;
                env.index += 1;
                let old_binding = env.locals.insert(name.to_owned(), local_idx);
                stack.push(old_binding);
            }
            let body = compile1(body, env);
            for (old_binding, name) in stack.into_iter().zip(params).rev() {
                if let Some(old_binding) = old_binding {
                    env.locals.insert(name.to_owned(), old_binding);
                } else {
                    env.locals.remove(name);
                }
                env.index -= 1;
            }
            CExpr::Abs(params.len(), Box::new(body))
        }
        Expr::Call(callee, args) => {
            let callee = compile1(callee, env);
            let args = args
                .iter()
                .map(|arg| compile1(arg, env))
                .collect::<Vec<_>>();
            CExpr::Call(Box::new(callee), args)
        }
        Expr::Cond(cond, then, else_) => {
            let cond = compile1(cond, env);
            let then = compile1(then, env);
            let else_ = compile1(else_, env);
            CExpr::Cond(Box::new(cond), Box::new(then), Box::new(else_))
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
        CExpr::Abs(num_params, body) => {
            env.index += *num_params;
            compile2(body, env, used);
            env.index -= *num_params;
        }
        CExpr::Call(callee, args) => {
            for arg in args.iter_mut().rev() {
                compile2(arg, env, used);
            }
            compile2(callee, env, used);
        }
        CExpr::Cond(cond, then, else_) => {
            // NOTE: this analysis is suboptimal but its ok for now because we are going to rewrite the interpreter
            compile2(else_, env, used);
            compile2(then, env, used);
            compile2(cond, env, used);
        }
        CExpr::Int(_) => {}
        CExpr::Arr(elems) => {
            for elem in elems.iter_mut().rev() {
                compile2(elem, env, used);
            }
        }
        CExpr::Builtin(_) => {}
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Value {
    Invalid,
    Int(i32),
    Arr(Vec<Value>),
    Closure(
        /** captured stack */ Vec<Value>,
        /** num params */ usize,
        ClosureBody,
    ),
    Builtin(BuiltinKind),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ClosureBody(Box<CExpr>);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum BuiltinKind {
    Add,
    Sub,
    Mul,
    Div,
    Lt,
    Gt,
    Le,
    Ge,
    Eq,
    Ne,
    ArrayLen,
    ArrayInit,
    ArrayGet,
    ArraySet,
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
                mem::replace(&mut env.locals[level], Value::Invalid)
            } else {
                env.locals[level].clone()
            }
        }
        CExpr::Abs(num_params, body) => {
            Value::Closure(env.locals.clone(), *num_params, ClosureBody(body.clone()))
        }
        CExpr::Call(callee, args) => {
            let callee_val = eval_c(callee, env);
            let args_val = args.iter().map(|arg| eval_c(arg, env)).collect::<Vec<_>>();
            eval_call(callee_val, args_val)
        }
        CExpr::Cond(cond, then, else_) => {
            let cond = eval_c(cond, env);
            let Value::Int(cond) = cond else {
                panic!("Condition not an integer");
            };
            if cond != 0 {
                eval_c(then, env)
            } else {
                eval_c(else_, env)
            }
        }
        CExpr::Int(x) => Value::Int(*x),
        CExpr::Arr(a) => Value::Arr(a.iter().map(|elem| eval_c(elem, env)).collect::<Vec<_>>()),
        CExpr::Builtin(builtin) => Value::Builtin(*builtin),
    }
}

fn eval_call(callee: Value, args: Vec<Value>) -> Value {
    match callee {
        Value::Closure(mut captured_stack, num_params, ClosureBody(body)) => {
            if args.len() != num_params {
                panic!(
                    "Wrong number of arguments: got {}, but required {}",
                    args.len(),
                    num_params
                );
            }
            for arg_val in args {
                captured_stack.push(arg_val.clone())
            }
            eval_c(
                &body,
                &mut Env {
                    locals: captured_stack,
                },
            )
        }
        Value::Builtin(callee) => match callee {
            BuiltinKind::Add => {
                let [Value::Int(x), Value::Int(y)] = args[..] else {
                    panic!("Invalid arguments to add");
                };
                Value::Int(x + y)
            }
            BuiltinKind::Sub => {
                let [Value::Int(x), Value::Int(y)] = args[..] else {
                    panic!("Invalid arguments to sub");
                };
                Value::Int(x - y)
            }
            BuiltinKind::Mul => {
                let [Value::Int(x), Value::Int(y)] = args[..] else {
                    panic!("Invalid arguments to mul");
                };
                Value::Int(x * y)
            }
            BuiltinKind::Div => {
                let [Value::Int(x), Value::Int(y)] = args[..] else {
                    panic!("Invalid arguments to div");
                };
                Value::Int(x / y)
            }
            BuiltinKind::Lt => {
                let [Value::Int(x), Value::Int(y)] = args[..] else {
                    panic!("Invalid arguments to lt");
                };
                Value::Int((x < y) as i32)
            }
            BuiltinKind::Le => {
                let [Value::Int(x), Value::Int(y)] = args[..] else {
                    panic!("Invalid arguments to le");
                };
                Value::Int((x <= y) as i32)
            }
            BuiltinKind::Gt => {
                let [Value::Int(x), Value::Int(y)] = args[..] else {
                    panic!("Invalid arguments to gt");
                };
                Value::Int((x > y) as i32)
            }
            BuiltinKind::Ge => {
                let [Value::Int(x), Value::Int(y)] = args[..] else {
                    panic!("Invalid arguments to ge");
                };
                Value::Int((x >= y) as i32)
            }
            BuiltinKind::Eq => {
                let [Value::Int(x), Value::Int(y)] = args[..] else {
                    panic!("Invalid arguments to eq");
                };
                Value::Int((x == y) as i32)
            }
            BuiltinKind::Ne => {
                let [Value::Int(x), Value::Int(y)] = args[..] else {
                    panic!("Invalid arguments to ne");
                };
                Value::Int((x != y) as i32)
            }
            BuiltinKind::ArrayLen => {
                let [Value::Arr(ref arr)] = args[..] else {
                    panic!("Invalid arguments to array_len");
                };
                Value::Int(arr.len() as i32)
            }
            BuiltinKind::ArrayInit => {
                let [Value::Int(len), ref initializer] = args[..] else {
                    panic!("Invalid arguments to array_init");
                };
                Value::Arr(
                    (0..len)
                        .map(|i| eval_call(initializer.clone(), vec![Value::Int(i)]))
                        .collect::<Vec<_>>(),
                )
            }
            BuiltinKind::ArrayGet => {
                let [Value::Arr(ref arr), Value::Int(i)] = args[..] else {
                    panic!("Invalid arguments to array_get");
                };
                arr[i as usize].clone()
            }
            BuiltinKind::ArraySet => {
                if args.len() != 3 {
                    panic!("Invalid arguments to array_set");
                }
                let mut iter = args.into_iter();
                let Value::Arr(mut arr) = iter.next().unwrap() else {
                    panic!("Invalid arguments to array_set");
                };
                let Value::Int(i) = iter.next().unwrap() else {
                    panic!("Invalid arguments to array_set");
                };
                let v = iter.next().unwrap();
                arr[i as usize] = v;
                Value::Arr(arr)
            }
        },
        _ => panic!("Callee not a function"),
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
