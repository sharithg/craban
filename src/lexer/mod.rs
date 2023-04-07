mod consts;
use std::fmt;

use consts::{TokenKind, LITERAL_TOKENS, TYPESCRIPT_KEYWORDS};

#[derive(Debug, Clone, PartialEq)]
pub struct Token<'a> {
    pub token_kind: TokenKind,
    pub text: &'a str,
    pub text_len: usize,
}

impl fmt::Display for Token<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        println!(
            "Token {{\n token_kind: TokenKind::{:#?},\n text: \"{}\",\n text_len: {}\n}}",
            self.token_kind,
            self.get_text(),
            self.text_len
        );
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Lexer<'a> {
    pub content: &'a str,
    pub content_len: usize,
    pub cursor: usize,
    pub line: usize,
    // begining of the line
    pub bol: usize,
}

fn is_symbol_start(s: char) -> bool {
    return s.is_alphabetic() || s == '_';
}

fn is_symbol(s: char) -> bool {
    return s.is_alphanumeric() || s == '_';
}

fn is_space(s: char) -> bool {
    s.is_whitespace()
}

impl Token<'_> {
    pub fn get_text(&self) -> String {
        if self.text_len == 1 {
            return self.text[..1].to_string();
        }
        self.text[..self.text_len].to_string()
    }
}

impl Lexer<'_> {
    pub fn new(content: &str, length: usize) -> Lexer {
        Lexer {
            content,
            content_len: length,
            bol: 0,
            cursor: 0,
            line: 0,
        }
    }

    fn _curr_cursor_char(&mut self) -> char {
        return self.content.chars().nth(self.cursor).unwrap();
    }

    fn _chop_char(&mut self) -> Result<char, ()> {
        if self.cursor >= self.content_len {
            return Err(());
        }

        let x = self._curr_cursor_char();

        self.cursor += 1;

        if x == '\n' {
            self.line += 1;
            self.bol = self.cursor;
        }

        Ok(x)
    }

    fn _trim(&mut self) {
        while self.cursor < self.content_len && is_space(self._curr_cursor_char()) {
            self._chop_char().unwrap();
        }
    }

    fn _starts_with(&mut self, prefix: &str) -> bool {
        let pref_len = prefix.len();

        if pref_len == 0 {
            return true;
        }

        if self.cursor + pref_len - 1 >= self.content_len {
            return false;
        }

        if &self.content[self.cursor..(self.cursor + pref_len)] != prefix {
            return false;
        }

        return true;
    }

    pub fn next(&mut self) -> Option<Token> {
        self._trim();

        let mut token = Token {
            text: &self.content[self.cursor..],
            text_len: 0,
            token_kind: TokenKind::TokenInvalid,
        };

        if self.cursor >= self.content_len {
            return None;
        }

        for i in 0..LITERAL_TOKENS.len() {
            if self._starts_with(LITERAL_TOKENS[i].text) {
                token.token_kind = LITERAL_TOKENS[i].token_kind.clone();
                token.text_len += LITERAL_TOKENS[i].text.len();
                self.cursor += LITERAL_TOKENS[i].text.len();
                return Some(token);
            }
        }

        if self._starts_with("//") {
            token.token_kind = TokenKind::TokenComment;
            while self.cursor < self.content_len && self._curr_cursor_char() != '\n' {
                self._chop_char().unwrap();
            }
            if self.cursor < self.content_len {
                self._chop_char().unwrap();
            }
            token.text_len = token.text.len() - self.content[self.cursor..].len();
            return Some(token);
        }

        if self._curr_cursor_char() == '*' {
            token.token_kind = TokenKind::TokenStar;
            token.text_len = 1;
            self.cursor += 1;
            return Some(token);
        }

        if is_symbol_start(self._curr_cursor_char()) {
            token.token_kind = TokenKind::TokenSymbol;
            while self.cursor <= self.content_len && is_symbol(self._curr_cursor_char()) {
                self._chop_char().unwrap();
                token.text_len += 1;
            }
            for i in 0..TYPESCRIPT_KEYWORDS.len() {
                if TYPESCRIPT_KEYWORDS[i] == token.get_text() {
                    token.token_kind = TokenKind::TokenKeyword;
                    break;
                }
            }
            return Some(token);
        }

        self.cursor += 1;
        token.text_len = 1;
        Some(token)
    }
}

