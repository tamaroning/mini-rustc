use crate::{ast::{Expr, ExprKind, self, UnOp}, lexer::{TokenKind, self}};
use super::Parser;

impl Parser {
    pub fn parse_expr(&mut self) -> Option<Expr> {
        let Some(t) = self.lexer.peek_token() else {
            return None;
        };

        match t.kind {
            TokenKind::NumLit(_)
            | TokenKind::OpenParen
            | TokenKind::BinOp(lexer::BinOp::Plus | lexer::BinOp::Minus) => self.parse_binary(),
            _ => {
                eprintln!("Expected expr, but found {:?}", t);
                None
            }
        }
    }

    // binary ::= add
    fn parse_binary(&mut self) -> Option<Expr> {
        self.parse_binary_add()
    }

    // add ::= mul ("+"|"-") add
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
        })
    }

    // mul ::= unary "*" mul
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
        })
    }

    // unary ::= ("+"|"-") primary
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
        })
    }

    // primary ::= num | "(" expr ")"
    fn parse_binary_primary(&mut self) -> Option<Expr> {
        let Some(t) = self.lexer.skip_token() else {
            return None;
        };
        match t.kind {
            TokenKind::NumLit(n) => Some(Expr {
                kind: ExprKind::NumLit(n),
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
