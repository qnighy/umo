use std::str;

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
    value_string(&eval(&parse(text)))
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum Token {
    LBrack,
    RBrack,
    Comma,
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
    assert_eq!(exec("[72, 101, 108, 108, 111]"), "Hello");
}
