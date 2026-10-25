//! Lexer for the sigi symbolic stack language.
//!
//! Every valid token is a single punctuation character — no alphanumeric
//! keywords.  Identifiers and store-targets use `.` and `:.` prefixes.
//! The lexer also strips C-style block comments and line comments (`\`).

use crate::ast::{NumType, SourceLocation};
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
    #[error("Integer constant out of range at {0:?}")]
    IntegerOutOfRange(SourceLocation),
    #[error("Invalid digit in octal constant at {0:?}")]
    InvalidOctal(SourceLocation),
    #[error("Invalid integer suffix at {0:?}")]
    InvalidSuffix(SourceLocation),
    #[error("Unknown escape sequence '\\{0}' at {1:?}")]
    UnknownEscape(char, SourceLocation),
    #[error("Escape sequence value out of range at {0:?}")]
    EscapeOutOfRange(SourceLocation),
    #[error("Malformed escape sequence at {0:?}")]
    InvalidEscape(SourceLocation),
    #[error("Character constant must denote one byte at {0:?}")]
    MultiByteChar(SourceLocation),
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
    BITAND,
    BITOR,
    BITXOR,
    BITNOT,
    SHL,
    SHR,
    LOGAND,
    LOGOR,
    INTDIV,
    FOR,
    DO,
    BREAK,
    CONTINUE,
    ENUM,
    LABEL,
    GOTO,
    SWITCH,
    CASE,
    DEFAULT,
    FUNCREF,
    CALLPTR,
    FfiDecl,
    FfiCall,
    CAST,
    SIZEOF,
    TYPEDEF,
    ADDRESS,
    DEREF,
    DECL,
    MEMBER,
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
    /// A suffixed integer constant, carrying its exact value and type.
    Int(i128, NumType),
    Var(usize),
    Ident(String),
    Char(u8),
    String(Vec<u8>),
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

    /// Read a double-quoted string.  The result is bytes: source text is
    /// UTF-8 and escapes produce the bytes C would store for them.
    fn read_string(&mut self, loc: SourceLocation) -> Result<Token, LexError> {
        let mut bytes = Vec::new();
        while let Some(ch) = self.advance() {
            if ch == '"' {
                return Ok(Token {
                    kind: TokenKind::STRING,
                    value: Some(TokenValue::String(bytes)),
                    loc,
                });
            }
            if ch == '\\' {
                bytes.extend(self.read_escape(loc)?);
            } else {
                let mut buf = [0u8; 4];
                bytes.extend_from_slice(ch.encode_utf8(&mut buf).as_bytes());
            }
        }
        Err(LexError::UnterminatedString(loc))
    }

    /// Read a single-quoted character literal.  The literal must denote
    /// exactly one byte, as a C character constant does.
    fn read_char(&mut self, loc: SourceLocation) -> Result<Token, LexError> {
        let ch = self.advance().ok_or(LexError::UnterminatedChar(loc))?;
        let bytes = if ch == '\\' {
            self.read_escape(loc)?
        } else {
            let mut buf = [0u8; 4];
            ch.encode_utf8(&mut buf).as_bytes().to_vec()
        };
        if bytes.len() != 1 {
            return Err(LexError::MultiByteChar(loc));
        }
        Ok(Token {
            kind: TokenKind::CHAR,
            value: Some(TokenValue::Char(bytes[0])),
            loc,
        })
    }

    /// Read one escape sequence; the backslash is already consumed.
    /// Covers the C23 simple escapes, octal, hexadecimal and universal
    /// character names (encoded as UTF-8).
    fn read_escape(&mut self, loc: SourceLocation) -> Result<Vec<u8>, LexError> {
        let esc = self.advance().ok_or(LexError::UnterminatedEscape(loc))?;
        let byte = match esc {
            '\'' => b'\'',
            '"' => b'"',
            '?' => b'?',
            '\\' => b'\\',
            'a' => 0x07,
            'b' => 0x08,
            'f' => 0x0c,
            'n' => b'\n',
            'r' => b'\r',
            't' => b'\t',
            'v' => 0x0b,
            '0'..='7' => {
                let mut value = esc.to_digit(8).unwrap();
                for _ in 0..2 {
                    match self.peek().and_then(|c| c.to_digit(8)) {
                        Some(digit) => {
                            self.advance();
                            value = value * 8 + digit;
                        }
                        None => break,
                    }
                }
                u8::try_from(value).map_err(|_| LexError::EscapeOutOfRange(loc))?
            }
            'x' => {
                let mut value: u32 = 0;
                let mut digits = 0;
                while let Some(digit) = self.peek().and_then(|c| c.to_digit(16)) {
                    self.advance();
                    value = value.saturating_mul(16).saturating_add(digit);
                    digits += 1;
                }
                if digits == 0 {
                    return Err(LexError::InvalidEscape(loc));
                }
                u8::try_from(value).map_err(|_| LexError::EscapeOutOfRange(loc))?
            }
            'u' | 'U' => {
                let count = if esc == 'u' { 4 } else { 8 };
                let mut value: u32 = 0;
                for _ in 0..count {
                    let digit = self
                        .advance()
                        .and_then(|c| c.to_digit(16))
                        .ok_or(LexError::InvalidEscape(loc))?;
                    value = value * 16 + digit;
                }
                let ch = char::from_u32(value).ok_or(LexError::InvalidEscape(loc))?;
                let mut buf = [0u8; 4];
                return Ok(ch.encode_utf8(&mut buf).as_bytes().to_vec());
            }
            other => return Err(LexError::UnknownEscape(other, loc)),
        };
        Ok(vec![byte])
    }

    /// Read a numeric literal after the `!` prefix (e.g. `!3.14`).
    /// Negative literals use `!-n` syntax.
    ///
    /// Integer constants follow C: `0x`/`0X` hexadecimal, `0b`/`0B` binary,
    /// leading `0` octal, and the `u`/`l`/`ll` suffix family.  Decimal
    /// floating constants keep their existing spelling.
    fn read_number(&mut self, loc: SourceLocation) -> Result<Token, LexError> {
        let first = self.peek().ok_or(LexError::ExpectedNumber(loc))?;
        if !first.is_ascii_digit() && first != '-' && first != '.' {
            return Err(LexError::ExpectedNumber(loc));
        }

        let negative = if first == '-' {
            self.advance();
            let next = self.peek().ok_or(LexError::ExpectedDigit(loc))?;
            if !next.is_ascii_digit() {
                return Err(LexError::ExpectedDigit(loc));
            }
            true
        } else {
            false
        };

        // Hexadecimal (0x) and binary (0b) integer constants.
        if self.peek() == Some('0') {
            if let Some(radix_ch) = self
                .peek_nth(1)
                .filter(|c| matches!(*c, 'x' | 'X' | 'b' | 'B'))
            {
                self.advance();
                self.advance();
                let radix = if radix_ch == 'x' || radix_ch == 'X' {
                    16
                } else {
                    2
                };
                let mut digits = String::new();
                while let Some(c) = self.peek() {
                    if c.is_digit(radix) {
                        digits.push(self.advance().unwrap());
                    } else {
                        break;
                    }
                }
                if digits.is_empty() {
                    return Err(LexError::ExpectedNumber(loc));
                }
                let magnitude = u64::from_str_radix(&digits, radix)
                    .map_err(|_| LexError::IntegerOutOfRange(loc))?;
                return self.integer_token(magnitude, negative, loc);
            }
        }

        // Decimal digits, with an optional fraction.
        let mut num_str = String::new();
        while let Some(c) = self.peek() {
            if c.is_ascii_digit() || c == '.' {
                num_str.push(self.advance().unwrap());
            } else {
                break;
            }
        }

        if num_str.contains('.') {
            // Floating constants take no integer suffix.
            let val: f64 = num_str.parse().map_err(|_| LexError::ExpectedNumber(loc))?;
            let val = if negative { -val } else { val };
            return Ok(Token {
                kind: TokenKind::NUM,
                value: Some(TokenValue::Num(val)),
                loc,
            });
        }

        // A leading zero starts an octal constant.
        let magnitude = if num_str.len() > 1 && num_str.starts_with('0') {
            if num_str.chars().any(|c| c == '8' || c == '9') {
                return Err(LexError::InvalidOctal(loc));
            }
            u64::from_str_radix(&num_str, 8).map_err(|_| LexError::IntegerOutOfRange(loc))?
        } else {
            num_str
                .parse::<u64>()
                .map_err(|_| LexError::IntegerOutOfRange(loc))?
        };
        self.integer_token(magnitude, negative, loc)
    }

    /// Finish an integer constant: read its optional suffix and build the token.
    /// Unsuffixed constants stay untyped doubles. A `u` suffix gives `u32`
    /// (or `u64` when the value does not fit), `l`/`ll` give `i64`, and
    /// `ul`/`ull` give `u64`.
    fn integer_token(
        &mut self,
        magnitude: u64,
        negative: bool,
        loc: SourceLocation,
    ) -> Result<Token, LexError> {
        let mut suffix = String::new();
        while let Some(c) = self.peek() {
            if matches!(c, 'u' | 'U' | 'l' | 'L') {
                suffix.push(self.advance().unwrap());
            } else {
                break;
            }
        }
        const SUFFIXES: [&str; 22] = [
            "u", "U", "l", "L", "ll", "LL", "ul", "uL", "Ul", "UL", "lu", "lU", "Lu", "LU", "ull",
            "uLL", "Ull", "ULL", "llu", "llU", "LLu", "LLU",
        ];
        if !suffix.is_empty() && !SUFFIXES.contains(&suffix.as_str()) {
            return Err(LexError::InvalidSuffix(loc));
        }
        if negative && suffix.contains(['u', 'U']) {
            // `-n` with an unsigned suffix wraps in C; sigi does not model that.
            return Err(LexError::InvalidSuffix(loc));
        }
        let signed = if negative {
            -(magnitude as i128)
        } else {
            magnitude as i128
        };
        if suffix.is_empty() {
            return Ok(Token {
                kind: TokenKind::NUM,
                value: Some(TokenValue::Num(signed as f64)),
                loc,
            });
        }
        let unsigned = suffix.contains(['u', 'U']);
        let ty = if unsigned {
            if magnitude <= u32::MAX as u64 && !suffix.contains(['l', 'L']) {
                NumType::U32
            } else {
                NumType::U64
            }
        } else {
            if signed < i64::MIN as i128 || signed > i64::MAX as i128 {
                return Err(LexError::IntegerOutOfRange(loc));
            }
            NumType::I64
        };
        Ok(Token {
            kind: TokenKind::NUM,
            value: Some(TokenValue::Int(signed, ty)),
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
            'B' => TokenKind::BITAND,
            'V' => TokenKind::BITOR,
            'Q' => TokenKind::BITXOR,
            'J' => TokenKind::BITNOT,
            'D' => TokenKind::SHL,
            'I' => TokenKind::SHR,
            'b' => TokenKind::LOGAND,
            'v' => TokenKind::LOGOR,
            'l' => TokenKind::INTDIV,
            'f' => TokenKind::FOR,
            'd' => TokenKind::DO,
            'q' => TokenKind::BREAK,
            'w' => TokenKind::CONTINUE,
            'e' => TokenKind::ENUM,
            'h' => TokenKind::LABEL,
            'g' => TokenKind::GOTO,
            's' => TokenKind::SWITCH,
            'k' => TokenKind::CASE,
            'm' => TokenKind::DEFAULT,
            'o' => TokenKind::FUNCREF,
            'p' => TokenKind::CALLPTR,
            'x' => TokenKind::FfiDecl,
            'c' => TokenKind::FfiCall,
            'i' => TokenKind::CAST,
            'z' => TokenKind::SIZEOF,
            't' => TokenKind::TYPEDEF,
            'j' => TokenKind::ADDRESS,
            'y' => TokenKind::DEREF,
            'r' => TokenKind::DECL,
            'n' => TokenKind::MEMBER,
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
