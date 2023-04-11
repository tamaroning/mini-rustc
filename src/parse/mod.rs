mod parse_expr;
mod parse_item;
mod parse_stmt;

use self::parse_item::is_item_start;
use crate::ast::{Crate, Item, NodeId, Path};
use crate::lexer::{Lexer, Token, TokenKind};
use crate::span::Ident;
use std::rc::Rc;

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

    pub fn get_next_id(&mut self) -> NodeId {
        let id = self.next_node_id;
        self.next_node_id += 1;
        NodeId::new(id)
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
        let id = self.get_next_id();
        Some(Crate { items, id })
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
            })
        } else {
            eprintln!("Expected ident, but found `{}`", t.span.to_snippet());
            None
        }
    }

    /// path ::= pathSegment ("::" PathSegment)*
    /// pathSegment ::= ident
    /// ref: https://doc.rust-lang.org/reference/paths.html#paths
    fn parse_path(&mut self) -> Option<Path> {
        let ident = self.parse_ident()?;
        let mut span = ident.span.clone();
        let mut segs = vec![ident];

        while self.peek_token().kind == TokenKind::ColCol {
            self.skip_token();
            let new_seg = self.parse_ident()?;
            span = span.concat(&new_seg.span);
            segs.push(new_seg);
        }

        Some(Path {
            span,
            segments: segs,
        })
    }
}
