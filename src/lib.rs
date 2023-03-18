use std::collections::HashMap;
use std::str;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Value {
    Int(i32),
    Arr(Vec<Value>),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Expr {
    Let(String, Box<Expr>, Box<Expr>),
    Var(String),
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
    pub fn int(x: i32) -> Expr {
        Expr::Int(x)
    }
    pub fn arr(a: &[Expr]) -> Expr {
        Expr::Arr(Vec::from(a))
    }
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

fn value_string(v: &Value) -> String {
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

pub fn exec(text: &str) -> String {
    value_string(&eval(&parse(text), &Env::default()))
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum Token {
    LBrack,
    RBrack,
    Comma,
    Equal,
    KeywordIn,
    KeywordLet,
    Ident(String),
    Int(i32),
}

fn tokenize(s: &[u8]) -> Vec<Token> {
    let mut i = 0;
    let mut tokens = Vec::new();
    loop {
        while i < s.len() && s[i].is_ascii_whitespace() {
            i += 1;
        }
        if i >= s.len() {
            break;
        }
        match s[i] {
            b'a'..=b'z' | b'A'..=b'Z' | b'_' => {
                let start = i;
                while i < s.len() && (s[i].is_ascii_alphanumeric() || s[i] == b'_') {
                    i += 1;
                }
                let ident = str::from_utf8(&s[start..i]).unwrap().to_owned();
                tokens.push(match ident.as_str() {
                    "in" => Token::KeywordIn,
                    "let" => Token::KeywordLet,
                    _ => Token::Ident(ident),
                })
            }
            b'0'..=b'9' => {
                let start = i;
                while i < s.len() && s[i].is_ascii_digit() {
                    i += 1;
                }
                tokens.push(Token::Int(
                    str::from_utf8(&s[start..i])
                        .unwrap()
                        .parse::<i32>()
                        .unwrap(),
                ));
            }
            b'[' => {
                i += 1;
                tokens.push(Token::LBrack);
            }
            b']' => {
                i += 1;
                tokens.push(Token::RBrack);
            }
            b',' => {
                i += 1;
                tokens.push(Token::Comma);
            }
            b'=' => {
                i += 1;
                tokens.push(Token::Equal);
            }
            _ => panic!("Invalid input: {:?}", s[i] as char),
        }
    }
    tokens
}

#[derive(Debug)]
struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    fn prog(&mut self) -> Expr {
        let e = self.expr();
        if self.pos < self.tokens.len() {
            panic!("Unexpected {:?} for EOF", self.tokens[self.pos]);
        }
        e
    }
    fn expr(&mut self) -> Expr {
        match self.tokens.get(self.pos) {
            Some(Token::KeywordLet) => {
                self.pos += 1;
                let name = if let Some(Token::Ident(name)) = self.tokens.get(self.pos) {
                    name.clone()
                } else {
                    panic!("Unexpected {:?} for Ident", self.tokens.get(self.pos));
                };
                self.pos += 1;
                if let Some(Token::Equal) = self.tokens.get(self.pos) {
                    // OK
                } else {
                    panic!("Unexpected {:?} for Equal", self.tokens.get(self.pos));
                }
                self.pos += 1;
                let init = self.expr();
                if let Some(Token::KeywordIn) = self.tokens.get(self.pos) {
                    // OK
                } else {
                    panic!("Unexpected {:?} for KeywordIn", self.tokens.get(self.pos));
                }
                self.pos += 1;
                let cont = self.expr();
                Expr::Let(name, Box::new(init), Box::new(cont))
            }
            Some(Token::Ident(name)) => {
                self.pos += 1;
                Expr::Var(name.to_owned())
            }
            Some(Token::Int(n)) => {
                self.pos += 1;
                Expr::Int(*n)
            }
            Some(Token::LBrack) => {
                self.pos += 1;
                let mut elems = Vec::new();
                loop {
                    if self.tokens.get(self.pos) == Some(&Token::RBrack) {
                        self.pos += 1;
                        break;
                    }
                    elems.push(self.expr());
                    if self.tokens.get(self.pos) == Some(&Token::RBrack) {
                        self.pos += 1;
                        break;
                    }
                    if self.tokens.get(self.pos) == Some(&Token::Comma) {
                        self.pos += 1;
                        continue;
                    } else {
                        panic!("Unexpected {:?} for Comma", self.tokens.get(self.pos));
                    }
                }
                Expr::Arr(elems)
            }
            Some(token) => panic!("Unexpected {:?} for expr", token),
            None => panic!("Unexpected EOF for expr"),
        }
    }
}

pub fn parse(text: &str) -> Expr {
    let tokens = tokenize(text.as_bytes());
    Parser { tokens, pos: 0 }.prog()
}

#[test]
fn test_lit() {
    assert_eq!(eval(&expr::int(42), &Env::default()), Value::Int(42));
    assert_eq!(
        eval(
            &expr::arr(&[
                expr::int(72),
                expr::int(101),
                expr::int(108),
                expr::int(108),
                expr::int(111),
            ]),
            &Env::default()
        ),
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
    assert_eq!(
        eval(
            &expr::let_(
                "foo",
                expr::int(42),
                expr::arr(&[expr::var("foo"), expr::int(50),]),
            ),
            &Env::default()
        ),
        Value::Arr(vec![Value::Int(42), Value::Int(50),])
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
