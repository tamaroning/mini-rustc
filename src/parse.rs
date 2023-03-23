use crate::ast::{Expr, ExprKind};
use crate::lexer::{Lexer, Token, TokenKind};

pub struct Parser {
    lexer: Lexer,
}

impl Parser {
    pub fn new(lexer: Lexer) -> Self {
        Parser { lexer }
    }

    fn peek() -> Token {
        todo!()
    }

    pub fn parse_crate(&mut self) -> Result<Expr, ()> {
        self.parse_expr()
    }

    pub fn parse_expr(&mut self) -> Result<Expr, ()> {
        let res = self.lexer.tokenize();
        match &res {
            Ok(t) => match t.kind {
                TokenKind::NumLit(n) => Ok(Expr {
                    kind: ExprKind::NumLit(n),
                }),
                _ => Err(()),
            },
            Err(()) => Err(()),
        }
    }
}
