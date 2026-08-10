//! Lexer for the sigi symbolic stack language.
//!
//! Every valid token is a single punctuation character — no alphanumeric
//! keywords.  Identifiers and store-targets use `.` and `:.` prefixes.
//! The lexer also strips C-style block comments and line comments (`\`).

use crate::ast::SourceLocation;
use std::iter::Peekable;
use std::str::Chars;
use thiserror::Error;

// ─── Lex Errors ─────────────────────────────────────────────────

#[derive(Error, Debug, PartialEq, Clone)]
pub enum LexError {
    #[error("Unterminated string at {0:?}")]
    UnterminatedString(SourceLocation),
    #[error("Unterminated escape at {0:?}")]
    UnterminatedEscape(SourceLocation),
    #[error("Unterminated char at {0:?}")]
    UnterminatedChar(SourceLocation),
    #[error("Unterminated block comment at {0:?}")]
    UnterminatedBlockComment(SourceLocation),
    #[error("Expected number after '!' at {0:?}")]
    ExpectedNumber(SourceLocation),
    #[error("Expected digit after '-' at {0:?}")]
    ExpectedDigit(SourceLocation),
    #[error("Expected name after '{0}' at {1:?}")]
    ExpectedName(String, SourceLocation),
    #[error("Unexpected character {0:?} at {1:?}")]
    UnexpectedCharacter(char, SourceLocation),
}

// ─── Token Types ─────────────────────────────────────────────────

/// Kinds of tokens the sigi language recognises.
///
/// The naming follows the symbolic character, not the semantic operation,
/// so mapping stays one-to-one with the language spec.
#[derive(Debug, Clone, PartialEq, Copy)]
pub enum TokenKind {
    NUM,
    VAR,
    IDENT,
    StoreIdent,
    CHAR,
    STRING,
    BLOCK,
    ENDB,
    ELSE,
    WHILE,
    WEND,
    CALL,
    ENDCALL,
    DUP,
    SWAP,
    DROP,
    ADD,
    SUB,
    MUL,
    DIV,
    MOD,
    EQ,
    LT,
    GT,
    NOT,
    PRINT,
    PRINTC,
    INPUT,
    STORE,
    SIN,
    COS,
    TAN,
    SQRT,
    POW,
    FLOOR,
    LOG,
    EXP,
    ABS,
    ATAN2,
    RAND,
    EXIT,
    TIME,
    ALEN,
    ALOAD,
    ASTORE,
    AINIT,
    AFREE,
    FileOpen,
    FileRead,
    FileWrite,
    FileClose,
    USLEEP,
    EOF,
}

/// A single token with its kind, optional payload, and source location.
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub value: Option<TokenValue>,
    pub loc: SourceLocation,
}

/// The typed payload carried by some tokens (numbers, vars, names, chars, strings).
#[derive(Debug, Clone, PartialEq)]
pub enum TokenValue {
    Num(f64),
    Var(usize),
    Ident(String),
    Char(u8),
    String(String),
}

// ─── Lexer ───────────────────────────────────────────────────────

