use crate::ast::{self, Expr, ExprKind};
use crate::lexer::{self, Lexer, Token, TokenKind};

pub struct Parser {
    lexer: Lexer,
}

impl Parser {
    pub fn new(lexer: Lexer) -> Self {
        Parser { lexer }
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
            },
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

    pub fn parse_crate(&mut self) -> Option<Expr> {
        let expr = self.parse_expr();
        if !self.skip_expected_token(TokenKind::Eof) {
            return None;
        }
        expr
    }

    fn parse_expr(&mut self) -> Option<Expr> {
        let Some(t) = self.lexer.peek_token() else {
            return None;
        };

        match t.kind {
            TokenKind::NumLit(_) => self.parse_binary(),
            _ => None,
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
        let primary = self.parse_binary_primary();
        primary
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
                self.skip_token();
                let Some(expr) = self.parse_expr() else {
                    return None;
                };
                if !self.skip_expected_token(TokenKind::CloseParen) {
                    eprintln!("Expected ')', but found {:?}", self.peek_token());
                    return None;
                }
                Some(expr)
            }
            t => {
                eprintln!("Expected num or (expr), but found {:?}", t);
                None
            }
        }
    }
}
