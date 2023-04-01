use super::parse_expr::is_expr_start;
use super::Parser;
use crate::ast::{Block, LetStmt, Stmt, StmtKind};
use crate::lexer::{Token, TokenKind};
use std::rc::Rc;

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
                let t = self.peek_token()?;
                if t.kind == TokenKind::Semi {
                    self.skip_token();
                    Some(Stmt {
                        kind: StmtKind::Semi(Box::new(expr)),
                        id: self.get_next_id(),
                    })
                } else {
                    Some(Stmt {
                        kind: StmtKind::Expr(Box::new(expr)),
                        id: self.get_next_id(),
                    })
                }
            }
            _ => {
                eprintln!("Expected expr, but found {:?}", self.peek_token().unwrap());
                None
            }
        }
    }

    fn parse_let_stmt(&mut self) -> Option<Stmt> {
        assert!(self.skip_expected_token(TokenKind::Let));
        let ident = self.parse_ident()?;
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
                ident,
                ty: Rc::new(ty),
            }),
            id: self.get_next_id(),
        })
    }

    /// block ::= "{" stmt* "}"
    pub fn parse_block(&mut self) -> Option<Block> {
        if !self.skip_expected_token(TokenKind::OpenBrace) {
            eprintln!("Expected '{{' but found {:?}", self.peek_token()?);
            return None;
        }
        let mut stmts = vec![];
        loop {
            let t = self.peek_token()?;
            if is_stmt_start(t) {
                let stmt = self.parse_stmt()?;
                stmts.push(stmt);
            } else if t.kind == TokenKind::CloseBrace {
                self.skip_token();
                return Some(Block { stmts });
            } else {
                dbg!(&stmts);
                eprintln!("Expected '}}' or statement, but found {:?}", t);
                break;
            }
        }
        None
    }
}
