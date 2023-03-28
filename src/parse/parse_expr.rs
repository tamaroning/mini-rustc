use super::Parser;
use crate::{
    ast::{self, Expr, ExprKind, Ident, UnOp},
    lexer::{self, Token, TokenKind},
};

pub fn is_expr_start(token: &Token) -> bool {
    matches!(
        token.kind,
        TokenKind::NumLit(_)
            | TokenKind::Ident(_)
            | TokenKind::OpenParen
            | TokenKind::OpenBrace
            | TokenKind::BinOp(lexer::BinOp::Plus | lexer::BinOp::Minus)
            | TokenKind::Return
            | TokenKind::True
            | TokenKind::False
    )
}

impl Parser {
    /// expr ::= "return" expr | assign
    pub fn parse_expr(&mut self) -> Option<Expr> {
        let t = self.peek_token()?;
        match &t.kind {
            TokenKind::Return => {
                self.skip_token();
                let e = self.parse_expr()?;
                Some(Expr {
                    kind: ExprKind::Return(Box::new(e)),
                    id: self.get_next_id(),
                })
            }
            _ => self.parse_assign(),
        }
    }

    /// assign ::= equality ("=" assign)?
    fn parse_assign(&mut self) -> Option<Expr> {
        let lhs = self.parse_binary_equality()?;
        let t = self.lexer.peek_token()?;
        if t.kind != TokenKind::Eq {
            return Some(lhs);
        }
        self.skip_token();
        let rhs = self.parse_assign()?;
        Some(Expr {
            kind: ExprKind::Assign(Box::new(lhs), Box::new(rhs)),
            id: self.get_next_id(),
        })
    }

    /// equality ::= relational (("=="|"!=") equality)?
    fn parse_binary_equality(&mut self) -> Option<Expr> {
        let lhs = self.parse_binary_relational()?;
        let t = self.lexer.peek_token()?;
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
            kind: ExprKind::Binary(binop, Box::new(lhs), Box::new(rhs)),
            id: self.get_next_id(),
        })
    }

    /// relational ::= add (("=="|"!=") relational)?
    fn parse_binary_relational(&mut self) -> Option<Expr> {
        let lhs = self.parse_binary_add()?;
        let t = self.lexer.peek_token()?;
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
            kind: ExprKind::Binary(binop, Box::new(lhs), Box::new(rhs)),
            id: self.get_next_id(),
        })
    }

    /// add ::= mul ("+"|"-") add
    fn parse_binary_add(&mut self) -> Option<Expr> {
        let lhs = self.parse_binary_mul()?;
        let t = self.lexer.peek_token()?;
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
            kind: ExprKind::Binary(binop, Box::new(lhs), Box::new(rhs)),
            id: self.get_next_id(),
        })
    }

    /// mul ::= unary "*" mul
    fn parse_binary_mul(&mut self) -> Option<Expr> {
        let lhs = self.parse_binary_unary()?;
        let t = self.lexer.peek_token()?;
        let binop = match t.kind {
            TokenKind::BinOp(lexer::BinOp::Star) => ast::BinOp::Mul,
            _ => {
                return Some(lhs);
            }
        };
        self.lexer.skip_token();

        let rhs = self.parse_binary_mul()?;

        Some(Expr {
            kind: ExprKind::Binary(binop, Box::new(lhs), Box::new(rhs)),
            id: self.get_next_id(),
        })
    }

    /// unary ::= ("+"|"-") primary
    fn parse_binary_unary(&mut self) -> Option<Expr> {
        let t = self.lexer.peek_token()?;
        let unup = match &t.kind {
            TokenKind::BinOp(lexer::BinOp::Plus) => UnOp::Plus,
            TokenKind::BinOp(lexer::BinOp::Minus) => UnOp::Minus,
            _ => {
                return self.parse_binary_primary();
            }
        };
        // skip unary op token
        self.skip_token();

        let primary = self.parse_binary_primary()?;
        Some(Expr {
            kind: ExprKind::Unary(unup, Box::new(primary)),
            id: self.get_next_id(),
        })
    }

    /// primary ::= num | true | false | ident ("(" ")")? | "(" expr ")" | block
    fn parse_binary_primary(&mut self) -> Option<Expr> {
        let t = self.lexer.peek_token()?;
        match t.kind {
            TokenKind::NumLit(n) => {
                self.skip_token();
                Some(Expr {
                    kind: ExprKind::NumLit(n),
                    id: self.get_next_id(),
                })
            }
            TokenKind::True => {
                self.skip_token();
                Some(Expr {
                    kind: ExprKind::BoolLit(true),
                    id: self.get_next_id(),
                })
            }
            TokenKind::False => {
                self.skip_token();
                Some(Expr {
                    kind: ExprKind::BoolLit(false),
                    id: self.get_next_id(),
                })
            }
            TokenKind::Ident(_) => {
                let TokenKind::Ident(symbol) = self.skip_token()?.kind else {
                    unreachable!();
                };
                let t = self.peek_token()?;
                if t.kind == TokenKind::OpenParen {
                    self.skip_token();
                    self.skip_expected_token(TokenKind::CloseParen);
                    Some(Expr {
                        kind: ExprKind::Call(Ident { symbol }),
                        id: self.get_next_id(),
                    })
                } else {
                    Some(Expr {
                        kind: ExprKind::Ident(Ident { symbol }),
                        id: self.get_next_id(),
                    })
                }
            }
            TokenKind::OpenParen => {
                self.skip_token();
                let expr = self.parse_expr()?;
                if !self.skip_expected_token(TokenKind::CloseParen) {
                    eprintln!("Expected ')', but found {:?}", self.peek_token());
                    return None;
                }
                Some(expr)
            }
            TokenKind::OpenBrace => Some(Expr {
                kind: ExprKind::Block(self.parse_block()?),
                id: self.get_next_id(),
            }),
            _ => {
                eprintln!("Expected num or (expr), but found {:?}", t);
                None
            }
        }
    }
}
