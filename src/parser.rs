use crate::ast::Expr;
use crate::cctx::Id;

#[derive(Debug)]
pub struct ParseError;

pub fn parse_expr(source: &str) -> Result<Expr, ParseError> {
    let mut parser = Parser {
        buf: source.as_bytes().to_vec(),
        pos: 0,
    };
    parser.parse_expr()
}

#[derive(Debug)]
struct Parser {
    buf: Vec<u8>,
    pos: usize,
}

impl Parser {
    fn parse_expr(&mut self) -> Result<Expr, ParseError> {
        let tok = self.next_token()?;
        match tok.kind {
            TokenKind::Identifier => {
                let name = std::str::from_utf8(&self.buf[tok.begin..tok.end]).unwrap();
                Ok(Expr::Var {
                    name: name.to_string(),
                    id: Id::dummy(),
                })
            }
            TokenKind::Integer => {
                let s = std::str::from_utf8(&self.buf[tok.begin..tok.end]).unwrap();
                let value = s.parse::<i32>().unwrap();
                Ok(Expr::IntegerLiteral { value })
            }
            TokenKind::String => {
                let s = std::str::from_utf8(&self.buf[tok.begin + 1..tok.end - 1]).unwrap();
                Ok(Expr::StringLiteral {
                    value: s.to_string(),
                })
            }
        }
    }
    fn next_token(&mut self) -> Result<Token, ParseError> {
        self.skip_whitespace();
        let begin = self.pos;
        let kind = match self.buf.get(self.pos).copied() {
            Some(b'a'..=b'z') | Some(b'A'..=b'Z') | Some(b'_') => {
                while self.pos < self.buf.len()
                    && (self.buf[self.pos].is_ascii_alphanumeric() || self.buf[self.pos] == b'_')
                {
                    self.pos += 1;
                }
                match &self.buf[begin..self.pos] {
                    // TODO: other reserved identifiers
                    b"true" | b"false" => todo!(),
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
            _ => return Err(ParseError),
        };
        let end = self.pos;
        Ok(Token { kind, begin, end })
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
    Identifier,
    Integer,
    String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_var_ref() {
        assert_eq!(
            parse_expr("x").unwrap(),
            Expr::Var {
                name: "x".to_string(),
                id: Id::dummy(),
            }
        );
    }

    #[test]
    fn test_parse_integer_literal() {
        assert_eq!(parse_expr("1").unwrap(), Expr::IntegerLiteral { value: 1 });
        assert_eq!(
            parse_expr("123").unwrap(),
            Expr::IntegerLiteral { value: 123 }
        );
    }

    #[test]
    fn test_parse_string_literal() {
        assert_eq!(
            parse_expr("\"hello\"").unwrap(),
            Expr::StringLiteral {
                value: "hello".to_string()
            }
        );
    }
}