/// Character-level tokeniser for sigi source.
///
/// Advances through the source char-by-char, tracking line/col for
/// error reporting.  Whitespace, commas, and comments are skipped
/// silently between tokens.
#[derive(Clone)]
pub struct Lexer<'a> {
    chars: Peekable<Chars<'a>>,
    line: usize,
    col: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            chars: source.chars().peekable(),
            line: 1,
            col: 1,
        }
    }

    fn peek(&mut self) -> Option<char> {
        self.chars.peek().copied()
    }

    fn advance(&mut self) -> Option<char> {
        let ch = self.chars.next()?;
        if ch == '\n' {
            self.line += 1;
            self.col = 1;
        } else {
            self.col += 1;
        }
        Some(ch)
    }

    fn current_loc(&self) -> SourceLocation {
        SourceLocation {
            line: self.line,
            col: self.col,
        }
    }

    fn skip_whitespace_and_comments(&mut self) -> Result<(), LexError> {
        while let Some(ch) = self.peek() {
            if ch.is_whitespace() || ch == ',' {
                // Commas are treated as whitespace — they exist solely to
                // improve readability in dense symbolic code.
                self.advance();
            } else if ch == '\\' {
                // Line comment: skip to end of line.
                self.advance();
                while let Some(c) = self.peek() {
                    if c == '\n' {
                        break;
                    }
                    self.advance();
                }
            } else if ch == '/' && self.peek_nth(1) == Some('*') {
                // Block comment: C-style /* ... */.
                let loc = self.current_loc();
                self.advance();
                self.advance();
                while let Some(c) = self.peek() {
                    if c == '*' && self.peek_nth(1) == Some('/') {
                        self.advance();
                        self.advance();
                        break;
                    }
                    if self.advance().is_none() {
                        return Err(LexError::UnterminatedBlockComment(loc));
                    }
                }
            } else {
                break;
            }
        }
        Ok(())
    }

    fn peek_nth(&mut self, n: usize) -> Option<char> {
        self.chars.clone().nth(n)
    }

    pub fn next_token(&mut self) -> Result<Token, LexError> {
        self.skip_whitespace_and_comments()?;

        let loc = self.current_loc();
        let ch = match self.advance() {
            Some(c) => c,
            None => {
                return Ok(Token {
                    kind: TokenKind::EOF,
                    value: None,
                    loc,
                })
            }
        };

        match ch {
            '"' => self.read_string(loc),
            '\'' => self.read_char(loc),
            '!' => self.read_number(loc), // !prefix for numeric literals
            '.' => self.read_identifier(loc), // .name for named vars/functions
            ':' => {
                if self.peek() == Some('.') {
                    // :.name for store-to-named
                    self.advance();
                    self.read_store_ident(loc)
                } else {
                    Ok(Token {
                        kind: TokenKind::STORE,
                        value: None,
                        loc,
                    })
                }
            }
            c if c.is_ascii_digit() => self.read_var(c, loc), // bare digits → variable ref
            _ => self.read_symbol(ch, loc),
        }
    }

    // ─── Token Readers ───────────────────────────────────────────

    /// Read a double-quoted string, processing escape sequences.
    fn read_string(&mut self, loc: SourceLocation) -> Result<Token, LexError> {
        let mut chars = String::new();
        while let Some(ch) = self.advance() {
            if ch == '"' {
                return Ok(Token {
                    kind: TokenKind::STRING,
                    value: Some(TokenValue::String(chars)),
                    loc,
                });
            }
            if ch == '\\' {
                let esc = self.advance().ok_or(LexError::UnterminatedString(loc))?;
                chars.push(match esc {
                    'n' => '\n',
                    't' => '\t',
                    'r' => '\r',
                    '\\' => '\\',
                    '"' => '"',
                    _ => esc,
                });
            } else {
                chars.push(ch);
            }
        }
        Err(LexError::UnterminatedString(loc))
    }

    /// Read a single-quoted character literal.
    fn read_char(&mut self, loc: SourceLocation) -> Result<Token, LexError> {
        let ch = self.advance().ok_or(LexError::UnterminatedChar(loc))?;
        let value = if ch == '\\' {
            let esc = self.advance().ok_or(LexError::UnterminatedChar(loc))?;
            match esc {
                'n' => b'\n',
                't' => b'\t',
                'r' => b'\r',
                '\'' => b'\'',
                '\\' => b'\\',
                _ => esc as u8,
            }
        } else {
            ch as u8
        };
        Ok(Token {
            kind: TokenKind::CHAR,
            value: Some(TokenValue::Char(value)),
            loc,
        })
    }

    /// Read a numeric literal after the `!` prefix (e.g. `!3.14`).
    /// Negative literals use `!-n` syntax.
    fn read_number(&mut self, loc: SourceLocation) -> Result<Token, LexError> {
        let first = self.peek().ok_or(LexError::ExpectedNumber(loc))?;
        if !first.is_ascii_digit() && first != '-' && first != '.' {
            return Err(LexError::ExpectedNumber(loc));
        }

        let mut num_str = String::new();
        if first == '-' {
            num_str.push(self.advance().unwrap());
            let next = self.peek().ok_or(LexError::ExpectedDigit(loc))?;
            if !next.is_ascii_digit() {
                return Err(LexError::ExpectedDigit(loc));
            }
        }

        while let Some(c) = self.peek() {
            if c.is_ascii_digit() || c == '.' {
                num_str.push(self.advance().unwrap());
            } else {
                break;
            }
        }

        let val: f64 = num_str.parse().map_err(|_| LexError::ExpectedNumber(loc))?;
        Ok(Token {
            kind: TokenKind::NUM,
            value: Some(TokenValue::Num(val)),
            loc,
        })
    }

    /// Read a named identifier after `.` prefix.
    fn read_identifier(&mut self, loc: SourceLocation) -> Result<Token, LexError> {
        let mut name = String::new();
        while let Some(c) = self.peek() {
            if c.is_alphanumeric() || c == '_' {
                name.push(self.advance().unwrap());
            } else {
                break;
            }
        }
        if name.is_empty() {
            return Err(LexError::ExpectedName(".".to_string(), loc));
        }
        Ok(Token {
            kind: TokenKind::IDENT,
            value: Some(TokenValue::Ident(name)),
            loc,
        })
    }

    /// Read a store-target name after `:.` prefix.
    fn read_store_ident(&mut self, loc: SourceLocation) -> Result<Token, LexError> {
        let mut name = String::new();
        while let Some(c) = self.peek() {
            if c.is_alphanumeric() || c == '_' {
                name.push(self.advance().unwrap());
            } else {
                break;
            }
        }
        if name.is_empty() {
            return Err(LexError::ExpectedName(":.".to_string(), loc));
        }
        Ok(Token {
            kind: TokenKind::StoreIdent,
            value: Some(TokenValue::Ident(name)),
            loc,
        })
    }

    /// Read a variable reference (bare digits 0--99).  Larger numbers
    /// are returned as NUM to allow non-var constant push.
    fn read_var(&mut self, first: char, loc: SourceLocation) -> Result<Token, LexError> {
        let mut num_str = first.to_string();
        while let Some(c) = self.peek() {
            if c.is_ascii_digit() {
                num_str.push(self.advance().unwrap());
            } else {
                break;
            }
        }
        let val: usize = num_str.parse().unwrap();
        Ok(Token {
            kind: if val <= 99 {
                TokenKind::VAR
            } else {
                TokenKind::NUM
            },
            value: Some(TokenValue::Var(val)),
            loc,
        })
    }

    /// Map a single punctuation character to its `TokenKind`.
    /// This is the core symbol table of the sigi language.
    fn read_symbol(&mut self, ch: char, loc: SourceLocation) -> Result<Token, LexError> {
        let kind = match ch {
            '@' => TokenKind::DUP,
            '#' => TokenKind::SWAP,
            '$' => TokenKind::DROP,
            '+' => TokenKind::ADD,
            '-' => TokenKind::SUB,
            '*' => TokenKind::MUL,
            '/' => TokenKind::DIV,
            '%' => TokenKind::MOD,
            '=' => TokenKind::EQ,
            '<' => TokenKind::LT,
            '>' => TokenKind::GT,
            '~' => TokenKind::NOT,
            '|' => TokenKind::PRINT,
            '^' => TokenKind::PRINTC,
            '?' => TokenKind::INPUT,
            '[' => TokenKind::WHILE,
            ']' => TokenKind::WEND,
            '(' => TokenKind::CALL,
            ')' => TokenKind::ENDCALL,
            '{' => TokenKind::BLOCK,
            '}' => TokenKind::ENDB,
            ';' => TokenKind::ELSE,
            'S' => TokenKind::SIN,
            'C' => TokenKind::COS,
            'T' => TokenKind::TAN,
            'R' => TokenKind::SQRT,
            'P' => TokenKind::POW,
            'F' => TokenKind::FLOOR,
            'L' => TokenKind::LOG,
            'E' => TokenKind::EXP,
            'M' => TokenKind::ABS,
            'N' => TokenKind::ATAN2,
            'W' => TokenKind::RAND,
            'X' => TokenKind::EXIT,
            'Z' => TokenKind::TIME,
            '&' => TokenKind::ALEN,
            'A' => TokenKind::ALOAD,
            'a' => TokenKind::ASTORE,
            '_' => TokenKind::AINIT,
            'K' => TokenKind::AFREE,
            'O' => TokenKind::FileOpen,
            'G' => TokenKind::FileRead,
            'H' => TokenKind::FileWrite,
            'Y' => TokenKind::FileClose,
            'U' => TokenKind::USLEEP,
            _ => return Err(LexError::UnexpectedCharacter(ch, loc)),
        };
        Ok(Token {
            kind,
            value: None,
            loc,
        })
    }
}

// ─── Iterator Adapter ────────────────────────────────────────────

/// Wrap `next_token` as a standard iterator, suppressing the EOF sentinel.
impl<'a> Iterator for Lexer<'a> {
    type Item = Result<Token, LexError>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.next_token() {
            Ok(token) => {
                if token.kind == TokenKind::EOF {
                    None
                } else {
                    Some(Ok(token))
                }
            }
            Err(e) => Some(Err(e)),
        }
    }
}

// ─── Tests ───────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lex_simple() {
        let mut lexer = Lexer::new("!5 @ +");
        let t1 = lexer.next_token().unwrap();
        assert_eq!(t1.kind, TokenKind::NUM);
        assert_eq!(t1.value, Some(TokenValue::Num(5.0)));
        let t2 = lexer.next_token().unwrap();
        assert_eq!(t2.kind, TokenKind::DUP);
        let t3 = lexer.next_token().unwrap();
        assert_eq!(t3.kind, TokenKind::ADD);
        let t4 = lexer.next_token().unwrap();
        assert_eq!(t4.kind, TokenKind::EOF);
    }
}
