use crate::{file::*, token::*};
use std::rc::Rc;

pub struct Scanner {
    file: Rc<File>,
    line: usize,
    start: usize,
    current: usize
}

fn is_alpha(ch: u8) -> bool {
    u8_in_range(ch, b'a', b'z') || u8_in_range(ch, b'A', b'Z')
}

fn is_digit(ch: u8) -> bool {
    u8_in_range(ch, b'0', b'9')
}

fn u8_in_range(ch: u8, first: u8, last: u8) -> bool {
    ch >= first && ch <= last
}

// TODO: Use chars. Use iterators.
impl Scanner {
    pub fn new(file: Rc<File>) -> Scanner {
        Scanner {
            file,
            line: 1,
            start: 0,
            current: 0
        }
    }

    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace();

        self.start = self.current;

        match self.advance() {
            Some(ch) => match *ch {
                b'(' => self.make_token(TokenType::LeftParen),
                b')' => self.make_token(TokenType::RightParen),
                b'{' => self.make_token(TokenType::LeftBracket),
                b'}' => self.make_token(TokenType::RightBracket),
                b'[' => self.make_token(TokenType::LeftBrace),
                b']' => self.make_token(TokenType::RightBrace),
                b';' => self.make_token(TokenType::Semicolon),
                b':' => self.make_token(TokenType::Colon),
                b',' => self.make_token(TokenType::Comma),
                b'?' => self.make_token(TokenType::QuestionMark),

                b'+' => if self.matching(b'+') {
                    self.make_token(TokenType::PlusPlus)
                } else {
                    self.make_token(TokenType::Plus)
                },

                b'*' => self.make_token(TokenType::Star),
                b'/' => self.make_token(TokenType::Slash),
                b'%' => self.make_token(TokenType::Percent),
                b'~' => self.make_token(TokenType::Tilda),

                b'-' => if self.matching(b'-') {
                    self.make_token(TokenType::MinusMinus)
                } else if is_digit(self.peek().map_or(b'_', |ch| *ch)) {
                    self.number()
                } else {
                    self.make_token(TokenType::Minus)
                },

                b'!' => if self.matching(b'=') {
                    self.make_token(TokenType::BangEqual)
                } else {
                    self.make_token(TokenType::Bang)
                }

                b'=' => if self.matching(b'=') {
                    self.make_token(TokenType::EqualEqual)
                } else {
                    self.make_token(TokenType::Equal)
                }

                b'>' => if self.matching(b'=') {
                    self.make_token(TokenType::GreaterEqual)
                } else {
                    if self.matching(b'>') {
                        self.make_token(TokenType::GreaterGreater)
                    } else {
                        self.make_token(TokenType::Greater)
                    }
                }

                b'<' => if self.matching(b'=') {
                    self.make_token(TokenType::LessEqual)
                } else {
                    if self.matching(b'<') {
                        self.make_token(TokenType::LessLess)
                    } else {
                        self.make_token(TokenType::Less)
                    }
                }

                b'|' => if self.matching(b'|') {
                    self.make_token(TokenType::BarBar)
                } else {
                    self.make_token(TokenType::Bar)
                }

                b'&' => if self.matching(b'&') {
                    self.make_token(TokenType::AmpersandAmpersand)
                } else {
                    self.make_token(TokenType::Ampersand)
                }

                b'^' => self.make_token(TokenType::UpArrow),

                b'\'' => self.character_literal(),

                b'\"' => self.string(),

                _ => {
                    if is_alpha(*ch) || *ch == b'_' {
                        self.identifier_or_keyword()
                    } else if is_digit(*ch) {
                        self.number()
                    } else {
                        self.make_error_token("unrecognized character")
                    }
                }
            }

            None => self.make_eof_token()
        }
    }

    fn character_literal(&mut self) -> Token {
        if self.is_at_end() {
            self.make_error_token("expected character")
        } else {
            self.advance(); // Consume character.

            if self.is_at_end() {
                self.make_error_token("unterminated character literal")
            } else {
                self.advance(); // Consume '.
                self.make_token(TokenType::CharLiteral)
            }
        }
    }

    fn string(&mut self) -> Token {
        self.advance_while(|ch| {
            ch != b'\"'
        });

        if self.is_at_end() {
            self.make_error_token("unterminated character literal")
        } else {
            self.advance(); // Consume ".
            self.make_token(TokenType::StringLiteral)
        }
    }

    fn number(&mut self) -> Token {
        self.advance_while(is_digit);
        self.make_token(TokenType::IntLiteral)
    }

    fn identifier_or_keyword(&mut self) -> Token {
        self.advance_while(|ch| {
            is_alpha(ch) || is_digit(ch) || ch == b'_'
        });

        self.make_token(self.check_identifier())
    }

    fn check_identifier(&self) -> TokenType {
        match self.file.data[self.start] {
            b'i' => self.check_rest(1, b"f", TokenType::KeywordIf),
            b'r' => self.check_rest(1, b"eturn", TokenType::KeywordReturn),

            b'e' => if self.current - self.start > 1 {
                match self.file.data[self.start + 1] {
                    b'l' => self.check_rest(2, b"se", TokenType::KeywordElse),
                    b'x' => self.check_rest(2, b"tern", TokenType::KeywordExtern),
                    _ => TokenType::Identifier
                }
            } else {
                TokenType::Identifier
            }
            
            b'w' => self.check_rest(1, b"hile", TokenType::KeywordWhile),
            b'd' => self.check_rest(1, b"o", TokenType::KeywordDo),
            b'b' => self.check_rest(1, b"reak", TokenType::KeywordBreak),
            b'c' => self.check_rest(1, b"ontinue", TokenType::KeywordContinue),
            b'a' => self.check_rest(1, b"uto", TokenType::KeywordAuto),

            _    => TokenType::Identifier
        }
    }

    fn check_rest(&self, already: usize, rest: &[u8], keyword: TokenType) -> TokenType {
        for i in already..(rest.len() + already) {
            match self.file.data.get(self.start + i) {
                Some(ch) => {
                    if *ch == rest[i - already] {
                        continue
                    } else {
                        return TokenType::Identifier;
                    }
                },
                None => return TokenType::Identifier
            }
        }

        return keyword;
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.file.data.len()
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.peek() {
            match *ch {
                b' ' | b'\t' | b'\r' => { self.advance(); },
                b'\n' => {
                    self.line += 1;
                    self.advance();
                }
                _ => break
            }
        }
    }

    fn advance(&mut self) -> Option<&u8> {
        self.current += 1;
        self.file.data
            .get(self.current - 1)
    }

    fn advance_while<F>(&mut self, f: F) where
        F: Fn(u8) -> bool {
        while let Some(ch) = self.peek() {
            if f(*ch) {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn matching(&mut self, ch: u8) -> bool {
        let peeked_ch = self.peek();

        if let Some(peeked_ch) = peeked_ch {
            if *peeked_ch == ch {
                self.advance();
                return true;
            }
        }

        return false;
    }

    fn peek(&self) -> Option<&u8> {
        self.file.data
            .get(self.current)
    }

    fn make_current_position(&self) -> FilePosition {
        FilePosition {
            file: self.file.clone(),
            line: self.line
        }
    }

    fn make_token(&self, kind: TokenType) -> Token {
        Token {
            kind,
            pos: self.make_current_position(),
            data: String::from_utf8_lossy(&self.file.data[self.start..self.current]).into_owned()
        }
    }

    fn make_error_token(&self, msg: &'static str) -> Token {
        Token {
            kind: TokenType::Error,
            pos: self.make_current_position(),
            data: String::from(msg)
        }
    }

    fn make_eof_token(&self) -> Token {
        Token {
            kind: TokenType::EndOfFile,
            pos: self.make_current_position(),
            data: String::from("")
        }
    }
}
