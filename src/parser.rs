//! Recursive-descent parser for the sigi stack language.
//!
//! Peekable lexer drives an LL(1) grammar: compound forms (blocks,
//! loops, if-else, functions) are delimited by matching symbol pairs.
//! Everything else is a single-token primitive.

use crate::ast::{ForeignFn, Function, Op, Program, SourceLocation, TargetName, VarName};
use crate::lexer::{Lexer, Token, TokenKind, TokenValue};
use thiserror::Error;

const MAX_FOREIGN_ARITY: usize = 4;

// ─── Parse Errors ────────────────────────────────────────────────

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Expected token {0:?} at {1:?}")]
    ExpectedToken(TokenKind, SourceLocation),
    #[error("Unexpected token {0:?} at {1:?}")]
    UnexpectedToken(TokenKind, SourceLocation),
    #[error("Function number must be 0-99 at {0:?}")]
    InvalidFunctionNumber(SourceLocation),
    #[error("Foreign arity must be 0-{MAX_FOREIGN_ARITY} at {0:?}")]
    InvalidArity(SourceLocation),
    #[error("Foreign function {0} is declared twice at {1:?}")]
    DuplicateForeign(String, SourceLocation),
    #[error("Foreign function {0} is not declared at {1:?}")]
    UndeclaredForeign(String, SourceLocation),
}

// ─── Parser ──────────────────────────────────────────────────────

/// Turns a token stream into a `Program` AST.
///
/// The parser distinguishes function definitions from anonymous blocks
/// by peeking ahead: `{N ...}` or `{.name ...}` is a function, `{...}`
/// alone is a block or if-else.
pub struct Parser<'a> {
    lexer: std::iter::Peekable<Lexer<'a>>,
    foreign: Vec<ForeignFn>,
}

impl<'a> Parser<'a> {
    pub fn new(lexer: Lexer<'a>) -> Self {
        Self {
            lexer: lexer.peekable(),
            foreign: Vec::new(),
        }
    }

    fn peek(&mut self) -> Result<&Token, ParseError> {
        match self.lexer.peek() {
            Some(Ok(t)) => Ok(t),
            // Future: propagate the actual lex error instead of masking it
            Some(Err(_)) => Err(ParseError::UnexpectedToken(
                TokenKind::EOF,
                SourceLocation { line: 0, col: 0 },
            )),
            None => Err(ParseError::UnexpectedToken(
                TokenKind::EOF,
                SourceLocation { line: 0, col: 0 },
            )),
        }
    }

    fn advance(&mut self) -> Result<Token, ParseError> {
        match self.lexer.next() {
            Some(Ok(t)) => Ok(t),
            _ => Err(ParseError::UnexpectedToken(
                TokenKind::EOF,
                SourceLocation { line: 0, col: 0 },
            )),
        }
    }

    fn consume(&mut self, kind: TokenKind) -> Result<Token, ParseError> {
        let tok = self.advance()?;
        if tok.kind == kind {
            Ok(tok)
        } else {
            Err(ParseError::ExpectedToken(kind, tok.loc))
        }
    }

    // ─── Program ────────────────────────────────────────────────

    pub fn parse_program(&mut self) -> Result<Program, ParseError> {
        let mut functions = Vec::new();
        let mut main_code = Vec::new();

        while self.lexer.peek().is_some() {
            if self.peek()?.kind == TokenKind::FfiDecl {
                self.parse_foreign()?;
                continue;
            }
            let tok = self.peek()?;
            if tok.kind == TokenKind::BLOCK {
                // Peek at the second token — if it's a var or ident this
                // is a function definition, otherwise an anonymous block.
                let next = self.lexer.clone().nth(1);
                if let Some(Ok(n)) = next {
                    if n.kind == TokenKind::VAR || n.kind == TokenKind::IDENT {
                        functions.push(self.parse_function()?);
                        continue;
                    }
                }
            }
            main_code.push(self.parse_op()?);
        }
        Ok(Program {
            functions,
            main_code,
            foreign: std::mem::take(&mut self.foreign),
        })
    }

    // ─── Foreign Declarations ───────────────────────────────────

    /// Parse `x "header" .name !arity`.
    fn parse_foreign(&mut self) -> Result<(), ParseError> {
        self.advance()?;
        let header_tok = self.advance()?;
        let header = match header_tok.value {
            Some(TokenValue::String(s)) => s,
            _ => return Err(ParseError::ExpectedToken(TokenKind::STRING, header_tok.loc)),
        };
        let name_tok = self.advance()?;
        let name = match name_tok.value {
            Some(TokenValue::Ident(s)) => s,
            _ => return Err(ParseError::ExpectedToken(TokenKind::IDENT, name_tok.loc)),
        };
        let arity_tok = self.advance()?;
        let arity = match arity_tok.value {
            Some(TokenValue::Num(n))
                if n.fract() == 0.0 && (0.0..=MAX_FOREIGN_ARITY as f64).contains(&n) =>
            {
                n as usize
            }
            _ => return Err(ParseError::InvalidArity(arity_tok.loc)),
        };
        if self.foreign.iter().any(|f| f.name == name) {
            return Err(ParseError::DuplicateForeign(name, name_tok.loc));
        }
        self.foreign.push(ForeignFn {
            name,
            header,
            arity,
        });
        Ok(())
    }

