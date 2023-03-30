use std::rc::Rc;

use crate::ast::{Func, Ident};
use crate::lexer::{Token, TokenKind};
use crate::ty::Ty;

use super::Parser;

pub fn is_item_start(token: &Token) -> bool {
    matches!(token.kind, TokenKind::Fn)
}

impl Parser {
    /// item ::= func
    pub fn parse_item(&mut self) -> Option<Func> {
        self.parse_func()
    }

    /// func ::= "fn" ident "(" funcParams? ")" "->" "i32" block
    pub fn parse_func(&mut self) -> Option<Func> {
        if !self.skip_expected_token(TokenKind::Fn) {
            return None;
        }
        let name = self.parse_ident()?;
        if !self.skip_expected_token(TokenKind::OpenParen) {
            return None;
        }
        let t = self.peek_token()?;
        let params = if t.kind == TokenKind::CloseParen {
            vec![]
        } else {
            self.parse_func_params()?
        };
        if !self.skip_expected_token(TokenKind::CloseParen) {
            eprintln!("Expected '(', but found {:?}", self.peek_token()?);
            return None;
        }

        if !self.skip_expected_token(TokenKind::Arrow) {
            return None;
        }
        let ret_ty = self.parse_type()?;
        let block = self.parse_block()?;

        Some(Func {
            name,
            params,
            ret_ty: Rc::new(ret_ty),
            body: block,
            id: self.get_next_id(),
        })
    }

    /// funcParams ::= funcParam ("," funcParam)* ","?
    /// funcParam ::= ident ":" type
    fn parse_func_params(&mut self) -> Option<Vec<(Ident, Rc<Ty>)>> {
        let mut params = vec![];
        params.push(self.parse_func_param()?);

        while matches!(self.peek_token()?.kind, TokenKind::Comma) {
            self.skip_token();
            if matches!(self.peek_token()?.kind, TokenKind::Ident(_)) {
                params.push(self.parse_func_param()?);
            }
        }
        Some(params)
    }

    fn parse_func_param(&mut self) -> Option<(Ident, Rc<Ty>)> {
        let ident = self.parse_ident()?;
        if !self.skip_expected_token(TokenKind::Colon) {
            eprintln!("Expected ':', but found {:?}", self.peek_token()?);
            return None;
        }
        let ty = self.parse_type()?;
        Some((ident, Rc::new(ty)))
    }
}
