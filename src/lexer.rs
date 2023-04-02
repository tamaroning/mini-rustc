use crate::span::Span;
use std::{collections::VecDeque, iter::Peekable, rc::Rc, vec::IntoIter};

#[derive(Debug)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

#[derive(Debug, PartialEq, Eq)]
pub enum TokenKind {
    // keywords
    I32,
    Str,
    Let,
    Return,
    Fn,
    Bool,
    True,
    False,
    If,
    Else,
    Struct,
    Extern,
    Unsafe,
    /// ->
    Arrow,
    /// !
    Bang,
    Eq,
    /// ;
    Semi,
    Colon,
    Comma,
    Dot,
    OpenParen,
    CloseParen,
    OpenBrace,
    CloseBracket,
    OpenBracket,
    CloseBrace,
    BinOp(BinOp),
    /// Identifier
    Ident(String),
    Lifetime(String),
    /// Number
    NumLit(u32),
    /// String literal
    StrLit(String),
    /// EOF
    Eof,
    /// Unknown character
    Unknown,
}

#[derive(Debug, PartialEq, Eq)]
pub enum BinOp {
    Plus,
    Minus,
    Star,
    Eq,
    Ne,
    Gt,
    Lt,
    And,
}

fn is_space(c: char) -> bool {
    matches!(c, ' ' | '\r' | '\n')
}

pub struct Lexer {
    token_start_pos: usize,
    current_pos: usize,
    char_stream: Peekable<IntoIter<char>>,
    buffered_tokens: VecDeque<Token>,
    src: Rc<String>,
}

impl Lexer {
    pub fn new(src: String) -> Self {
        let char_stream = src.chars().collect::<Vec<char>>().into_iter().peekable();
        Lexer {
            token_start_pos: 0,
            current_pos: 0,
            char_stream,
            buffered_tokens: VecDeque::new(),
            src: Rc::new(src),
        }
    }

    fn new_token(&mut self, kind: TokenKind) -> Token {
        let t = Token {
            kind,
            span: Span::new(self.token_start_pos, self.current_pos, Rc::clone(&self.src)),
        };
        self.token_start_pos = self.current_pos;
        t
    }

    fn peek_input(&mut self) -> Option<&char> {
        self.char_stream.peek()
    }

    fn skip_input(&mut self) -> Option<char> {
        let c = self.char_stream.next();
        if c.is_some() {
            self.current_pos += 1;
        }
        c
    }

    /*
    fn skip_input_by(&mut self, n: usize) {
        for _ in 0..n {
            self.char_stream.next();
        }
    }
    */

    fn skip_whitespaces(&mut self) {
        while let Some(c) = self.peek_input()
            && is_space(*c) {
            self.skip_input();
        }
        self.token_start_pos = self.current_pos;
    }

