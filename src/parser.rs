use crate::ast::{BinOp, Expr, Stmt};
use crate::cctx::Id;

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
                Ok(Stmt::Let {
                    name,
                    id: Id::dummy(),
                    init,
                })
            }
            TokenKind::KeywordThen => {
                self.bump();
                let expr = self.parse_expr()?;
                let tok = self.next_token()?;
                if tok.kind != TokenKind::Semicolon {
                    return Err(ParseError);
                }
                self.bump();
                Ok(Stmt::Expr {
                    expr,
                    use_value: true,
                })
            }
            _ => {
                let expr = self.parse_expr()?;
                let tok = self.next_token()?;
                if tok.kind != TokenKind::Semicolon {
                    return Err(ParseError);
                }
                self.bump();
                Ok(Stmt::Expr {
                    expr,
                    use_value: false,
                })
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
        Ok(matches!(tok.kind, TokenKind::RParen | TokenKind::Eof))
    }
    fn parse_expr(&mut self) -> Result<Expr, ParseError> {
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
                    name: name.to_string(),
                    id: Id::dummy(),
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
            Some(b'a'..=b'z') | Some(b'A'..=b'Z') | Some(b'_') => {
                while self.pos < self.buf.len()
                    && (self.buf[self.pos].is_ascii_alphanumeric() || self.buf[self.pos] == b'_')
                {
                    self.pos += 1;
                }
                match &self.buf[begin..self.pos] {
                    // TODO: other reserved identifiers
                    b"true" | b"false" => todo!(),
                    b"let" => TokenKind::KeywordLet,
                    b"then" => TokenKind::KeywordThen,
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
    KeywordLet,
    KeywordThen,
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
                name: "x".to_string(),
                id: Id::dummy(),
            }
        );
    }

    #[test]
    fn test_parse_paren() {
        assert_eq!(
            Parser::new("(x)").parse_expr().unwrap(),
            Expr::Var {
                name: "x".to_string(),
                id: Id::dummy(),
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
                    name: "f".to_string(),
                    id: Id::dummy(),
                }),
                args: vec![],
            }
        );
        assert_eq!(
            Parser::new("f(x)").parse_expr().unwrap(),
            Expr::Call {
                callee: Box::new(Expr::Var {
                    name: "f".to_string(),
                    id: Id::dummy(),
                }),
                args: vec![Expr::Var {
                    name: "x".to_string(),
                    id: Id::dummy(),
                }],
            }
        );
        assert_eq!(
            Parser::new("f(x, y)").parse_expr().unwrap(),
            Expr::Call {
                callee: Box::new(Expr::Var {
                    name: "f".to_string(),
                    id: Id::dummy(),
                }),
                args: vec![
                    Expr::Var {
                        name: "x".to_string(),
                        id: Id::dummy(),
                    },
                    Expr::Var {
                        name: "y".to_string(),
                        id: Id::dummy(),
                    }
                ],
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
    fn test_parse_let_stmt() {
        assert_eq!(
            Parser::new("let x = 1;").parse_stmt().unwrap(),
            Stmt::Let {
                name: "x".to_string(),
                id: Id::dummy(),
                init: Expr::IntegerLiteral { value: 1 },
            }
        );
    }

    #[test]
    fn test_parse_then_stmt() {
        assert_eq!(
            Parser::new("then 1;").parse_stmt().unwrap(),
            Stmt::Expr {
                expr: Expr::IntegerLiteral { value: 1 },
                use_value: true,
            }
        );
    }

    #[test]
    fn test_parse_expr_stmt() {
        assert_eq!(
            Parser::new("1;").parse_stmt().unwrap(),
            Stmt::Expr {
                expr: Expr::IntegerLiteral { value: 1 },
                use_value: false,
            }
        );
    }

    #[test]
    fn test_parse_stmts() {
        assert_eq!(
            Parser::new("let x = 1; then x;").parse_stmts().unwrap(),
            vec![
                Stmt::Let {
                    name: "x".to_string(),
                    id: Id::dummy(),
                    init: Expr::IntegerLiteral { value: 1 },
                },
                Stmt::Expr {
                    expr: Expr::Var {
                        name: "x".to_string(),
                        id: Id::dummy(),
                    },
                    use_value: true,
                }
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
                Stmt::Let {
                    name: "x".to_string(),
                    id: Id::dummy(),
                    init: Expr::IntegerLiteral { value: 1 },
                },
                Stmt::Expr {
                    expr: Expr::Var {
                        name: "x".to_string(),
                        id: Id::dummy(),
                    },
                    use_value: true,
                }
            ]
        );
    }
}
