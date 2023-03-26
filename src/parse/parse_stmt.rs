use super::parse_expr::is_expr_start;
use super::Parser;
use crate::ast::{Ident, LetStmt, Stmt, StmtKind};
use crate::lexer::{Token, TokenKind};

pub fn is_stmt_start(t: &Token) -> bool {
    is_expr_start(t) || matches!(t.kind, TokenKind::Let)
}

impl Parser {
    pub fn parse_stmt(&mut self) -> Option<Stmt> {
        let t = self.peek_token().unwrap();

        match &t.kind {
            TokenKind::Let => {
                self.skip_token();
                let t = self.skip_token().unwrap();
                let TokenKind::Ident(symbol) = t.kind else {
                    eprintln!("Expected ident, but found {:?}", t);
                    return None;
                };
                if !self.skip_expected_token(TokenKind::Semi) {
                    eprintln!("Expected ';', but found {:?}", self.peek_token().unwrap());
                    return None;
                }
                Some(Stmt {
                    kind: StmtKind::Let(LetStmt {
                        ident: Ident { symbol },
                    }),
                })
            }
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
}
