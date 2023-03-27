mod parse_expr;
mod parse_item;
mod parse_stmt;

use self::parse_item::is_item_start;
use crate::ast::{Crate, Func, Ident};
use crate::lexer::{Lexer, Token, TokenKind};

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

    /// crate ::= item*
    pub fn parse_crate(&mut self) -> Option<Crate> {
        let Some(items) = self.parse_items() else {
            return None;
        };
        if !self.at_eof() {
            return None;
        }
        Some(Crate { items })
    }

    fn parse_items(&mut self) -> Option<Vec<Func>> {
        let mut items = vec![];

        while is_item_start(self.peek_token().unwrap()) {
            if let Some(item) = self.parse_item() {
                items.push(item);
            } else {
                return None;
            }
        }
        Some(items)
    }

    fn parse_ident(&mut self) -> Option<Ident> {
        let t = self.skip_token()?;
        if let TokenKind::Ident(symbol) = t.kind {
            Some(Ident { symbol })
        } else {
            eprintln!("Expected ident, but found {:?}", t);
            None
        }
    }
}
