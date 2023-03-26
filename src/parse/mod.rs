mod parse_expr;

use crate::ast::{Crate, Stmt, StmtKind};
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

    pub fn parse_crate(&mut self) -> Option<Crate> {
        let Some(stmts) = self.parse_stmts() else {
            return None;
        };
        if !self.at_eof() {
            return None;
        }
        Some(Crate { stmts })
    }

    fn parse_stmts(&mut self) -> Option<Vec<Stmt>> {
        let mut stmts = vec![];

        while self.is_stmt_start() {
            if let Some(stmt) = self.parse_stmt() {
                stmts.push(stmt);
            } else {
                return None;
            }
        }
        Some(stmts)
    }

    fn parse_stmt(&mut self) -> Option<Stmt> {
        if self.is_expr_start() {
            let Some(expr) = self.parse_expr() else {
                return None;
            };
            if !self.skip_expected_token(TokenKind::Semi) {
                eprintln!("Expected ';', but found {:?}", self.peek_token().unwrap());
                return None;
            }
            Some(Stmt {
                kind: StmtKind::ExprStmt(Box::new(expr)),
            })
        } else {
            eprintln!("Expected expr, but found {:?}", self.peek_token().unwrap());
            None
        }
    }

    fn is_stmt_start(&mut self) -> bool {
        self.is_expr_start()
    }
}
