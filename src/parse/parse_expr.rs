use super::Parser;
use crate::ast::{self, Expr, ExprKind, Ident, UnOp};
use crate::lexer::{self, Token, TokenKind};

pub fn is_expr_start(token: &Token) -> bool {
    matches!(
        token.kind,
        TokenKind::NumLit(_)
            | TokenKind::StrLit(_)
            | TokenKind::Ident(_)
            | TokenKind::OpenParen
            | TokenKind::OpenBrace
            | TokenKind::BinOp(lexer::BinOp::Plus | lexer::BinOp::Minus)
            | TokenKind::Return
            | TokenKind::True
            | TokenKind::False
            | TokenKind::If
            | TokenKind::Unsafe
    )
}

impl Parser {
    /// expr ::= assign
    pub fn parse_expr(&mut self) -> Option<Expr> {
        self.parse_assign()
    }

    /// ifExpr ::= "if" expr  block ("else" (block | ifExpr))?
    fn parse_if_expr(&mut self) -> Option<Expr> {
        let mut span = self.peek_token().span.clone();
        if !self.skip_expected_token(TokenKind::If) {
            eprintln!(
                "Expected \"if\", but found `{}`",
                self.peek_token().span.to_snippet()
            );
            return None;
        }
        let cond = self.parse_expr()?;

        // parse then block
        let then_block = self.parse_block()?;
        span = span.concat(&then_block.span);

        let then = Expr {
            span: then_block.span.clone(),
            kind: ExprKind::Block(then_block),
            id: self.get_next_id(),
        };
        let t = self.peek_token();
        let els = if t.kind == TokenKind::Else {
            self.skip_token();
            let t = self.peek_token();
            if t.kind == TokenKind::If {
                let elif = self.parse_if_expr()?;
                span = span.concat(&elif.span);
                Some(elif)
            } else {
                // parse else block
                let els_block = self.parse_block()?;
                span = span.concat(&els_block.span);
                Some(Expr {
                    span: els_block.span.clone(),
                    kind: ExprKind::Block(els_block),
                    id: self.get_next_id(),
                })
            }
        } else {
            None
        };

        Some(Expr {
            kind: ExprKind::If(Box::new(cond), Box::new(then), els.map(Box::new)),
            id: self.get_next_id(),
            span,
        })
    }

    /// assign ::= equality ("=" assign)?
    fn parse_assign(&mut self) -> Option<Expr> {
        let lhs = self.parse_binary_equality()?;
        let t = self.lexer.peek_token();
        if t.kind != TokenKind::Eq {
            return Some(lhs);
        }
        self.skip_token();
        let rhs = self.parse_assign()?;
        Some(Expr {
            span: lhs.span.concat(&rhs.span),
            kind: ExprKind::Assign(Box::new(lhs), Box::new(rhs)),
            id: self.get_next_id(),
        })
    }

    /// equality ::= relational (("=="|"!=") equality)?
    fn parse_binary_equality(&mut self) -> Option<Expr> {
        let lhs = self.parse_binary_relational()?;
        let t = self.lexer.peek_token();
        let binop = match t.kind {
            TokenKind::BinOp(lexer::BinOp::Eq) => ast::BinOp::Eq,
            TokenKind::BinOp(lexer::BinOp::Ne) => ast::BinOp::Ne,
            _ => {
                return Some(lhs);
            }
        };
        self.lexer.skip_token();

        let rhs = self.parse_binary_equality()?;

        Some(Expr {
            span: lhs.span.concat(&rhs.span),
            kind: ExprKind::Binary(binop, Box::new(lhs), Box::new(rhs)),
            id: self.get_next_id(),
        })
    }

