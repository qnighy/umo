use crate::ast::{BinOp, Expr, Ident, Stmt};

#[derive(Debug)]
pub struct ParseError;

pub fn parse(source: &str) -> Result<Vec<Stmt>, ParseError> {
    let mut parser = Parser::new(source);
    parser.parse_program()
}

#[derive(Debug)]
struct Parser {
    buf: Vec<u8>,
    pos: usize,
    next_token_cache: Option<Token>,
}

impl Parser {
    fn new(source: &str) -> Self {
        Self {
            buf: source.as_bytes().to_vec(),
            pos: 0,
            next_token_cache: None,
        }
    }
    fn parse_program(&mut self) -> Result<Vec<Stmt>, ParseError> {
        // Preliminary preamble
        if self.buf[self.pos..].starts_with(b"use lang::\"0.0.1\";\n") {
            self.pos += b"use lang::\"0.0.1\";\n".len();
        } else {
            return Err(ParseError);
        }
        let stmts = self.parse_stmts()?;
        self.expect_eof()?;
        Ok(stmts)
    }
    fn parse_stmts(&mut self) -> Result<Vec<Stmt>, ParseError> {
        let mut stmts = vec![];
        loop {
            if self.lookahead_delim()? {
                break;
            }
            stmts.push(self.parse_stmt()?);
        }
        Ok(stmts)
    }
    fn parse_stmt(&mut self) -> Result<Stmt, ParseError> {
        let tok = self.next_token()?;
        match tok.kind {
            TokenKind::KeywordLet => {
                self.bump();
                let id_token = self.next_token()?;
                let name = match id_token.kind {
                    TokenKind::Identifier => {
                        self.bump();
                        std::str::from_utf8(&self.buf[id_token.begin..id_token.end])
                            .unwrap()
                            .to_owned()
                    }
                    _ => return Err(ParseError),
                };
                let tok = self.next_token()?;
                if tok.kind != TokenKind::Equal {
                    return Err(ParseError);
                }
                self.bump();
                let init = self.parse_expr()?;
                let tok = self.next_token()?;
                if tok.kind != TokenKind::Semicolon {
                    return Err(ParseError);
                }
                self.bump();
                Ok(Stmt::let_(Ident::from(name), init))
            }
            TokenKind::KeywordThen => {
                self.bump();
                let expr = self.parse_expr()?;
                let tok = self.next_token()?;
                if tok.kind != TokenKind::Semicolon {
                    return Err(ParseError);
                }
                self.bump();
                Ok(Stmt::expr(expr, true))
            }
            _ => {
                let expr = self.parse_expr()?;
                let tok = self.next_token()?;
                if tok.kind != TokenKind::Semicolon {
                    return Err(ParseError);
                }
                self.bump();
                Ok(Stmt::expr(expr, false))
            }
        }
    }
    fn parse_exprs(&mut self) -> Result<Vec<Expr>, ParseError> {
        let mut exprs = vec![];
        loop {
            if self.lookahead_delim()? {
                // Empty list or trailing comma
                break;
            }
            exprs.push(self.parse_expr()?);

            let tok = self.next_token()?;
            if matches!(tok.kind, TokenKind::Comma) {
                self.bump();
            } else if self.lookahead_delim()? {
                // Non-empty list without trailing comma
                break;
            } else {
                return Err(ParseError);
            }
        }
        Ok(exprs)
    }
    fn lookahead_delim(&mut self) -> Result<bool, ParseError> {
        let tok = self.next_token()?;
        Ok(matches!(
            tok.kind,
            TokenKind::RParen | TokenKind::RBrace | TokenKind::Eof
        ))
    }
    fn parse_expr(&mut self) -> Result<Expr, ParseError> {
        let e = self.parse_expr_comparison()?;
        let tok = self.next_token()?;
        match tok.kind {
            TokenKind::Equal => {
                self.bump();
                let Expr::Var { ident } = e else {
                    return Err(ParseError);
                };
                let rhs = self.parse_expr()?;
                return Ok(Expr::Assign {
                    lhs: ident,
                    rhs: Box::new(rhs),
                });
            }
            _ => {}
        }
        Ok(e)
    }
    fn parse_expr_comparison(&mut self) -> Result<Expr, ParseError> {
        let mut e = self.parse_expr_additive()?;
        loop {
            let tok = self.next_token()?;
            let bin_op = match tok.kind {
                TokenKind::LessThan => BinOp::Lt,
                _ => break,
            };
            self.bump();
            let rhs = self.parse_expr_additive()?;
            e = Expr::BinOp {
                op: bin_op,
                lhs: Box::new(e),
                rhs: Box::new(rhs),
            };
        }
        Ok(e)
    }
    fn parse_expr_additive(&mut self) -> Result<Expr, ParseError> {
        let mut e = self.parse_expr_call()?;
        loop {
            let tok = self.next_token()?;
            let bin_op = match tok.kind {
                TokenKind::Plus => BinOp::Add,
                _ => break,
            };
            self.bump();
            let rhs = self.parse_expr_call()?;
            e = Expr::BinOp {
                op: bin_op,
                lhs: Box::new(e),
                rhs: Box::new(rhs),
            };
        }
        Ok(e)
    }
    fn parse_expr_call(&mut self) -> Result<Expr, ParseError> {
        let mut e = self.parse_expr_primary()?;
        loop {
            let tok = self.next_token()?;
            match tok.kind {
                TokenKind::LParen => {
                    self.bump();
                    let args = self.parse_exprs()?;
                    let tok = self.next_token()?;
                    if tok.kind != TokenKind::RParen {
                        return Err(ParseError);
                    }
                    self.bump();
                    e = Expr::Call {
                        callee: Box::new(e),
                        args,
                    };
                }
                _ => {
                    break;
                }
            }
        }
        Ok(e)
    }
    fn parse_expr_primary(&mut self) -> Result<Expr, ParseError> {
        let tok = self.next_token()?;
        match tok.kind {
            TokenKind::LParen => {
                self.bump();
                let e = self.parse_expr()?;
                let tok = self.next_token()?;
                if tok.kind != TokenKind::RParen {
                    return Err(ParseError);
                }
                Ok(e)
            }
            TokenKind::Identifier => {
                self.bump();
                let name = std::str::from_utf8(&self.buf[tok.begin..tok.end]).unwrap();
                Ok(Expr::Var {
                    ident: Ident::from(name),
                })
            }
            TokenKind::KeywordDo => {
                // do { <stmts> }
                self.bump();
                Ok(self.parse_block_expr()?)
            }
            TokenKind::KeywordIf => {
                self.bump();
                let cond = self.parse_expr()?;
                let tok = self.next_token()?;
                match tok.kind {
                    TokenKind::KeywordThen => {
                        // if <cond> then <then> else <else>
                        self.bump();
                        let then = self.parse_expr()?;
                        let tok = self.next_token()?;
                        if tok.kind != TokenKind::KeywordElse {
                            return Err(ParseError);
                        }
                        self.bump();
                        // TODO: primary should not be right-open
                        let else_ = self.parse_expr_primary()?;
                        Ok(Expr::Branch {
                            cond: Box::new(cond),
                            then: Box::new(then),
                            else_: Box::new(else_),
                        })
                    }
                    TokenKind::LBrace => {
                        let then = self.parse_block_expr()?;
                        let tok = self.next_token()?;
                        if tok.kind == TokenKind::KeywordElse {
                            // if <cond> { <then> } else { <else> }

                            // TODO: deal with ambiguous cases like
                            // if <cond1> then if <cond2> { <then2> } else { <else1> }
                            self.bump();
                            // TODO: also handle else-if exceptions
                            let else_ = self.parse_block_expr()?;
                            Ok(Expr::Branch {
                                cond: Box::new(cond),
                                then: Box::new(then),
                                else_: Box::new(else_),
                            })
                        } else {
                            // if <cond> { <then> }
                            Ok(Expr::Branch {
                                cond: Box::new(cond),
                                then: Box::new(then),
                                else_: Box::new(Expr::Block { stmts: vec![] }),
                            })
                        }
                    }
                    _ => return Err(ParseError),
                }
            }
            TokenKind::KeywordWhile => {
                // while <cond> { <body> }
                self.bump();
                let cond = self.parse_expr()?;
                let tok = self.next_token()?;
                if tok.kind != TokenKind::LBrace {
                    return Err(ParseError);
                }
                let body = self.parse_block_expr()?;
                Ok(Expr::While {
                    cond: Box::new(cond),
                    body: Box::new(body),
                })
            }
            TokenKind::Integer => {
                self.bump();
                let s = std::str::from_utf8(&self.buf[tok.begin..tok.end]).unwrap();
                let value = s.parse::<i32>().unwrap();
                Ok(Expr::IntegerLiteral { value })
            }
            TokenKind::String => {
                self.bump();
                let s = std::str::from_utf8(&self.buf[tok.begin + 1..tok.end - 1]).unwrap();
                Ok(Expr::StringLiteral {
                    value: s.to_string(),
                })
            }
            _ => Err(ParseError),
        }
    }
    fn parse_block_expr(&mut self) -> Result<Expr, ParseError> {
        let tok = self.next_token()?;
        if tok.kind != TokenKind::LBrace {
            return Err(ParseError);
        }
        self.bump();
        let stmts = self.parse_stmts()?;
        let tok = self.next_token()?;
        if tok.kind != TokenKind::RBrace {
            return Err(ParseError);
        }
        self.bump();
        Ok(Expr::Block { stmts })
    }
    fn expect_eof(&mut self) -> Result<(), ParseError> {
        let tok = self.next_token()?;
        if tok.kind != TokenKind::Eof {
            return Err(ParseError);
        }
        Ok(())
    }
    fn bump(&mut self) {
        assert!(self.next_token_cache.is_some());
        self.next_token_cache = None;
    }
    fn next_token(&mut self) -> Result<Token, ParseError> {
        if let Some(tok) = self.next_token_cache.clone() {
            return Ok(tok);
        }
        self.skip_whitespace();
        let begin = self.pos;
        let kind = match self.buf.get(self.pos).copied() {
            Some(b'(') => {
                self.pos += 1;
                TokenKind::LParen
            }
            Some(b')') => {
                self.pos += 1;
                TokenKind::RParen
            }
            Some(b'+') => {
                self.pos += 1;
                TokenKind::Plus
            }
            Some(b',') => {
                self.pos += 1;
                TokenKind::Comma
            }
            Some(b';') => {
                self.pos += 1;
                TokenKind::Semicolon
            }
            Some(b'<') => {
                self.pos += 1;
                TokenKind::LessThan
            }
            Some(b'=') => {
                self.pos += 1;
                TokenKind::Equal
            }
            Some(b'{') => {
                self.pos += 1;
                TokenKind::LBrace
            }
            Some(b'}') => {
                self.pos += 1;
                TokenKind::RBrace
            }
            Some(b'a'..=b'z') | Some(b'A'..=b'Z') | Some(b'_') => {
                while self.pos < self.buf.len()
                    && (self.buf[self.pos].is_ascii_alphanumeric() || self.buf[self.pos] == b'_')
                {
                    self.pos += 1;
                }
                match &self.buf[begin..self.pos] {
                    // TODO: other reserved identifiers
                    b"true" | b"false" => todo!(),
                    b"do" => TokenKind::KeywordDo,
                    b"else" => TokenKind::KeywordElse,
                    b"if" => TokenKind::KeywordIf,
                    b"let" => TokenKind::KeywordLet,
                    b"then" => TokenKind::KeywordThen,
                    b"while" => TokenKind::KeywordWhile,
                    _ => TokenKind::Identifier,
                }
            }
            Some(b'0'..=b'9') => {
                // TODO: check leading zero
                while self.pos < self.buf.len() && self.buf[self.pos].is_ascii_digit() {
                    self.pos += 1;
                }
                TokenKind::Integer
            }
            Some(b'"') => {
                self.pos += 1;
                while self.pos < self.buf.len() && self.buf[self.pos] != b'"' {
                    // TODO: handle escapes etc.
                    self.pos += 1;
                }
                if self.pos == self.buf.len() {
                    return Err(ParseError);
                }
                self.pos += 1;
                TokenKind::String
            }
            None => TokenKind::Eof,
            _ => return Err(ParseError),
        };
        let end = self.pos;
        let tok = Token { kind, begin, end };
        self.next_token_cache = Some(tok.clone());
        Ok(tok)
    }

