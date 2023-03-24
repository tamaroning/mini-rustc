use crate::ast::{Expr, ExprKind};
use crate::lexer::{Lexer, Token, TokenKind};

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

    fn eat_expected(&mut self, kind: TokenKind) -> bool {
        match self.lexer.peek_token() {
            Some(t) => t.kind == kind,
            None => false,
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
        if !self.eat_expected(TokenKind::Eof) {
            return None;
        }
        expr
    }

    pub fn parse_expr(&mut self) -> Option<Expr> {
        let t = self.lexer.skip_token();
        match &t {
            Some(t) => match t.kind {
                TokenKind::NumLit(n) => Some(Expr {
                    kind: ExprKind::NumLit(n),
                }),
                _ => None,
            },
            None => None,
        }
    }
}