    /// Tokenize current token and set it to buffer
    fn tokenize(&mut self) {
        // skip whitespaces
        self.skip_whitespaces();

        let tokenize_res = if let Some(c) = self.peek_input() {
            match c {
                'A'..='Z' | 'a'..='z' | '_' => self.parse_keyword_or_ident(),
                '\'' => self.parse_lifetime(),
                '0'..='9' => self.parse_number_lit(),
                // skip comments
                '/' => {
                    // skip first '/'
                    self.skip_input().unwrap();
                    let c = self.skip_input();
                    if c == Some('/') {
                        loop {
                            let c = self.peek_input();
                            dbg!(c);
                            if matches!(c, Some('\n') | None) {
                                self.skip_input();
                                break;
                            } else {
                                self.skip_input();
                            }
                        }
                        return self.tokenize();
                    } else {
                        self.new_token(TokenKind::Unknown)
                    }
                }
                '=' => {
                    self.skip_input();
                    if self.peek_input() == Some(&'=') {
                        self.skip_input();
                        self.new_token(TokenKind::BinOp(BinOp::Eq))
                    } else {
                        self.new_token(TokenKind::Eq)
                    }
                }
                '!' => {
                    self.skip_input();
                    if self.peek_input() == Some(&'=') {
                        self.skip_input();
                        self.new_token(TokenKind::BinOp(BinOp::Ne))
                    } else {
                        self.new_token(TokenKind::Bang)
                    }
                }
                '-' => {
                    self.skip_input();
                    if self.peek_input() == Some(&'>') {
                        self.skip_input();
                        self.new_token(TokenKind::Arrow)
                    } else {
                        self.new_token(TokenKind::BinOp(BinOp::Minus))
                    }
                }
                '>' => {
                    self.skip_input();
                    self.new_token(TokenKind::BinOp(BinOp::Gt))
                }
                '<' => {
                    self.skip_input();
                    self.new_token(TokenKind::BinOp(BinOp::Lt))
                }
                '&' => {
                    self.skip_input();
                    self.new_token(TokenKind::BinOp(BinOp::And))
                }
                ';' => {
                    self.skip_input();
                    self.new_token(TokenKind::Semi)
                }
                ':' => {
                    self.skip_input();
                    self.new_token(TokenKind::Colon)
                }
                ',' => {
                    self.skip_input();
                    self.new_token(TokenKind::Comma)
                }
                '.' => {
                    self.skip_input();
                    self.new_token(TokenKind::Dot)
                }
                '(' => {
                    self.skip_input();
                    self.new_token(TokenKind::OpenParen)
                }
                ')' => {
                    self.skip_input();
                    self.new_token(TokenKind::CloseParen)
                }
                '{' => {
                    self.skip_input();
                    self.new_token(TokenKind::OpenBrace)
                }
                '}' => {
                    self.skip_input();
                    self.new_token(TokenKind::CloseBrace)
                }
                '[' => {
                    self.skip_input();
                    self.new_token(TokenKind::OpenBracket)
                }
                ']' => {
                    self.skip_input();
                    self.new_token(TokenKind::CloseBracket)
                }
                '+' => {
                    self.skip_input();
                    self.new_token(TokenKind::BinOp(BinOp::Plus))
                }
                '*' => {
                    self.skip_input();
                    self.new_token(TokenKind::BinOp(BinOp::Star))
                }
                '\"' => self.parse_string_lit(),
                // Unknown token
                _ => {
                    eprintln!("Unknwon token starting with: {:?}", c);
                    self.skip_input();
                    self.new_token(TokenKind::Unknown)
                }
            }
        } else {
            // EOF
            self.new_token(TokenKind::Eof)
        };

        // skip whitespaces
        self.skip_whitespaces();

        self.buffered_tokens.push_back(tokenize_res);
    }

    fn parse_keyword_or_ident(&mut self) -> Token {
        let mut chars = vec![];
        while let Some(c) = &self.peek_input() {
            match c {
                'A'..='Z' | 'a'..='z' | '_' | '0'..='9' => {
                    chars.push(**c);
                    self.skip_input();
                }
                _ => break,
            };
        }
        let s: String = chars.into_iter().collect();
        match s.as_str() {
            "i32" => self.new_token(TokenKind::I32),
            "str" => self.new_token(TokenKind::Str),
            "bool" => self.new_token(TokenKind::Bool),
            "true" => self.new_token(TokenKind::True),
            "false" => self.new_token(TokenKind::False),
            "let" => self.new_token(TokenKind::Let),
            "return" => self.new_token(TokenKind::Return),
            "fn" => self.new_token(TokenKind::Fn),
            "if" => self.new_token(TokenKind::If),
            "else" => self.new_token(TokenKind::Else),
            "struct" => self.new_token(TokenKind::Struct),
            "extern" => self.new_token(TokenKind::Extern),
            "unsafe" => self.new_token(TokenKind::Unsafe),
            _ => self.new_token(TokenKind::Ident(s)),
        }
    }

    fn parse_lifetime(&mut self) -> Token {
        // skip '\''
        self.skip_input();
        let mut chars = vec![];
        while let Some(c) = &self.peek_input() {
            match c {
                'A'..='Z' | 'a'..='z' | '_' | '0'..='9' => {
                    chars.push(**c);
                    self.skip_input();
                }
                _ => break,
            };
        }
        if chars.is_empty() {
            eprintln!(
                "Expected lifetime identifier, but found {:?}",
                self.peek_input()
            );
            self.new_token(TokenKind::Unknown)
        } else {
            let s: String = chars.into_iter().collect();
            self.new_token(TokenKind::Lifetime(s))
        }
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
        self.new_token(TokenKind::NumLit(n))
    }

