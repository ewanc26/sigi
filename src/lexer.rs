use thiserror::Error;
use crate::ast::SourceLocation;

#[derive(Error, Debug, PartialEq)]
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

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    NUM,
    VAR,
    IDENT,
    STORE_IDENT,
    CHAR,
    STRING,
    BLOCK,
    ENDB,
    ELSE,
    WHILE,
    WEND,
    CALL,
    ENDCALL,
    // Simple tokens
    DUP, SWAP, DROP, ADD, SUB, MUL, DIV, MOD,
    EQ, LT, GT, NOT,
    PRINT, PRINTC, INPUT, STORE,
    SIN, COS, TAN, SQRT, POW, FLOOR, LOG, EXP, ABS, ATAN2,
    RAND, EXIT, TIME,
    ALEN, ALOAD, ASTORE, AINIT, AFREE,
    FILE_OPEN, FILE_READ, FILE_WRITE, FILE_CLOSE,
    USLEEP,
    EOF,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub value: Option<TokenValue>,
    pub loc: SourceLocation,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenValue {
    Num(f64),
    Var(usize),
    Ident(String),
    Char(u8),
    String(String),
}

pub struct Lexer<'a> {
    source: &'a str,
    chars: std::iter::Peekable<std::str::Chars<'a>>,
    line: usize,
    col: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
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
        SourceLocation { line: self.line, col: self.col }
    }
    
    // ... Implement tokenization logic ...
}