    /// relational ::= add (("=="|"!=") relational)?
    fn parse_binary_relational(&mut self) -> Option<Expr> {
        let lhs = self.parse_binary_add()?;
        let t = self.lexer.peek_token();
        let binop = match t.kind {
            TokenKind::BinOp(lexer::BinOp::Lt) => ast::BinOp::Lt,
            TokenKind::BinOp(lexer::BinOp::Gt) => ast::BinOp::Gt,
            _ => {
                return Some(lhs);
            }
        };
        self.lexer.skip_token();

        let rhs = self.parse_binary_relational()?;

        Some(Expr {
            span: lhs.span.concat(&rhs.span),
            kind: ExprKind::Binary(binop, Box::new(lhs), Box::new(rhs)),
            id: self.get_next_id(),
        })
    }

    /// add ::= mul (("+"|"-") add)?
    fn parse_binary_add(&mut self) -> Option<Expr> {
        let lhs = self.parse_binary_mul()?;
        let t = self.lexer.peek_token();
        let binop = match t.kind {
            TokenKind::BinOp(lexer::BinOp::Plus) => ast::BinOp::Add,
            TokenKind::BinOp(lexer::BinOp::Minus) => ast::BinOp::Sub,
            _ => {
                return Some(lhs);
            }
        };
        self.lexer.skip_token();

        let rhs = self.parse_binary_add()?;

        Some(Expr {
            span: lhs.span.concat(&rhs.span),
            kind: ExprKind::Binary(binop, Box::new(lhs), Box::new(rhs)),
            id: self.get_next_id(),
        })
    }

    /// mul ::= unary ("*" mul)?
    fn parse_binary_mul(&mut self) -> Option<Expr> {
        let lhs = self.parse_binary_unary()?;
        let t = self.lexer.peek_token();
        let binop = match t.kind {
            TokenKind::BinOp(lexer::BinOp::Star) => ast::BinOp::Mul,
            _ => {
                return Some(lhs);
            }
        };
        self.lexer.skip_token();

        let rhs = self.parse_binary_mul()?;

        Some(Expr {
            span: lhs.span.concat(&rhs.span),
            kind: ExprKind::Binary(binop, Box::new(lhs), Box::new(rhs)),
            id: self.get_next_id(),
        })
    }

    /// unary ::= ("+"|"-")? primary
    fn parse_binary_unary(&mut self) -> Option<Expr> {
        let span = self.peek_token().span.clone();
        let t = self.lexer.peek_token();
        let unup = match &t.kind {
            TokenKind::BinOp(lexer::BinOp::Plus) => UnOp::Plus,
            TokenKind::BinOp(lexer::BinOp::Minus) => UnOp::Minus,
            _ => {
                return self.parse_binary_primary();
            }
        };
        // skip unary op token
        self.skip_token();
        // parse primary
        let primary = self.parse_binary_primary()?;

        Some(Expr {
            span: span.concat(&primary.span),
            kind: ExprKind::Unary(unup, Box::new(primary)),
            id: self.get_next_id(),
        })
    }

