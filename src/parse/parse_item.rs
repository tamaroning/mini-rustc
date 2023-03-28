use std::rc::Rc;

use crate::ast::Func;
use crate::lexer::{Token, TokenKind};

use super::Parser;

pub fn is_item_start(token: &Token) -> bool {
    matches!(token.kind, TokenKind::Fn)
}

impl Parser {
    /// item ::= func
    pub fn parse_item(&mut self) -> Option<Func> {
        self.parse_func()
    }

    /// func ::= "fn" ident "(" ")" "->" "i32" block
    pub fn parse_func(&mut self) -> Option<Func> {
        if !self.skip_expected_token(TokenKind::Fn) {
            return None;
        }
        let name = self.parse_ident()?;
        if !self.skip_expected_token(TokenKind::OpenParen) {
            return None;
        }
        if !self.skip_expected_token(TokenKind::CloseParen) {
            return None;
        }
        if !self.skip_expected_token(TokenKind::Arrow) {
            return None;
        }
        let ret_ty = self.parse_type()?;
        let block = self.parse_block()?;

        Some(Func {
            name,
            ret_ty: Rc::new(ret_ty),
            body: block,
            id: self.get_next_id(),
        })
    }
}