    fn skip_whitespace(&mut self) {
        while self.pos < self.buf.len() {
            match self.buf[self.pos] {
                b' ' | b'\n' | b'\r' | b'\t' => self.pos += 1,
                _ => break,
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Token {
    kind: TokenKind,
    begin: usize,
    end: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum TokenKind {
    /// `(`
    LParen,
    /// `)`
    RParen,
    /// `+`
    Plus,
    /// `,`
    Comma,
    /// `;`
    Semicolon,
    /// `<`
    LessThan,
    /// `=`
    Equal,
    /// `{`
    LBrace,
    /// `}`
    RBrace,
    KeywordDo,
    KeywordElse,
    KeywordIf,
    KeywordLet,
    KeywordThen,
    KeywordWhile,
    Identifier,
    Integer,
    String,
    Eof,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_var_ref() {
        assert_eq!(
            Parser::new("x").parse_expr().unwrap(),
            Expr::Var {
                ident: Ident::from("x"),
            }
        );
    }

    #[test]
    fn test_parse_paren() {
        assert_eq!(
            Parser::new("(x)").parse_expr().unwrap(),
            Expr::Var {
                ident: Ident::from("x"),
            }
        );
    }

    #[test]
    fn test_parse_integer_literal() {
        assert_eq!(
            Parser::new("1").parse_expr().unwrap(),
            Expr::IntegerLiteral { value: 1 }
        );
        assert_eq!(
            Parser::new("123").parse_expr().unwrap(),
            Expr::IntegerLiteral { value: 123 }
        );
    }

    #[test]
    fn test_parse_string_literal() {
        assert_eq!(
            Parser::new("\"hello\"").parse_expr().unwrap(),
            Expr::StringLiteral {
                value: "hello".to_string()
            }
        );
    }

    #[test]
    fn test_parse_funcall() {
        assert_eq!(
            Parser::new("f()").parse_expr().unwrap(),
            Expr::Call {
                callee: Box::new(Expr::Var {
                    ident: Ident::from("f"),
                }),
                args: vec![],
            }
        );
        assert_eq!(
            Parser::new("f(x)").parse_expr().unwrap(),
            Expr::Call {
                callee: Box::new(Expr::Var {
                    ident: Ident::from("f"),
                }),
                args: vec![Expr::Var {
                    ident: Ident::from("x"),
                }],
            }
        );
        assert_eq!(
            Parser::new("f(x, y)").parse_expr().unwrap(),
            Expr::Call {
                callee: Box::new(Expr::Var {
                    ident: Ident::from("f"),
                }),
                args: vec![
                    Expr::Var {
                        ident: Ident::from("x"),
                    },
                    Expr::Var {
                        ident: Ident::from("y"),
                    }
                ],
            }
        );
    }

    #[test]
    fn test_parse_if_else_in_block_style() {
        assert_eq!(
            Parser::new("if x { y; } else { z; }").parse_expr().unwrap(),
            Expr::Branch {
                cond: Box::new(Expr::Var {
                    ident: Ident::from("x"),
                }),
                then: Box::new(Expr::Block {
                    stmts: vec![Stmt::expr(
                        Expr::Var {
                            ident: Ident::from("y"),
                        },
                        false,
                    )],
                }),
                else_: Box::new(Expr::Block {
                    stmts: vec![Stmt::expr(
                        Expr::Var {
                            ident: Ident::from("z"),
                        },
                        false,
                    )],
                }),
            }
        );
    }

    #[test]
    fn test_parse_if_without_else_in_block_style() {
        assert_eq!(
            Parser::new("if x { y; }").parse_expr().unwrap(),
            Expr::Branch {
                cond: Box::new(Expr::Var {
                    ident: Ident::from("x"),
                }),
                then: Box::new(Expr::Block {
                    stmts: vec![Stmt::expr(
                        Expr::Var {
                            ident: Ident::from("y"),
                        },
                        false,
                    )],
                }),
                else_: Box::new(Expr::Block { stmts: vec![] }),
            }
        );
    }

    #[test]
    fn test_parse_if_then_else() {
        assert_eq!(
            Parser::new("if x then y else z").parse_expr().unwrap(),
            Expr::Branch {
                cond: Box::new(Expr::Var {
                    ident: Ident::from("x"),
                }),
                then: Box::new(Expr::Var {
                    ident: Ident::from("y"),
                }),
                else_: Box::new(Expr::Var {
                    ident: Ident::from("z"),
                }),
            }
        );
    }

    #[test]
    fn test_parse_while() {
        assert_eq!(
            Parser::new("while x { y; }").parse_expr().unwrap(),
            Expr::While {
                cond: Box::new(Expr::Var {
                    ident: Ident::from("x"),
                }),
                body: Box::new(Expr::Block {
                    stmts: vec![Stmt::expr(
                        Expr::Var {
                            ident: Ident::from("y"),
                        },
                        false,
                    )],
                }),
            }
        );
    }

    #[test]
    fn test_parse_do_expr() {
        assert_eq!(
            Parser::new("do { x; }").parse_expr().unwrap(),
            Expr::Block {
                stmts: vec![Stmt::expr(
                    Expr::Var {
                        ident: Ident::from("x"),
                    },
                    false,
                )],
            }
        );
    }

    #[test]
    fn test_parse_additive() {
        assert_eq!(
            Parser::new("1 + 2").parse_expr().unwrap(),
            Expr::BinOp {
                op: BinOp::Add,
                lhs: Box::new(Expr::IntegerLiteral { value: 1 }),
                rhs: Box::new(Expr::IntegerLiteral { value: 2 }),
            }
        );
    }

    #[test]
    fn test_parse_comparison() {
        assert_eq!(
            Parser::new("1 < 2").parse_expr().unwrap(),
            Expr::BinOp {
                op: BinOp::Lt,
                lhs: Box::new(Expr::IntegerLiteral { value: 1 }),
                rhs: Box::new(Expr::IntegerLiteral { value: 2 }),
            }
        );
    }

    #[test]
    fn test_parse_assignment() {
        assert_eq!(
            Parser::new("x = 1").parse_expr().unwrap(),
            Expr::Assign {
                lhs: Ident::from("x"),
                rhs: Box::new(Expr::IntegerLiteral { value: 1 }),
            }
        );
    }

    #[test]
    fn test_parse_let_stmt() {
        assert_eq!(
            Parser::new("let x = 1;").parse_stmt().unwrap(),
            Stmt::let_(Ident::from("x"), Expr::IntegerLiteral { value: 1 })
        );
    }

    #[test]
    fn test_parse_then_stmt() {
        assert_eq!(
            Parser::new("then 1;").parse_stmt().unwrap(),
            Stmt::expr(Expr::IntegerLiteral { value: 1 }, true)
        );
    }

    #[test]
    fn test_parse_expr_stmt() {
        assert_eq!(
            Parser::new("1;").parse_stmt().unwrap(),
            Stmt::expr(Expr::IntegerLiteral { value: 1 }, false)
        );
    }

    #[test]
    fn test_parse_stmts() {
        assert_eq!(
            Parser::new("let x = 1; then x;").parse_stmts().unwrap(),
            vec![
                Stmt::let_(Ident::from("x"), Expr::IntegerLiteral { value: 1 }),
                Stmt::expr(
                    Expr::Var {
                        ident: Ident::from("x"),
                    },
                    true
                )
            ]
        );
    }

    #[test]
    fn test_parse_program() {
        assert_eq!(
            Parser::new("use lang::\"0.0.1\";\nlet x = 1;\nthen x;\n")
                .parse_program()
                .unwrap(),
            vec![
                Stmt::let_(Ident::from("x"), Expr::IntegerLiteral { value: 1 }),
                Stmt::expr(
                    Expr::Var {
                        ident: Ident::from("x"),
                    },
                    true
                )
            ]
        );
    }
}
