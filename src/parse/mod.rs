mod parse_expr;
mod parse_stmt;

use crate::ast::{Crate, Stmt};
use crate::lexer::{Lexer, Token, TokenKind};

use self::parse_stmt::is_stmt_start;

pub struct Parser {
    lexer: Lexer,
    next_node_id: u32,
}

impl Parser {
    pub fn new(lexer: Lexer) -> Self {
        Parser {
            lexer,
            next_node_id: 0,
        }
    }

    pub fn get_next_id(&mut self) -> u32 {
        let id = self.next_node_id;
        self.next_node_id += 1;
        id
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
