use std::rc::Rc;

use super::parse_expr::is_expr_start;
use super::Parser;
use crate::ast::{Ident, LetStmt, Stmt, StmtKind};
use crate::lexer::{Token, TokenKind};
use crate::ty::Ty;

pub fn is_stmt_start(t: &Token) -> bool {
    is_expr_start(t) || matches!(t.kind, TokenKind::Let)
}

impl Parser {
    pub fn parse_stmt(&mut self) -> Option<Stmt> {
        let t = self.peek_token().unwrap();

        match &t.kind {
            TokenKind::Let => self.parse_let_stmt(),
            _ if is_expr_start(t) => {
                let expr = self.parse_expr()?;
                if !self.skip_expected_token(TokenKind::Semi) {
                    eprintln!("Expected ';', but found {:?}", self.peek_token().unwrap());
                    return None;
                }
                Some(Stmt {
                    kind: StmtKind::ExprStmt(Box::new(expr)),
                })
            }
            _ => {
                eprintln!("Expected expr, but found {:?}", self.peek_token().unwrap());
                None
            }
        }
    }

    fn parse_let_stmt(&mut self) -> Option<Stmt> {
        assert!(self.skip_expected_token(TokenKind::Let));
        let t = self.skip_token().unwrap();
        let TokenKind::Ident(symbol) = t.kind else {
                    eprintln!("Expected ident pattern, but found {:?}", t);
                    return None;
                };
        // skip colon
        if !self.skip_expected_token(TokenKind::Colon) {
            eprintln!("Expected ':', but found {:?}", self.peek_token().unwrap());
            return None;
        }
        // parse type
        let ty = self.parse_type()?;
        // skip semi
        if !self.skip_expected_token(TokenKind::Semi) {
            eprintln!("Expected ';', but found {:?}", self.peek_token().unwrap());
            return None;
        }
        Some(Stmt {
            kind: StmtKind::Let(LetStmt {
                ident: Ident { symbol },
                ty: Rc::new(ty),
            }),
        })
    }

    fn parse_type(&mut self) -> Option<Ty> {
        let t = self.skip_token().unwrap();
        match &t.kind {
            // Unit type: ()
            TokenKind::OpenParen => {
                if !self.skip_expected_token(TokenKind::CloseParen) {
                    eprintln!("Expected ')', but found {:?}", self.peek_token().unwrap());
                    None
                } else {
                    Some(Ty::Unit)
                }
            }
            // Never type: !
            TokenKind::Bang => Some(Ty::Never),
            // i32
            TokenKind::I32 => Some(Ty::I32),
            // bool
            TokenKind::Bool => Some(Ty::Bool),
            _ => {
                eprintln!("Expected type, but found {:?}", t);
                None
            }
        }
    }
}
