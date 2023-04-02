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
        let t = self.peek_token();
        let mut span = t.span.clone();

        match &t.kind {
            TokenKind::Let => self.parse_let_stmt(),
            _ if is_expr_start(t) => {
                let expr = self.parse_expr()?;
                span = span.concat(&expr.span);

                let t = self.peek_token();
                if t.kind == TokenKind::Semi {
                    // skip ';'
                    span = span.concat(&self.skip_token().span);
                    Some(Stmt {
                        kind: StmtKind::Semi(Box::new(expr)),
                        id: self.get_next_id(),
                        span,
                    })
                } else {
                    Some(Stmt {
                        kind: StmtKind::Expr(Box::new(expr)),
                        id: self.get_next_id(),
                        span,
                    })
                }
            }
            _ => {
                eprintln!("Expected expr, but found {:?}", self.peek_token());
                None
            }
        }
    }

    /// https://doc.rust-lang.org/reference/statements.html#let-statements
    fn parse_let_stmt(&mut self) -> Option<Stmt> {
        // skip "let"
        let mut span = self.skip_token().span;

        let ident = self.parse_ident()?;
        // skip colon
        if !self.skip_expected_token(TokenKind::Colon) {
            eprintln!("Expected ':', but found {:?}", self.peek_token());
            return None;
        }
        // parse type
        let ty = self.parse_type()?;

        // parse ("=" expr)?
        let t = self.peek_token();
        let init = if t.kind == TokenKind::Eq {
            self.skip_token();
            Some(self.parse_expr()?)
        } else {
            None
        };

        // skip semi
        span = span.concat(&self.peek_token().span);
        if !self.skip_expected_token(TokenKind::Semi) {
            eprintln!(
                "Expected ';' for let statement, but found {:?}",
                self.peek_token()
            );
            return None;
        }

        Some(Stmt {
            kind: StmtKind::Let(LetStmt {
                ident,
                ty: Rc::new(ty),
                init,
            }),
            id: self.get_next_id(),
            span,
        })
    }

    /// block ::= "{" stmt* "}"
    pub fn parse_block(&mut self) -> Option<Block> {
        let mut span = self.peek_token().span.clone();

        if !self.skip_expected_token(TokenKind::OpenBrace) {
            eprintln!("Expected '{{' but found {:?}", self.peek_token());
            return None;
        }
        let mut stmts = vec![];
        loop {
            let t = self.peek_token();
            if is_stmt_start(t) {
                let stmt = self.parse_stmt()?;
                span = span.concat(&stmt.span);
                stmts.push(stmt);
            } else if t.kind == TokenKind::CloseBrace {
                // skip '}'
                span = span.concat(&self.skip_token().span);
                return Some(Block { stmts, span });
            } else {
                dbg!(&stmts);
                eprintln!("Expected '}}' or statement, but found {:?}", t);
                break;
            }
        }
        None
    }
}
