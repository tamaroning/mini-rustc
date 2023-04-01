use std::rc::Rc;

use crate::ast::{Func, Ident, Item, ItemKind, StructItem};
use crate::lexer::{self, Token, TokenKind};
use crate::ty::Ty;

use super::Parser;

pub fn is_item_start(token: &Token) -> bool {
    matches!(token.kind, TokenKind::Fn | TokenKind::Struct)
}

impl Parser {
    /// item ::= func
    pub fn parse_item(&mut self) -> Option<Item> {
        let t = self.peek_token()?;
        match &t.kind {
            TokenKind::Fn => Some(Item {
                kind: ItemKind::Func(self.parse_func()?),
            }),
            TokenKind::Struct => Some(Item {
                kind: ItemKind::Struct(self.parse_struct_item()?),
            }),
            _ => {
                eprintln!("Expected item, but found {:?}", self.peek_token());
                None
            }
        }
    }

    /// func ::= "fn" ident "(" funcParams? ")" "->" "i32" block
    pub fn parse_func(&mut self) -> Option<Func> {
        if !self.skip_expected_token(TokenKind::Fn) {
            return None;
        }
        let name = self.parse_ident()?;
        if !self.skip_expected_token(TokenKind::OpenParen) {
            return None;
        }
        let t = self.peek_token()?;
        let params = if t.kind == TokenKind::CloseParen {
            vec![]
        } else {
            self.parse_func_params()?
        };
        if !self.skip_expected_token(TokenKind::CloseParen) {
            eprintln!("Expected '(', but found {:?}", self.peek_token()?);
            return None;
        }

        if !self.skip_expected_token(TokenKind::Arrow) {
            return None;
        }
        let ret_ty = self.parse_type()?;
        let block = self.parse_block()?;

        Some(Func {
            name,
            params,
            ret_ty: Rc::new(ret_ty),
            body: block,
            id: self.get_next_id(),
        })
    }

    /// funcParams ::= funcParam ("," funcParam)* ","?
    /// funcParam ::= ident ":" type
    fn parse_func_params(&mut self) -> Option<Vec<(Ident, Rc<Ty>)>> {
        let mut params = vec![];
        params.push(self.parse_func_param()?);

        while matches!(self.peek_token()?.kind, TokenKind::Comma) {
            self.skip_token();
            if matches!(self.peek_token()?.kind, TokenKind::Ident(_)) {
                params.push(self.parse_func_param()?);
            }
        }
        Some(params)
    }

    fn parse_func_param(&mut self) -> Option<(Ident, Rc<Ty>)> {
        let ident = self.parse_ident()?;
        if !self.skip_expected_token(TokenKind::Colon) {
            eprintln!("Expected ':', but found {:?}", self.peek_token());
            return None;
        }
        let ty = self.parse_type()?;
        Some((ident, Rc::new(ty)))
    }

    fn parse_struct_item(&mut self) -> Option<StructItem> {
        if !self.skip_expected_token(TokenKind::Struct) {
            eprintln!("Expected \"struct\", but found {:?}", self.peek_token());
            return None;
        }
        let ident = self.parse_ident()?;
        if !self.skip_expected_token(TokenKind::OpenBrace) {
            eprintln!("Expected '{{', but found {:?}", self.peek_token());
            return None;
        }

        let fields = if matches!(self.peek_token().unwrap().kind, TokenKind::Ident(_)) {
            self.parse_struct_fields()?
        } else {
            vec![]
        };
        if !self.skip_expected_token(TokenKind::CloseBrace) {
            eprintln!("Expected '}}', but found {:?}", self.peek_token());
            return None;
        }

        Some(StructItem { ident, fields })
    }

    fn parse_struct_fields(&mut self) -> Option<Vec<(Ident, Rc<Ty>)>> {
        let mut fields = vec![];
        fields.push(self.parse_struct_field()?);

        while matches!(self.peek_token()?.kind, TokenKind::Comma) {
            self.skip_token();
            if matches!(self.peek_token()?.kind, TokenKind::Ident(_)) {
                fields.push(self.parse_struct_field()?);
            }
        }
        Some(fields)
    }

    /// structField ::= ident ":" type
    fn parse_struct_field(&mut self) -> Option<(Ident, Rc<Ty>)> {
        let name = self.parse_ident()?;
        if !self.skip_expected_token(TokenKind::Colon) {
            eprintln!("Expected ':', but found {:?}", self.peek_token());
            return None;
        }
        let ty = self.parse_type()?;
        Some((name, Rc::new(ty)))
    }

    pub fn parse_type(&mut self) -> Option<Ty> {
        let t = self.skip_token().unwrap();
        match t.kind {
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
            // str
            TokenKind::Str => Some(Ty::Str),
            // bool
            TokenKind::Bool => Some(Ty::Bool),
            // [type; n]
            TokenKind::OpenBracket => {
                let elem_ty = self.parse_type()?;
                if !self.skip_expected_token(TokenKind::Semi) {
                    eprintln!("Expected ';', but found {:?}", self.peek_token().unwrap());
                    return None;
                }
                let t = self.skip_token()?;
                let TokenKind::NumLit(n) = t.kind else {
                    return None;
                };
                if !self.skip_expected_token(TokenKind::CloseBracket) {
                    eprintln!("Expected ']', but found {:?}", self.peek_token().unwrap());
                    return None;
                }
                Some(Ty::Array(Rc::new(elem_ty), n))
            }
            TokenKind::Ident(s) => Some(Ty::Adt(s)),
            TokenKind::BinOp(lexer::BinOp::And) => {
                let t = self.peek_token().unwrap();
                let region = if let TokenKind::Lifetime(_) = t.kind {
                    let TokenKind::Lifetime(r) = self.skip_token().unwrap().kind else { unreachable!() };
                    r
                } else {
                    // FIXME: infer?
                    "static".to_string()
                };
                let referent = self.parse_type()?;
                Some(Ty::Ref(region, Rc::new(referent)))
            }
            _ => {
                eprintln!("Expected type, but found {:?}", t);
                None
            }
        }
    }
}
