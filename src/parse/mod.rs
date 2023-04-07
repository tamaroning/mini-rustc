mod parse_expr;
mod parse_item;
mod parse_stmt;

use std::rc::Rc;

use self::parse_item::is_item_start;
use crate::ast::{Crate, Ident, Item};
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

    fn peek_token(&mut self) -> &Token {
        self.lexer.peek_token()
    }

    fn skip_token(&mut self) -> Token {
        self.lexer.skip_token()
    }

    /// Skip token only when bumping into the expected token.
    fn skip_expected_token(&mut self, kind: TokenKind) -> bool {
        let t = self.peek_token();
        if t.kind == kind {
            self.lexer.skip_token();
            true
        } else {
            false
        }
    }

    fn at_eof(&mut self) -> bool {
        matches!(
            self.peek_token(),
            &Token {
                kind: TokenKind::Eof,
                ..
            }
        )
    }

    /// crate ::= item*
    pub fn parse_crate(&mut self) -> Option<Crate> {
        let items = self.parse_items()?;
        if !self.at_eof() {
            eprintln!(
                "Expected crate item but found `{}`",
                self.peek_token().span.to_snippet()
            );
            return None;
        }
        Some(Crate { items })
    }

    fn parse_items(&mut self) -> Option<Vec<Item>> {
        let mut items = vec![];

        while is_item_start(self.peek_token()) {
            items.push(self.parse_item()?);
        }
        Some(items)
    }

    fn parse_ident(&mut self) -> Option<Ident> {
        let t = self.skip_token();
        if let TokenKind::Ident(symbol) = t.kind {
            Some(Ident {
                symbol: Rc::new(symbol),
                span: t.span,
                id: self.get_next_id(),
            })
        } else {
            eprintln!("Expected ident, but found `{}`", t.span.to_snippet());
            None
        }
    }
}
