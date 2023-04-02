use super::Parser;
use crate::ast::{ExternBlock, Func, Ident, Item, ItemKind, StructItem};
use crate::lexer::{self, Token, TokenKind};
use crate::middle::ty::Ty;
use std::rc::Rc;

pub fn is_item_start(token: &Token) -> bool {
    matches!(
        token.kind,
        TokenKind::Fn | TokenKind::Extern | TokenKind::Struct
    )
}

impl Parser {
    /// item ::= func | structItem | externBlock
    pub fn parse_item(&mut self) -> Option<Item> {
        let t = self.peek_token();
        match &t.kind {
            TokenKind::Fn => Some(Item {
                kind: ItemKind::Func(self.parse_func(None)?),
            }),
            TokenKind::Struct => Some(Item {
                kind: ItemKind::Struct(self.parse_struct_item()?),
            }),
            TokenKind::Extern => Some(Item {
                kind: ItemKind::ExternBlock(self.parse_extern_block()?),
            }),
            _ => {
                eprintln!("Expected item, but found {:?}", self.peek_token());
                None
            }
        }
    }

    /// externBlock ::= "extern" abi "{" externalItem* "}"
    /// abi ::= "\"C\""
    /// https://doc.rust-lang.org/reference/items/external-blocks.html
    fn parse_extern_block(&mut self) -> Option<ExternBlock> {
        // skip `extern`
        self.skip_token();
        // parse ABI
        let t = self.skip_token();
        let abi = if let TokenKind::StrLit(s) = t.kind {
            s
        } else {
            eprintln!("Expected extern ABI, but found {:?}", t);
            return None;
        };
        // check if ABI is "C"
        if abi != "C" {
            eprintln!(
                "Found `extern {}`, but `extern \"C\"` can only be supported",
                abi
            );
            return None;
        }

        if !self.skip_expected_token(TokenKind::OpenBrace) {
            eprintln!("Expected '{{', but found {:?}", self.peek_token());
            return None;
        }

        let mut funcs = vec![];
        while self.peek_token().kind == TokenKind::Fn {
            funcs.push(self.parse_func(Some(abi.clone()))?);
        }

        if !self.skip_expected_token(TokenKind::CloseBrace) {
            eprintln!(
                "Expected '}}' or external item, but found {:?}",
                self.peek_token()
            );
            return None;
        }

        Some(ExternBlock { funcs })
    }

    /// func ::= "fn" ident "(" funcParams? ")" "->" "i32" (block | ";")
    /// https://doc.rust-lang.org/reference/items/functions.html
    pub fn parse_func(&mut self, ext: Option<String>) -> Option<Func> {
        if !self.skip_expected_token(TokenKind::Fn) {
            eprintln!("Expected \"fn\", but found {:?}", self.peek_token());
            return None;
        }
        let name = self.parse_ident()?;
        if !self.skip_expected_token(TokenKind::OpenParen) {
            eprintln!("Expected '(', but found {:?}", self.peek_token());
            return None;
        }
        let t = self.peek_token();
        let params = if t.kind == TokenKind::CloseParen {
            vec![]
        } else {
            self.parse_func_params()?
        };
        if !self.skip_expected_token(TokenKind::CloseParen) {
            eprintln!("Expected ')', but found {:?}", self.peek_token());
            return None;
        }

        if !self.skip_expected_token(TokenKind::Arrow) {
            eprintln!("Expected '->', but found {:?}", self.peek_token());
            return None;
        }
        let ret_ty = self.parse_type()?;

        let t = self.peek_token();
        let body = if t.kind == TokenKind::OpenBrace {
            Some(self.parse_block()?)
        } else if t.kind == TokenKind::Semi {
            self.skip_token();
            None
        } else {
            eprintln!("Expected function body or ';', but found {:?}", t);
            return None;
        };

        Some(Func {
            name,
            params,
            ret_ty: Rc::new(ret_ty),
            ext,
            body,
            id: self.get_next_id(),
        })
    }

    /// funcParams ::= funcParam ("," funcParam)* ","?
    /// funcParam ::= ident ":" type
    fn parse_func_params(&mut self) -> Option<Vec<(Ident, Rc<Ty>)>> {
        let mut params = vec![];
        params.push(self.parse_func_param()?);

        while matches!(self.peek_token().kind, TokenKind::Comma) {
            self.skip_token();
            if matches!(self.peek_token().kind, TokenKind::Ident(_)) {
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

        let fields = if matches!(self.peek_token().kind, TokenKind::Ident(_)) {
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

        while matches!(self.peek_token().kind, TokenKind::Comma) {
            self.skip_token();
            if matches!(self.peek_token().kind, TokenKind::Ident(_)) {
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
        let t = self.skip_token();
        match t.kind {
            // Unit type: ()
            TokenKind::OpenParen => {
                if !self.skip_expected_token(TokenKind::CloseParen) {
                    eprintln!("Expected ')', but found {:?}", self.peek_token());
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
                    eprintln!("Expected ';', but found {:?}", self.peek_token());
                    return None;
                }
                let t = self.skip_token();
                let TokenKind::NumLit(n) = t.kind else {
                    return None;
                };
                if !self.skip_expected_token(TokenKind::CloseBracket) {
                    eprintln!("Expected ']', but found {:?}", self.peek_token());
                    return None;
                }
                Some(Ty::Array(Rc::new(elem_ty), n))
            }
            TokenKind::Ident(s) => Some(Ty::Adt(s)),
            TokenKind::BinOp(lexer::BinOp::And) => {
                let t = self.peek_token();
                let region = if let TokenKind::Lifetime(_) = t.kind {
                    let TokenKind::Lifetime(r) = self.skip_token().kind else { unreachable!() };
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
