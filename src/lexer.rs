use std::{collections::VecDeque, iter::Peekable, vec::IntoIter};

#[derive(Debug, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
}

impl Token {
    fn new(kind: TokenKind) -> Self {
        Token { kind }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum TokenKind {
    OpenParen,
    CloseParen,
    BinOp(BinOp),
    NumLit(u32),
    Eof,
}

#[derive(Debug, PartialEq, Eq)]
pub enum BinOp {
    Plus,
    Minus,
    Star,
}

fn is_space(c: char) -> bool {
    c == ' '
}

pub struct Lexer {
    char_stream: Peekable<IntoIter<char>>,
    buffered_tokens: VecDeque<Result<Token, ()>>,
}

impl Lexer {
    pub fn new(source: &str) -> Self {
        let char_stream = source.chars().collect::<Vec<char>>().into_iter().peekable();
        Lexer {
            char_stream,
            buffered_tokens: VecDeque::new(),
        }
    }

    fn peek_input(&mut self) -> Option<&char> {
        self.char_stream.peek()
    }

    fn skip_input(&mut self) -> Option<char> {
        self.char_stream.next()
    }

    fn skip_input_by(&mut self, n: usize) {
        for _ in 0..n {
            self.char_stream.next();
        }
    }

    fn skip_whitespaces(&mut self) {
        while let Some(c) = self.peek_input()
            && is_space(*c) {
            self.skip_input();
        }
    }

    /// Tokenize current token and set it to buffer
    fn tokenize(&mut self) {
        // skip whitespaces
        self.skip_whitespaces();

        let tokenize_res = if let Some(c) = self.peek_input() {
            match c {
                '0'..='9' => Ok(self.parse_number_lit()),
                '(' => {
                    self.skip_input();
                    Ok(Token::new(TokenKind::OpenParen))
                },
                ')' => {
                    self.skip_input();
                    Ok(Token::new(TokenKind::CloseParen))
                },
                '+' => {
                    self.skip_input();
                    Ok(Token::new(TokenKind::BinOp(BinOp::Plus)))
                }
                '-' => {
                    self.skip_input();
                    Ok(Token::new(TokenKind::BinOp(BinOp::Minus)))
                }
                '*' => {
                    self.skip_input();
                    Ok(Token::new(TokenKind::BinOp(BinOp::Star)))
                }
                // Unknown token
                _ => Err(()),
            }
        } else {
            // EOF
            Ok(Token::new(TokenKind::Eof))
        };

        // skip whitespaces
        self.skip_whitespaces();

        self.buffered_tokens.push_back(tokenize_res);
    }

    fn parse_number_lit(&mut self) -> Token {
        let mut chars = vec![];
        while let Some(c) = &self.peek_input() {
            match c {
                '0'..='9' => {
                    chars.push(**c);
                    self.skip_input();
                }
                '_' => {
                    self.skip_input();
                    continue;
                }
                _ => break,
            };
        }

        let s: String = chars.into_iter().collect();
        let n = s.parse::<u32>().unwrap();
        Token {
            kind: TokenKind::NumLit(n),
        }
    }

    pub fn peek_token(&mut self) -> Option<&Token> {
        // do tokenize if the current token is not buffered
        if self.buffered_tokens.len() < 1 {
            self.tokenize();
        }
        match &self.buffered_tokens[0] {
            Ok(t) => Some(t),
            Err(()) => None,
        }
    }

    /// Skip the current token. Keep returning EOF after lexer reached EOF
    pub fn skip_token(&mut self) -> Option<Token> {
        // make sure that the current token is buffered
        if self.buffered_tokens.len() < 1 {
            self.tokenize();
        }
        match self.buffered_tokens.pop_front() {
            Some(Ok(t)) => Some(t),
            _ => None,
        }
    }
}

#[test]
fn test_peek() {
    let mut lexer = Lexer::new("123456");
    assert_eq!(lexer.peek_input(), Some(&'1'));
    lexer.skip_input();
    assert_eq!(lexer.peek_input(), Some(&'2'));
    lexer.skip_input_by(3);
    assert_eq!(lexer.peek_input(), Some(&'5'));
    assert_eq!(lexer.skip_input(), Some('5'));
    assert_eq!(lexer.skip_input(), Some('6'));
    assert_eq!(lexer.peek_input(), None);
    assert_eq!(lexer.skip_input(), None);
}

#[test]
fn test_tokenize() {
    let mut lexer = Lexer::new("123");
    assert_eq!(
        lexer.peek_token(),
        Some(&Token {
            kind: TokenKind::NumLit(123)
        })
    );
    let mut lexer = Lexer::new("987_654_321");
    assert_eq!(
        lexer.peek_token(),
        Some(&Token {
            kind: TokenKind::NumLit(987654321)
        })
    );
}

#[test]
fn test_parser_func() {
    let mut lexer = Lexer::new("123 + 456 ");
    assert_eq!(
        lexer.skip_token(),
        Some(Token {
            kind: TokenKind::NumLit(123)
        })
    );
    assert_eq!(
        lexer.skip_token(),
        Some(Token {
            kind: TokenKind::BinOp(BinOp::Plus)
        })
    );
    assert_eq!(
        lexer.peek_token(),
        Some(&Token {
            kind: TokenKind::NumLit(456)
        })
    );
    let _ = lexer.skip_token();
    assert_eq!(
        lexer.skip_token(),
        Some(Token {
            kind: TokenKind::Eof
        })
    );
    assert_eq!(
        lexer.skip_token(),
        Some(Token {
            kind: TokenKind::Eof
        })
    );
    assert_eq!(
        lexer.skip_token(),
        Some(Token {
            kind: TokenKind::Eof
        })
    );
}