    fn parse_string_lit(&mut self) -> Token {
        // skip '"'
        self.skip_input();

        let mut chars = vec![];
        while let Some(c) = &self.peek_input() {
            match c {
                '"' => {
                    self.skip_input();
                    break;
                }
                /*
                TODO: escape
                '\\' => {
                    self.skip_input();
                    let escp = match self.skip_input().unwrap() {
                        'n' => '\n',
                        c => {
                            eprintln!("Escape \"\\{c}\" is not supported");
                            return Err(());
                        }
                    };
                    chars.push(escp);
                }
                */
                '\n' => {
                    eprintln!("Unexpected newline in string literal");
                    return self.new_token(TokenKind::Unknown);
                }
                _ => {
                    chars.push(**c);
                    self.skip_input();
                }
            };
        }

        let s: String = chars.into_iter().collect();
        self.new_token(TokenKind::StrLit(s))
    }

    pub fn peek_token(&mut self) -> &Token {
        // do tokenize if the current token is not buffered
        if self.buffered_tokens.is_empty() {
            self.tokenize();
        }
        &self.buffered_tokens[0]
    }

    /// Skip the current token. Keep returning EOF after lexer reached EOF
    pub fn skip_token(&mut self) -> Token {
        // make sure that the current token is buffered
        if self.buffered_tokens.is_empty() {
            self.tokenize();
        }
        self.buffered_tokens.pop_front().unwrap()
    }
}

#[test]
fn test_peek() {
    let mut lexer = Lexer::new("123456".to_string());
    assert_eq!(lexer.peek_input(), Some(&'1'));
    lexer.skip_input();
    assert_eq!(lexer.peek_input(), Some(&'2'));
    lexer.skip_input();
    lexer.skip_input();
    lexer.skip_input();
    assert_eq!(lexer.peek_input(), Some(&'5'));
    assert_eq!(lexer.skip_input(), Some('5'));
    assert_eq!(lexer.skip_input(), Some('6'));
    assert_eq!(lexer.peek_input(), None);
    assert_eq!(lexer.skip_input(), None);
}

#[test]
fn test_tokenize() {
    let mut lexer = Lexer::new("123".to_string());
    assert_eq!(&lexer.peek_token().kind, &TokenKind::NumLit(123));
    let mut lexer = Lexer::new("987_654_321".to_string());
    assert_eq!(lexer.peek_token().kind, TokenKind::NumLit(987654321));
}

#[test]
fn test_lexer() {
    let mut lexer = Lexer::new("123 + 456 ".to_string());
    assert_eq!(lexer.skip_token().kind, TokenKind::NumLit(123));
    assert_eq!(lexer.skip_token().kind, TokenKind::BinOp(BinOp::Plus));
    assert_eq!(lexer.peek_token().kind, TokenKind::NumLit(456));
    let _ = lexer.skip_token();
    assert_eq!(lexer.skip_token().kind, TokenKind::Eof);
    assert_eq!(lexer.skip_token().kind, TokenKind::Eof);
    assert_eq!(lexer.skip_token().kind, TokenKind::Eof);
}

#[test]
fn test_span() {
    let mut lexer = Lexer::new("let a;".to_string());
    assert_eq!(lexer.skip_token().span.to_snippet(), "let");
    dbg!(lexer.current_pos);
    assert_eq!(lexer.skip_token().span.to_snippet(), "a");
    dbg!(lexer.current_pos);
    assert_eq!(lexer.skip_token().span.to_snippet(), ";");
    dbg!(lexer.current_pos);
}

#[test]
fn test_empty() {
    let mut lexer = Lexer::new("".to_string());
    let t = lexer.peek_token();
    assert_eq!(t.span.to_snippet(), "");
    assert_eq!(t.kind, TokenKind::Eof);
}