#[test]
fn test_lexer_import() {
    let expected: &[Token] = &[
        Token {
            text: "import",
            text_len: 6,
            token_kind: TokenKind::TokenKeyword,
        },
        Token {
            text: "{",
            text_len: 1,
            token_kind: TokenKind::TokenOpenCurly,
        },
        Token {
            text: "myfunc",
            text_len: 6,
            token_kind: TokenKind::TokenSymbol,
        },
        Token {
            text: ",",
            text_len: 1,
            token_kind: TokenKind::TokenComma,
        },
        Token {
            text: "myvar",
            text_len: 5,
            token_kind: TokenKind::TokenSymbol,
        },
        Token {
            text: "}",
            text_len: 1,
            token_kind: TokenKind::TokenCloseCurly,
        },
        Token {
            text: "from",
            text_len: 4,
            token_kind: TokenKind::TokenKeyword,
        },
        Token {
            text: "'",
            text_len: 1,
            token_kind: TokenKind::TokenInvalid,
        },
        Token {
            text: "asds",
            text_len: 4,
            token_kind: TokenKind::TokenSymbol,
        },
        Token {
            text: "'",
            text_len: 1,
            token_kind: TokenKind::TokenInvalid,
        },
        Token {
            text: ";",
            text_len: 1,
            token_kind: TokenKind::TokenInvalid,
        },
        Token {
            text: "const",
            text_len: 5,
            token_kind: TokenKind::TokenKeyword,
        },
        Token {
            text: "myv",
            text_len: 3,
            token_kind: TokenKind::TokenSymbol,
        },
        Token {
            text: "=",
            text_len: 1,
            token_kind: TokenKind::TokenInvalid,
        },
        Token {
            text: "'",
            text_len: 1,
            token_kind: TokenKind::TokenInvalid,
        },
        Token {
            text: "sad",
            text_len: 3,
            token_kind: TokenKind::TokenSymbol,
        },
        Token {
            text: "'",
            text_len: 1,
            token_kind: TokenKind::TokenInvalid,
        },
    ];
    let mut idx = 0;

    let content = "import{myfunc,myvar} from 'asds'; const myv = 'sad'";
    let mut l = Lexer::new(content.clone(), content.len());
    while let Some(tok) = l.next() {
        assert_eq!(expected[idx].text, tok.get_text());
        assert_eq!(expected[idx].text_len, tok.text_len);
        assert_eq!(expected[idx].token_kind, tok.token_kind);
        idx += 1;
    }
}

#[test]
fn test_lexer_comment() {
    let expected: &[Token] = &[
        Token {
            token_kind: TokenKind::TokenKeyword,
            text: "const",
            text_len: 5,
        },
        Token {
            token_kind: TokenKind::TokenSymbol,
            text: "val",
            text_len: 3,
        },
        Token {
            token_kind: TokenKind::TokenInvalid,
            text: "=",
            text_len: 1,
        },
        Token {
            token_kind: TokenKind::TokenInvalid,
            text: "4",
            text_len: 1,
        },
        Token {
            token_kind: TokenKind::TokenComment,
            text: "// asdasdasd",
            text_len: 12,
        },
    ];
    let mut idx = 0;

    let content = "const val = 4 // asdasdasd";
    let mut l = Lexer::new(content.clone(), content.len());
    while let Some(tok) = l.next() {
        assert_eq!(expected[idx].text, tok.get_text());
        assert_eq!(expected[idx].text_len, tok.text_len);
        assert_eq!(expected[idx].token_kind, tok.token_kind);
        idx += 1;
    }
}