    // ─── Functions ──────────────────────────────────────────────

    fn parse_function(&mut self) -> Result<Function, ParseError> {
        let tok = self.advance()?;
        let num_tok = self.advance()?;

        let name = match num_tok.value {
            Some(TokenValue::Var(n)) => {
                if n > 99 {
                    return Err(ParseError::InvalidFunctionNumber(num_tok.loc));
                }
                TargetName::Index(n)
            }
            Some(TokenValue::Ident(s)) => TargetName::Named(s),
            _ => return Err(ParseError::ExpectedToken(TokenKind::IDENT, num_tok.loc)),
        };

        let body = self.parse_ops()?;
        self.consume(TokenKind::ENDB)?;

        // Skip optional else branch (used when function doubles as if-else body).
        if self
            .peek()
            .map(|t| t.kind == TokenKind::ELSE)
            .unwrap_or(false)
        {
            self.advance()?;
        }

        Ok(Function {
            name,
            body,
            loc: tok.loc,
        })
    }

    // ─── Op Sequences ───────────────────────────────────────────

    fn parse_ops(&mut self) -> Result<Vec<Op>, ParseError> {
        let mut ops = Vec::new();
        while let Ok(tok) = self.peek() {
            if matches!(
                tok.kind,
                TokenKind::EOF | TokenKind::ENDB | TokenKind::WEND | TokenKind::ELSE
            ) {
                break;
            }
            ops.push(self.parse_op()?);
        }
        Ok(ops)
    }

    // ─── Single Op ──────────────────────────────────────────────

    fn parse_op(&mut self) -> Result<Op, ParseError> {
        let tok = self.advance()?;
        match tok.kind {
            TokenKind::NUM => Ok(Op::Push(if let Some(TokenValue::Num(n)) = tok.value {
                n
            } else {
                0.0
            })),
            TokenKind::CHAR => Ok(Op::Push(if let Some(TokenValue::Char(c)) = tok.value {
                c as f64
            } else {
                0.0
            })),
            TokenKind::VAR => Ok(Op::Var(VarName::Index(
                if let Some(TokenValue::Var(n)) = tok.value {
                    n
                } else {
                    0
                },
            ))),
            TokenKind::IDENT => Ok(Op::Var(VarName::Named(
                if let Some(TokenValue::Ident(s)) = tok.value {
                    s
                } else {
                    "".to_string()
                },
            ))),
            TokenKind::StoreIdent => Ok(Op::StoreNamed(
                if let Some(TokenValue::Ident(s)) = tok.value {
                    s
                } else {
                    "".to_string()
                },
            )),
            TokenKind::CALL => {
                let target_tok = self.advance()?;
                let target = match target_tok.value {
                    Some(TokenValue::Var(n)) => TargetName::Index(n),
                    Some(TokenValue::Ident(s)) => TargetName::Named(s),
                    _ => return Err(ParseError::ExpectedToken(TokenKind::IDENT, target_tok.loc)),
                };
                self.consume(TokenKind::ENDCALL)?;
                Ok(Op::Call(target))
            }
            TokenKind::STRING => Ok(Op::String(if let Some(TokenValue::String(s)) = tok.value {
                s
            } else {
                "".to_string()
            })),
            TokenKind::WHILE => {
                let body = self.parse_ops()?;
                self.consume(TokenKind::WEND)?;
                Ok(Op::While(body))
            }
            TokenKind::FOR => match self.parse_op()? {
                Op::Block(body) => Ok(Op::For(body)),
                _ => Err(ParseError::ExpectedToken(TokenKind::BLOCK, tok.loc)),
            },
            TokenKind::FfiDecl => Err(ParseError::UnexpectedToken(tok.kind, tok.loc)),
            TokenKind::FfiCall => {
                let name_tok = self.advance()?;
                let name = match name_tok.value {
                    Some(TokenValue::Ident(s)) => s,
                    _ => return Err(ParseError::ExpectedToken(TokenKind::IDENT, name_tok.loc)),
                };
                if !self.foreign.iter().any(|f| f.name == name) {
                    return Err(ParseError::UndeclaredForeign(name, name_tok.loc));
                }
                Ok(Op::FfiCall(name))
            }
            TokenKind::BLOCK => {
                let then_body = self.parse_ops()?;
                if self
                    .peek()
                    .map(|t| t.kind == TokenKind::ELSE)
                    .unwrap_or(false)
                {
                    // Block followed by `;` is an if-else.
                    self.advance()?;
                    let else_body = self.parse_ops()?;
                    self.consume(TokenKind::ENDB)?;
                    Ok(Op::IfElse {
                        then_body,
                        else_body,
                    })
                } else {
                    self.consume(TokenKind::ENDB)?;
                    Ok(Op::Block(then_body))
                }
            }
            // Everything else is a primitive op named by its debug repr.
            _ => Ok(Op::Simple(format!("{:?}", tok.kind))),
        }
    }
}