    /// primary ::= num | true | false | stringLit
    ///     | ident | callExpr | indexExpr | ifExpr
    ///     | returnExpr | "(" expr ")"
    ///     | unsafeBlock | block
    ///     | fieldExpr | structExpr
    /// returnExpr ::= "return" expr
    fn parse_binary_primary(&mut self) -> Option<Expr> {
        let t = &self.lexer.peek_token();
        let mut expr = match t.kind {
            TokenKind::NumLit(n) => {
                let span = self.skip_token().span;
                Expr {
                    kind: ExprKind::NumLit(n),
                    id: self.get_next_id(),
                    span,
                }
            }
            TokenKind::True => {
                let span = self.skip_token().span;
                Expr {
                    kind: ExprKind::BoolLit(true),
                    id: self.get_next_id(),
                    span,
                }
            }
            TokenKind::False => {
                let span = self.skip_token().span;
                Expr {
                    kind: ExprKind::BoolLit(false),
                    id: self.get_next_id(),
                    span,
                }
            }
            TokenKind::StrLit(_) => {
                let t = self.skip_token();
                let TokenKind::StrLit(s) = t.kind else { unreachable!() };
                Expr {
                    kind: ExprKind::StrLit(s),
                    id: self.get_next_id(),
                    span: t.span,
                }
            }
            TokenKind::If => self.parse_if_expr()?,
            TokenKind::Return => {
                // TODO: parse `return;`
                let span = self.skip_token().span;
                let e = self.parse_expr()?;
                Expr {
                    span: span.concat(&e.span),
                    kind: ExprKind::Return(Box::new(e)),
                    id: self.get_next_id(),
                }
            }
            // FIXME: ambiguity: parser cannot decide ident or struct expr.
            // e.g. `if s { } else {}`
            TokenKind::Ident(_) => self.parse_ident_or_struct_expr()?,
            TokenKind::OpenParen => {
                let mut span = self.peek_token().span.clone();
                // skip '('
                self.skip_token();
                let t = self.peek_token();
                if t.kind == TokenKind::CloseParen {
                    // skip ')'
                    let t = self.skip_token();
                    span = span.concat(&t.span);
                    Expr {
                        kind: ExprKind::Unit,
                        id: self.get_next_id(),
                        span,
                    }
                } else {
                    let expr = self.parse_expr()?;
                    span = span.concat(&self.peek_token().span);
                    // skip ')'
                    if !self.skip_expected_token(TokenKind::CloseParen) {
                        eprintln!(
                            "Expected ')', but found `{}`",
                            self.peek_token().span.to_snippet()
                        );
                        return None;
                    }
                    // just expand span
                    Expr {
                        kind: expr.kind,
                        span,
                        id: expr.id,
                    }
                }
            }
            // unsafe block expression
            // TODO: Should AST node have `unsafe` info?
            TokenKind::Unsafe => {
                // skip "unsafe"
                let unsafe_span = self.skip_token().span;
                let block = self.parse_block()?;
                Expr {
                    span: unsafe_span.concat(&block.span),
                    kind: ExprKind::Block(block),
                    id: self.get_next_id(),
                }
            }
            // block expression
            TokenKind::OpenBrace => {
                let block = self.parse_block()?;
                Expr {
                    span: block.span.clone(),
                    kind: ExprKind::Block(block),
                    id: self.get_next_id(),
                }
            }
            _ => {
                eprintln!(
                    "Expected num or (expr), but found `{}`",
                    t.span.to_snippet()
                );
                return None;
            }
        };
        // deal with tailing `(...)` (func call), `[...]` (indexing), .ident (field access)
        // FIXME: disambiguity: () () => FuncCall or ExprStmt ExprStmt
        loop {
            let t = self.peek_token();
            match &t.kind {
                TokenKind::OpenParen => {
                    expr = self.parse_call_expr(expr)?;
                }
                TokenKind::OpenBracket => expr = self.parse_index_expr(expr)?,
                TokenKind::Dot => expr = self.parse_field_expr(expr)?,
                _ => break,
            }
        }
        Some(expr)
    }

    /// ident | structExpr
    fn parse_ident_or_struct_expr(&mut self) -> Option<Expr> {
        let ident = self.parse_ident().unwrap();
        let t = self.peek_token();
        if let TokenKind::OpenBrace = t.kind {
            self.parse_struct_expr(ident)
        } else {
            Some(Expr {
                span: ident.span.clone(),
                kind: ExprKind::Ident(ident),
                id: self.get_next_id(),
            })
        }
    }

    /// structExpr ::= ident "{" structExprFields? "}"
    /// NOTE: first ident is already parsed
    fn parse_struct_expr(&mut self, ident: Ident) -> Option<Expr> {
        let mut span = self.peek_token().span.clone();

        if !self.skip_expected_token(TokenKind::OpenBrace) {
            eprintln!(
                "Expected '{{ for struct expr', but found `{}`",
                self.peek_token().span.to_snippet()
            );
            return None;
        }

        let fields = if matches!(self.peek_token().kind, TokenKind::Ident(_)) {
            self.parse_struct_expr_fields()?
        } else {
            vec![]
        };

        span = span.concat(&self.peek_token().span);
        if !self.skip_expected_token(TokenKind::CloseBrace) {
            eprintln!(
                "Expected '}}' for struct expr, but found `{}`",
                self.peek_token().span.to_snippet()
            );
            return None;
        }
        Some(Expr {
            kind: ExprKind::Struct(ident, fields),
            id: self.get_next_id(),
            span,
        })
    }

