use crate::ast::{Func, Stmt};
use crate::lexer::{Token, TokenKind};

use super::parse_stmt::is_stmt_start;
use super::Parser;

pub fn is_item_start(token: &Token) -> bool {
    matches!(token.kind, TokenKind::Fn)
}

impl Parser {
    /// item ::= func
    pub fn parse_item(&mut self) -> Option<Func> {
        self.parse_func()
    }

    /// func ::= "fn" ident "(" ")" "->" i32 "{" stmt*  "}"
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
        if !self.skip_expected_token(TokenKind::I32) {
            return None;
        }

        if !self.skip_expected_token(TokenKind::OpenBrace) {
            return None;
        }
        let stmts = self.parse_stmts()?;
        if !self.skip_expected_token(TokenKind::CloseBrace) {
            return None;
        }
        Some(Func { name, stmts })
    }

    fn parse_stmts(&mut self) -> Option<Vec<Stmt>> {
        let mut stmts = vec![];

        while is_stmt_start(self.peek_token().unwrap()) {
            if let Some(stmt) = self.parse_stmt() {
                stmts.push(stmt);
            } else {
                return None;
            }
        }
        Some(stmts)
    }
}
