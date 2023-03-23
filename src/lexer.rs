use std::{iter::Peekable, vec::IntoIter};

#[derive(Debug, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
}

#[derive(Debug, PartialEq, Eq)]
pub enum TokenKind {
    NumLit(u32),
}

fn is_space(c: char) -> bool {
    c == ' '
}

pub struct Lexer {
    char_stream: Peekable<IntoIter<char>>,
}

impl Lexer {
    pub fn new(source: &str) -> Self {
        let char_stream = source.chars().collect::<Vec<char>>().into_iter().peekable();
        Lexer { char_stream }
    }

    fn peek_input(&mut self) -> Option<&char> {
        self.char_stream.peek()
    }

    fn skip(&mut self) -> Option<char> {
        self.char_stream.next()
    }

    fn skip_by(&mut self, n: usize) {
        for _ in 0..n {
            self.char_stream.next();
        }
    }

    /*
    fn at_eof(&mut self) -> bool {
        self.peek_input() == None
    }
    */

    pub fn tokenize(&mut self) -> Result<Token, ()> {
        // skip whitespaces
        while let Some(c) = self.peek_input()
            && is_space(*c) {
            self.skip();
        }

        if let Some(c) = self.peek_input() {
            match c {
                '0'..='9' => Ok(self.parse_number_lit()),
                // Unknown token
                _ => Err(()),
            }
            //Ok(Some(Token {kind: TokenKind::NumLit(0)}))
        } else {
            // EOF
            // Try to tokenize EOF
            Err(())
        }
    }

    fn parse_number_lit(&mut self) -> Token {
        let mut chars = vec![];
        while let Some(c) = &self.peek_input() {
            match c {
                '0'..='9' => {
                    chars.push(**c);
                    self.skip();
                }
                '_' => {
                    self.skip();
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
}

#[test]
fn test_peek() {
    let mut lexer = Lexer::new("123456");
    assert_eq!(lexer.peek_input(), Some(&'1'));
    lexer.skip();
    assert_eq!(lexer.peek_input(), Some(&'2'));
    lexer.skip_by(3);
    assert_eq!(lexer.peek_input(), Some(&'5'));
    assert_eq!(lexer.skip(), Some('5'));
    assert_eq!(lexer.skip(), Some('6'));
    assert_eq!(lexer.peek_input(), None);
    assert_eq!(lexer.skip(), None);
}

#[test]
fn test_tokenize() {
    let mut lexer = Lexer::new("123");
    assert_eq!(
        lexer.tokenize(),
        Ok(Token {
            kind: TokenKind::NumLit(123)
        })
    );
    let mut lexer = Lexer::new("987_654_321");
    assert_eq!(
        lexer.tokenize(),
        Ok(Token {
            kind: TokenKind::NumLit(987654321)
        })
    );
}