    /// structExprFields ::= structExprField ("," structExprField)* ","?
    fn parse_struct_expr_fields(&mut self) -> Option<Vec<(Ident, Box<Expr>)>> {
        let mut fds = vec![];
        fds.push(self.parse_struct_expr_field()?);

        while matches!(self.peek_token().kind, TokenKind::Comma) {
            self.skip_token();
            if matches!(self.peek_token().kind, TokenKind::Ident(_)) {
                fds.push(self.parse_struct_expr_field()?);
            }
        }
        Some(fds)
    }

    /// structExprField ::= ident ":" expr
    fn parse_struct_expr_field(&mut self) -> Option<(Ident, Box<Expr>)> {
        let ident = self.parse_ident()?;
        if !self.skip_expected_token(TokenKind::Colon) {
            eprintln!(
                "Expected ':', but found `{}`",
                self.peek_token().span.to_snippet()
            );
            return None;
        }
        let expr = self.parse_expr()?;
        Some((ident, Box::new(expr)))
    }

    /// callExpr ::= primary "(" callParams? ")"
    /// NOTE: first primary is already parsed
    fn parse_call_expr(&mut self, fn_expr: Expr) -> Option<Expr> {
        let mut span = fn_expr.span.clone();

        // skip '('
        self.skip_token();
        let args = if self.peek_token().kind == TokenKind::CloseParen {
            vec![]
        } else {
            self.parse_call_params()?
        };

        span = span.concat(&self.peek_token().span);
        if !self.skip_expected_token(TokenKind::CloseParen) {
            eprintln!(
                "Expected ')', but found `{}`",
                self.peek_token().span.to_snippet()
            );
            return None;
        }
        Some(Expr {
            kind: ExprKind::Call(Box::new(fn_expr), args),
            id: self.get_next_id(),
            span,
        })
    }

    /// callParams ::= callParam ("," callParam)* ","?
    /// callParam = expr
    fn parse_call_params(&mut self) -> Option<Vec<Expr>> {
        let mut args = vec![];
        args.push(self.parse_expr()?);

        while matches!(self.peek_token().kind, TokenKind::Comma) {
            self.skip_token();
            if is_expr_start(self.peek_token()) {
                args.push(self.parse_expr()?);
            }
        }
        Some(args)
    }

    /// indexExpr ::= priamry "[" expr "]"
    /// NOTE: first primary is already parsed
    fn parse_index_expr(&mut self, array_expr: Expr) -> Option<Expr> {
        let mut span = array_expr.span.clone();

        // skip '['
        if !self.skip_expected_token(TokenKind::OpenBracket) {
            eprintln!(
                "Expected '[', but found `{}`",
                self.peek_token().span.to_snippet()
            );
            return None;
        }
        let index = self.parse_expr()?;

        span = span.concat(&self.peek_token().span);
        // skip ']'
        if !self.skip_expected_token(TokenKind::CloseBracket) {
            eprintln!(
                "Expected ']', but found `{}`",
                self.peek_token().span.to_snippet()
            );
            return None;
        }
        Some(Expr {
            kind: ExprKind::Index(Box::new(array_expr), Box::new(index)),
            id: self.get_next_id(),
            span,
        })
    }

    /// fieldExpr ::= primary "(" callParams? ")"
    /// NOTE: first primary is already parsed
    fn parse_field_expr(&mut self, recv: Expr) -> Option<Expr> {
        let mut span = recv.span.clone();

        // skip '.'
        self.skip_token();
        let fd = self.parse_ident()?;

        span = span.concat(&fd.span);
        Some(Expr {
            kind: ExprKind::Field(Box::new(recv), fd),
            id: self.get_next_id(),
            span,
        })
    }
}
