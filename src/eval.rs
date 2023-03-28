use std::{
    collections::{HashMap, HashSet},
    mem,
};

use crate::ast::Expr;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum Expr2 {
    Let(usize, Box<Expr2>, Box<Expr2>),
    Var(usize),
    Abs(
        /** params */ Vec<usize>,
        Box<Expr2>,
        /** captures */ Vec<usize>,
    ),
    Call(Box<Expr2>, Vec<Expr2>),
    Cond(
        /** cond */ Box<Expr2>,
        /** then */ Box<Expr2>,
        /** else */ Box<Expr2>,
    ),
    Int(i32),
    Arr(Vec<Expr2>),
    Builtin(BuiltinKind),
}

#[derive(Debug, Clone, Default)]
struct Ctx2 {
    next_local: usize,
    locals: HashMap<String, usize>,
}

impl Ctx2 {
    fn fresh_local(&mut self) -> usize {
        let l = self.next_local;
        self.next_local += 1;
        l
    }
    fn convert(&mut self, e: &Expr, captures: &mut HashSet<usize>) -> Expr2 {
        match e {
            Expr::Let(name, init, cont) => {
                let id = self.fresh_local();
                let init = self.convert(init, captures);
                let old_binding = self.locals.insert(name.to_owned(), id);
                let cont = self.convert(cont, captures);
                if let Some(old_binding) = old_binding {
                    self.locals.insert(name.to_owned(), old_binding);
                } else {
                    self.locals.remove(name);
                }
                Expr2::Let(id, Box::new(init), Box::new(cont))
            }
            Expr::Var(name) => {
                if let Some(&id) = self.locals.get(name) {
                    captures.insert(id);
                    Expr2::Var(id)
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
                    Expr2::Builtin(builtin)
                }
            }
            Expr::Abs(params, body) => {
                let mut inner_captures = HashSet::new();
                let mut param_ids = Vec::new();
                let mut stack = Vec::new();
                for name in params {
                    let id = self.fresh_local();
                    param_ids.push(id);
                    let old_binding = self.locals.insert(name.to_owned(), id);
                    stack.push(old_binding);
                }
                let body = self.convert(body, &mut inner_captures);
                for (old_binding, name) in stack.into_iter().zip(params).rev() {
                    if let Some(old_binding) = old_binding {
                        self.locals.insert(name.to_owned(), old_binding);
                    } else {
                        self.locals.remove(name);
                    }
                }
                for &param_id in &param_ids {
                    inner_captures.remove(&param_id);
                }
                for &c in &inner_captures {
                    captures.insert(c);
                }
                let mut inner_captures = inner_captures.into_iter().collect::<Vec<_>>();
                inner_captures.sort();
                Expr2::Abs(param_ids, Box::new(body), inner_captures)
            }
            Expr::Call(callee, args) => {
                let callee = self.convert(callee, captures);
                let args = args
                    .iter()
                    .map(|arg| self.convert(arg, captures))
                    .collect::<Vec<_>>();
                Expr2::Call(Box::new(callee), args)
            }
            Expr::Cond(cond, then, else_) => {
                let cond = self.convert(cond, captures);
                let then = self.convert(then, captures);
                let else_ = self.convert(else_, captures);
                Expr2::Cond(Box::new(cond), Box::new(then), Box::new(else_))
            }
            Expr::Int(x) => Expr2::Int(*x),
            Expr::Arr(elems) => Expr2::Arr(
                elems
                    .iter()
                    .map(|elem| self.convert(elem, captures))
                    .collect::<Vec<_>>(),
            ),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct FunDef {
    num_args: usize,
    num_locals: usize,
    body: Vec<BasicBlock>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct BasicBlock {
    middle: Vec<MInst>,
    tail: TInst,
}

/// Middle instruction
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum MInst {
    Read(usize),
    Write(usize),
    CCall(/** num_args */ usize),
    Closure(/** num_capture */ usize, /** function id */ usize),
    Int(i32),
    Arr(/** len */ usize),
    Builtin(BuiltinKind),
}

/// Tail instruction
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum TInst {
    Ret,
    Jump(/** target */ usize),
    JumpIf(/** then-target */ usize, /** else-target */ usize),
    TCCall,
}

#[derive(Debug)]
struct Ctx3 {
    functions: Vec<FunDef>,
}

#[derive(Debug)]
struct Ctx3F<'a> {
    base: &'a mut Ctx3,
    current_function: FunDef,
    current_function_idx: usize,
    local_map: HashMap<usize, usize>,
}

#[derive(Debug)]
struct Ctx3B<'a> {
    func: &'a mut Ctx3F<'a>,
    current_block: Vec<MInst>,
    current_block_idx: usize,
}

impl Ctx3B<'_> {
    fn convert(&mut self, e: &Expr2) {
        match e {
            Expr2::Let(id, init, cont) => {
                let local_index = *self.func.local_map.get(id).unwrap();
                self.convert(init);
                self.current_block.push(MInst::Write(local_index));
                self.convert(cont);
            }
            Expr2::Var(id) => {
                let local_index = *self.func.local_map.get(id).unwrap();
                self.current_block.push(MInst::Read(local_index));
            }
            Expr2::Abs(params, body, captures) => {
                let (num_locals, local_map) = Self::map_locals(body);
                let current_function_idx = self.func.base.functions.len();
                self.func.base.functions.push(FunDef {
                    num_args: 0,
                    num_locals: 0,
                    body: Vec::new(),
                });
                let mut func = Ctx3F {
                    base: &mut *self.func.base,
                    current_function: FunDef {
                        num_args: params.len(),
                        num_locals,
                        body: vec![BasicBlock {
                            middle: Vec::new(),
                            tail: TInst::Ret,
                        }],
                    },
                    current_function_idx,
                    local_map,
                };
                Ctx3B {
                    func: &mut func,
                    current_block: Vec::new(),
                    current_block_idx: 0,
                }
                .convert(body);
                todo!();
            }
            Expr2::Call(callee, args) => {
                self.convert(callee);
                for arg in args {
                    self.convert(arg);
                }
                self.current_block.push(MInst::CCall(args.len()));
            }
            Expr2::Cond(cond, then, else_) => {
                self.convert(cond);
                let start_block = self.steal_block();
                self.fresh_block();
                self.convert(then);
                let then_block = self.steal_block();
                self.fresh_block();
                self.convert(else_);
                let else_block = self.steal_block();
                self.fresh_block();

                self.finish_block(start_block, TInst::JumpIf(then_block.0, else_block.0));
                self.finish_block(then_block, TInst::Jump(self.current_block_idx));
                self.finish_block(else_block, TInst::Jump(self.current_block_idx));
            }
            Expr2::Int(n) => {
                self.current_block.push(MInst::Int(*n));
            }
            Expr2::Arr(elems) => {
                for elem in elems {
                    self.convert(elem);
                }
                self.current_block.push(MInst::Arr(elems.len()));
            }
            Expr2::Builtin(builtin) => {
                todo!()
            }
        }
    }

    fn fresh_block(&mut self) {
        self.current_block_idx = self.func.current_function.body.len();
        // Insert sentinel
        self.func.current_function.body.push(BasicBlock {
            middle: Vec::new(),
            tail: TInst::Ret,
        });
    }
    fn steal_block(&mut self) -> (usize, Vec<MInst>) {
        let idx = mem::replace(&mut self.current_block_idx, usize::MAX);
        let block = mem::replace(&mut self.current_block, Vec::new());
        (idx, block)
    }
    fn finish_block(&mut self, block: (usize, Vec<MInst>), tail: TInst) {
        self.func.current_function.body[block.0] = BasicBlock {
            middle: block.1,
            tail,
        };
    }

    fn map_locals(e: &Expr2) -> (usize, HashMap<usize, usize>) {
        let mut num_locals = 0;
        let mut local_map = HashMap::new();
        Self::collect_locals(e, &mut num_locals, &mut local_map);
        (num_locals, local_map)
    }

    fn collect_locals(e: &Expr2, num_locals: &mut usize, local_map: &mut HashMap<usize, usize>) {
        match e {
            Expr2::Let(id, init, cont) => {
                local_map.entry(*id).or_insert_with(|| {
                    let local_id = *num_locals;
                    *num_locals += 1;
                    local_id
                });
                Self::collect_locals(init, num_locals, local_map);
                Self::collect_locals(cont, num_locals, local_map);
            }
            Expr2::Var(_) => { /* do nothing */ }
            Expr2::Abs(_, _, _) => { /* do nothing */ }
            Expr2::Call(callee, args) => {
                Self::collect_locals(callee, num_locals, local_map);
                for arg in args {
                    Self::collect_locals(arg, num_locals, local_map);
                }
            }
            Expr2::Cond(cond, then, else_) => {
                Self::collect_locals(cond, num_locals, local_map);
                Self::collect_locals(then, num_locals, local_map);
                Self::collect_locals(else_, num_locals, local_map);
            }
            Expr2::Int(_) => {}
            Expr2::Arr(elems) => {
                for elem in elems {
                    Self::collect_locals(elem, num_locals, local_map);
                }
            }
            Expr2::Builtin(_) => todo!(),
        }
    }
}

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
