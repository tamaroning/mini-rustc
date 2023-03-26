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
            | TokenKind::BinOp(lexer::BinOp::Plus | lexer::BinOp::Minus)
    )
}

impl Parser {
    pub fn parse_expr(&mut self) -> Option<Expr> {
        self.parse_assign()
    }

    /// assign ::= binary ("=" assign)?
    fn parse_assign(&mut self) -> Option<Expr> {
        let Some(lhs) = self.parse_binary() else {
            return None;
        };
        let t = self.lexer.peek_token().unwrap();
        if t.kind != TokenKind::Eq {
            return Some(lhs);
        }
        self.skip_token();
        let Some(rhs) = self.parse_assign() else {
            return None;
        };
        Some(Expr {
            kind: ExprKind::Assign(Box::new(lhs), Box::new(rhs)),
            id: self.get_next_id(),
        })
    }

    /// binary ::= add
    fn parse_binary(&mut self) -> Option<Expr> {
        self.parse_binary_add()
    }

    /// add ::= mul ("+"|"-") add
    fn parse_binary_add(&mut self) -> Option<Expr> {
        let Some(lhs) = self.parse_binary_mul() else {
            return None;
        };

        let Some(t) = self.lexer.peek_token() else {
            return None;
        };
        let binop = match t.kind {
            TokenKind::BinOp(lexer::BinOp::Plus) => ast::BinOp::Add,
            TokenKind::BinOp(lexer::BinOp::Minus) => ast::BinOp::Sub,
            _ => {
                return Some(lhs);
            }
        };
        self.lexer.skip_token();

        let Some(rhs) = self.parse_binary_add() else {
            return None;
        };

        Some(Expr {
            kind: ExprKind::Binary(binop, Box::new(lhs), Box::new(rhs)),
            id: self.get_next_id(),
        })
    }

    /// mul ::= unary "*" mul
    fn parse_binary_mul(&mut self) -> Option<Expr> {
        let Some(lhs) = self.parse_binary_unary() else {
            return None;
        };

        let Some(t) = self.lexer.peek_token() else {
            return None;
        };
        let binop = match t.kind {
            TokenKind::BinOp(lexer::BinOp::Star) => ast::BinOp::Mul,
            _ => {
                return Some(lhs);
            }
        };
        self.lexer.skip_token();

        let Some(rhs) = self.parse_binary_mul() else {
            return None;
        };

        Some(Expr {
            kind: ExprKind::Binary(binop, Box::new(lhs), Box::new(rhs)),
            id: self.get_next_id(),
        })
    }

    /// unary ::= ("+"|"-") primary
    fn parse_binary_unary(&mut self) -> Option<Expr> {
        let Some(t) = self.lexer.peek_token() else {
            return None;
        };

        let unup = match &t.kind {
            TokenKind::BinOp(lexer::BinOp::Plus) => UnOp::Plus,
            TokenKind::BinOp(lexer::BinOp::Minus) => UnOp::Minus,
            _ => {
                return self.parse_binary_primary();
            }
        };
        // skip unary op token
        self.skip_token();

        let Some(primary) = self.parse_binary_primary() else {
            return None;
        };
        Some(Expr {
            kind: ExprKind::Unary(unup, Box::new(primary)),
            id: self.get_next_id(),
        })
    }

    /// primary ::= num | ident | "(" expr ")"
    fn parse_binary_primary(&mut self) -> Option<Expr> {
        let Some(t) = self.lexer.skip_token() else {
            return None;
        };
        match t.kind {
            TokenKind::NumLit(n) => Some(Expr {
                kind: ExprKind::NumLit(n),
                id: self.get_next_id(),
            }),
            TokenKind::Ident(symbol) => Some(Expr {
                kind: ExprKind::Ident(Ident { symbol }),
                id: self.get_next_id(),
            }),
            TokenKind::OpenParen => {
                let Some(expr) = self.parse_expr() else {
                    return None;
                };
                if !self.skip_expected_token(TokenKind::CloseParen) {
                    eprintln!("Expected ')', but found {:?}", self.peek_token());
                    return None;
                }
                Some(expr)
            }
            _ => {
                eprintln!("Expected num or (expr), but found {:?}", t);
                None
            }
        }
    }
}
