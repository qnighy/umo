use std::str;

use crate::ast::Expr;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum Token {
    LParen,
    RParen,
    LBrack,
    RBrack,
    Comma,
    Equal,
    FatArrow,
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
            b'(' => {
                i += 1;
                tokens.push(Token::LParen);
            }
            b')' => {
                i += 1;
                tokens.push(Token::RParen);
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
                if s.get(i) == Some(&b'>') {
                    i += 1;
                    tokens.push(Token::FatArrow);
                } else {
                    tokens.push(Token::Equal);
                }
            }
            _ => panic!("Invalid input: {:?}", s[i] as char),
        }
    }
    tokens
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum PExpr {
    Expr(Expr),
    AmbiguousParen(Vec<Expr>, /** trailing_comma */ bool),
}

impl From<Expr> for PExpr {
    fn from(e: Expr) -> Self {
        PExpr::Expr(e)
    }
}

impl PExpr {
    fn crush(self) -> Expr {
        match self {
            PExpr::AmbiguousParen(elems, trailing_comma) => {
                if !trailing_comma && elems.len() == 1 {
                    { elems }.pop().unwrap()
                } else {
                    todo!("tuple expression");
                }
            }
            PExpr::Expr(e) => e,
        }
    }
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
        let lhs = self.expr_call();
        if matches!(lhs, PExpr::AmbiguousParen(..)) {
            if self.tokens.get(self.pos) == Some(&Token::FatArrow) {
                self.pos += 1;
                let PExpr::AmbiguousParen(elems, _) = lhs else {
                    unreachable!();
                };
                let arrow_head = self.reparse_paren(elems);
                let body = self.expr();
                return Expr::Abs(arrow_head, Box::new(body));
            }
        }
        lhs.crush()
    }
    fn expr_call(&mut self) -> PExpr {
        let mut lhs = self.expr_primary();
        loop {
            match self.tokens.get(self.pos) {
                Some(Token::LParen) => {
                    self.pos += 1;
                    let mut elems = Vec::new();
                    loop {
                        if self.tokens.get(self.pos) == Some(&Token::RParen) {
                            self.pos += 1;
                            break;
                        }
                        elems.push(self.expr());
                        if self.tokens.get(self.pos) == Some(&Token::RParen) {
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
                    lhs = Expr::Call(Box::new(lhs.crush()), elems).into();
                }
                _ => break,
            }
        }
        lhs
    }
    fn expr_primary(&mut self) -> PExpr {
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
                Expr::Let(name, Box::new(init), Box::new(cont)).into()
            }
            Some(Token::Ident(name)) => {
                self.pos += 1;
                Expr::Var(name.to_owned()).into()
            }
            Some(Token::Int(n)) => {
                self.pos += 1;
                Expr::Int(*n).into()
            }
            Some(Token::LParen) => {
                self.pos += 1;
                let mut elems = Vec::new();
                let trailing_comma = loop {
                    if self.tokens.get(self.pos) == Some(&Token::RParen) {
                        self.pos += 1;
                        break true;
                    }
                    elems.push(self.expr());
                    if self.tokens.get(self.pos) == Some(&Token::RParen) {
                        self.pos += 1;
                        break false;
                    }
                    if self.tokens.get(self.pos) == Some(&Token::Comma) {
                        self.pos += 1;
                        continue;
                    } else {
                        panic!("Unexpected {:?} for Comma", self.tokens.get(self.pos));
                    }
                };
                PExpr::AmbiguousParen(elems, trailing_comma)
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
                Expr::Arr(elems).into()
            }
            Some(token) => panic!("Unexpected {:?} for expr", token),
            None => panic!("Unexpected EOF for expr"),
        }
    }
    fn reparse_paren(&mut self, elems: Vec<Expr>) -> Vec<String> {
        elems
            .iter()
            .map(|elem| {
                if let Expr::Var(name) = elem {
                    name.clone()
                } else {
                    panic!("Unexpected expression in arrow head");
                }
            })
            .collect::<Vec<_>>()
    }
}

pub fn parse(text: &str) -> Expr {
    let tokens = tokenize(text.as_bytes());
    Parser { tokens, pos: 0 }.prog()
}
