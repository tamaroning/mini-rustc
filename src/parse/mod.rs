mod parse_expr;

use crate::ast::{self, Expr, ExprKind, UnOp};
use crate::lexer::{self, Lexer, Token, TokenKind};

pub struct Parser {
    lexer: Lexer,
}

impl Parser {
    pub fn new(lexer: Lexer) -> Self {
        Parser { lexer }
    }

    fn peek_token(&mut self) -> Option<&Token> {
        self.lexer.peek_token()
    }

    fn skip_token(&mut self) -> Option<Token> {
        self.lexer.skip_token()
    }

    /// Skip token only when bumping into the expected token.
    fn skip_expected_token(&mut self, kind: TokenKind) -> bool {
        match self.lexer.peek_token() {
            Some(t) if t.kind == kind => {
                self.lexer.skip_token();
                true
            }
            _ => false,
        }
    }

    fn at_eof(&mut self) -> bool {
        matches!(
            self.peek_token(),
            Some(&Token {
                kind: TokenKind::Eof,
                ..
            })
        )
    }

    pub fn parse_crate(&mut self) -> Option<Expr> {
        let expr = self.parse_expr();
        if !self.at_eof() {
            return None;
        }
        expr
    }
}