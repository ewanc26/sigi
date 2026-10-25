//! Recursive-descent parser for the sigi stack language.
//!
//! Peekable lexer drives an LL(1) grammar: compound forms (blocks,
//! loops, if-else, functions) are delimited by matching symbol pairs.
//! Everything else is a single-token primitive.

use crate::ast::{
    walk_ops, ForeignFn, Function, Op, Program, SourceLocation, SwitchArm, TargetName, VarName,
};
use crate::lexer::{Lexer, Token, TokenKind, TokenValue};
use std::collections::{HashMap, HashSet};
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
    #[error("Break outside a loop at {0:?}")]
    BreakOutsideLoop(SourceLocation),
    #[error("Continue outside a loop at {0:?}")]
    ContinueOutsideLoop(SourceLocation),
    #[error("Enum value must be an integer that fits in 32 bits at {0:?}")]
    InvalidEnumValue(SourceLocation),
    #[error("Name {0} is already used as a variable, function, or enum constant at {1:?}")]
    NameConflict(String, SourceLocation),
    #[error("Enum constant {0} is declared twice at {1:?}")]
    DuplicateEnumConstant(String, SourceLocation),
    #[error("Case value must be an integer at {0:?}")]
    InvalidCaseValue(SourceLocation),
    #[error("Case {0} appears twice in one switch at {1:?}")]
    DuplicateCase(i64, SourceLocation),
    #[error("Switch has more than one default at {0:?}")]
    DuplicateDefault(SourceLocation),
    #[error("Label {0} is defined twice in one function")]
    DuplicateLabel(String),
    #[error("Goto targets label {0}, which is not defined in this function")]
    UndefinedLabel(String),
    #[error("Function {0:?} is referenced but not defined")]
    UndefinedFunction(TargetName),
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
    loop_depth: usize,
    switch_depth: usize,
    enum_consts: HashMap<String, i64>,
    named_refs: HashSet<String>,
    fn_names: HashSet<String>,
}

impl<'a> Parser<'a> {
    pub fn new(lexer: Lexer<'a>) -> Self {
        Self {
            lexer: lexer.peekable(),
            foreign: Vec::new(),
            loop_depth: 0,
            switch_depth: 0,
            enum_consts: HashMap::new(),
            named_refs: HashSet::new(),
            fn_names: HashSet::new(),
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
            if self.peek()?.kind == TokenKind::ENUM {
                self.parse_enum()?;
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
        check_labels(&main_code)?;
        for f in &functions {
            check_labels(&f.body)?;
        }
        let mut missing = None;
        let mut check_ref = |op: &Op| {
            if let Op::FuncRef(target) = op {
                if !functions.iter().any(|f| f.name == *target) && missing.is_none() {
                    missing = Some(target.clone());
                }
            }
        };
        walk_ops(&main_code, &mut check_ref);
        for f in &functions {
            walk_ops(&f.body, &mut check_ref);
        }
        if let Some(target) = missing {
            return Err(ParseError::UndefinedFunction(target));
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
            Some(TokenValue::String(bytes)) => String::from_utf8(bytes)
                .map_err(|_| ParseError::ExpectedToken(TokenKind::STRING, header_tok.loc))?,
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

    // ─── Enums ──────────────────────────────────────────────────

    /// Parse `e{ .A .B .C !5 .D }`. Each constant takes the previous value
    /// plus one unless a `!n` follows its name, as in C's `C = 5`.
    fn parse_enum(&mut self) -> Result<(), ParseError> {
        self.consume(TokenKind::ENUM)?;
        self.consume(TokenKind::BLOCK)?;
        let mut next: i64 = 0;
        loop {
            let tok = self.advance()?;
            match tok.kind {
                TokenKind::ENDB => break,
                TokenKind::IDENT => {
                    let name = match tok.value {
                        Some(TokenValue::Ident(s)) => s,
                        _ => return Err(ParseError::ExpectedToken(TokenKind::IDENT, tok.loc)),
                    };
                    if self.enum_consts.contains_key(&name) {
                        return Err(ParseError::DuplicateEnumConstant(name, tok.loc));
                    }
                    if self.named_refs.contains(&name) || self.fn_names.contains(&name) {
                        return Err(ParseError::NameConflict(name, tok.loc));
                    }
                    let mut value = next;
                    if self.peek()?.kind == TokenKind::NUM {
                        let num = self.advance()?;
                        value = match num.value {
                            Some(TokenValue::Num(n))
                                if n.fract() == 0.0
                                    && n >= i32::MIN as f64
                                    && n <= i32::MAX as f64 =>
                            {
                                n as i64
                            }
                            _ => return Err(ParseError::InvalidEnumValue(num.loc)),
                        };
                    }
                    self.enum_consts.insert(name, value);
                    next = value + 1;
                }
                _ => return Err(ParseError::UnexpectedToken(tok.kind, tok.loc)),
            }
        }
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
            Some(TokenValue::Ident(s)) => {
                if self.enum_consts.contains_key(&s) {
                    return Err(ParseError::NameConflict(s, num_tok.loc));
                }
                self.fn_names.insert(s.clone());
                TargetName::Named(s)
            }
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

    /// Parse the body of a `[ ]` or `d[ ]` loop, tracking loop depth so
    /// `q` and `w` are only accepted inside a loop.
    fn parse_loop_body(&mut self) -> Result<Vec<Op>, ParseError> {
        self.loop_depth += 1;
        let body = self.parse_ops();
        self.loop_depth -= 1;
        body
    }

    /// Parse `k!n` and `m` arms up to the closing `}` of a switch.
    /// Case and default labels are only recognised at this top level.
    fn parse_switch_arms(&mut self) -> Result<Vec<SwitchArm>, ParseError> {
        let mut arms = Vec::new();
        let mut seen = HashSet::new();
        let mut has_default = false;
        loop {
            let tok = self.advance()?;
            match tok.kind {
                TokenKind::ENDB => break,
                TokenKind::CASE => {
                    let num = self.advance()?;
                    let value = match num.value {
                        Some(TokenValue::Num(n))
                            if num.kind == TokenKind::NUM
                                && n.fract() == 0.0
                                && n.abs() < 9_007_199_254_740_992.0 =>
                        {
                            n as i64
                        }
                        _ => return Err(ParseError::InvalidCaseValue(num.loc)),
                    };
                    if !seen.insert(value) {
                        return Err(ParseError::DuplicateCase(value, tok.loc));
                    }
                    let body = self.parse_case_body()?;
                    arms.push(SwitchArm {
                        value: Some(value),
                        body,
                    });
                }
                TokenKind::DEFAULT => {
                    if has_default {
                        return Err(ParseError::DuplicateDefault(tok.loc));
                    }
                    has_default = true;
                    let body = self.parse_case_body()?;
                    arms.push(SwitchArm { value: None, body });
                }
                _ => return Err(ParseError::UnexpectedToken(tok.kind, tok.loc)),
            }
        }
        Ok(arms)
    }

    fn parse_case_body(&mut self) -> Result<Vec<Op>, ParseError> {
        let mut ops = Vec::new();
        loop {
            let kind = self.peek()?.kind;
            if matches!(kind, TokenKind::CASE | TokenKind::DEFAULT | TokenKind::ENDB) {
                break;
            }
            ops.push(self.parse_op()?);
        }
        Ok(ops)
    }

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
            TokenKind::CHAR => Ok(Op::Char(if let Some(TokenValue::Char(c)) = tok.value {
                c
            } else {
                0
            })),
            TokenKind::VAR => Ok(Op::Var(VarName::Index(
                if let Some(TokenValue::Var(n)) = tok.value {
                    n
                } else {
                    0
                },
            ))),
            TokenKind::IDENT => {
                let name = match tok.value {
                    Some(TokenValue::Ident(s)) => s,
                    _ => String::new(),
                };
                if let Some(value) = self.enum_consts.get(&name) {
                    return Ok(Op::Push(*value as f64));
                }
                self.named_refs.insert(name.clone());
                Ok(Op::Var(VarName::Named(name)))
            }
            TokenKind::StoreIdent => {
                let name = match tok.value {
                    Some(TokenValue::Ident(s)) => s,
                    _ => String::new(),
                };
                if self.enum_consts.contains_key(&name) {
                    return Err(ParseError::NameConflict(name, tok.loc));
                }
                self.named_refs.insert(name.clone());
                Ok(Op::StoreNamed(name))
            }
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
            TokenKind::FUNCREF => {
                let target_tok = self.advance()?;
                match target_tok.value {
                    Some(TokenValue::Var(n)) => Ok(Op::FuncRef(TargetName::Index(n))),
                    Some(TokenValue::Ident(s)) => Ok(Op::FuncRef(TargetName::Named(s))),
                    _ => Err(ParseError::ExpectedToken(TokenKind::IDENT, target_tok.loc)),
                }
            }
            TokenKind::CALLPTR => Ok(Op::CallPtr),
            TokenKind::LABEL | TokenKind::GOTO => {
                let name_tok = self.advance()?;
                let name = match name_tok.value {
                    Some(TokenValue::Ident(s)) => s,
                    _ => return Err(ParseError::ExpectedToken(TokenKind::IDENT, name_tok.loc)),
                };
                Ok(if tok.kind == TokenKind::LABEL {
                    Op::Label(name)
                } else {
                    Op::Goto(name)
                })
            }
            TokenKind::SWITCH => {
                self.consume(TokenKind::BLOCK)?;
                self.switch_depth += 1;
                let arms = self.parse_switch_arms();
                self.switch_depth -= 1;
                Ok(Op::Switch(arms?))
            }
            TokenKind::STRING => Ok(Op::String(
                if let Some(TokenValue::String(bytes)) = tok.value {
                    bytes
                } else {
                    Vec::new()
                },
            )),
            TokenKind::WHILE => {
                let body = self.parse_loop_body()?;
                self.consume(TokenKind::WEND)?;
                Ok(Op::While(body))
            }
            TokenKind::DO => {
                self.consume(TokenKind::WHILE)?;
                let body = self.parse_loop_body()?;
                self.consume(TokenKind::WEND)?;
                Ok(Op::DoWhile(body))
            }
            TokenKind::FOR => {
                self.loop_depth += 1;
                let op = self.parse_op();
                self.loop_depth -= 1;
                match op? {
                    Op::Block(body) => Ok(Op::For(body)),
                    _ => Err(ParseError::ExpectedToken(TokenKind::BLOCK, tok.loc)),
                }
            }
            TokenKind::BREAK => {
                if self.loop_depth + self.switch_depth == 0 {
                    return Err(ParseError::BreakOutsideLoop(tok.loc));
                }
                Ok(Op::Break)
            }
            TokenKind::CONTINUE => {
                if self.loop_depth == 0 {
                    return Err(ParseError::ContinueOutsideLoop(tok.loc));
                }
                Ok(Op::Continue)
            }
            TokenKind::FfiDecl | TokenKind::ENUM | TokenKind::CASE | TokenKind::DEFAULT => {
                Err(ParseError::UnexpectedToken(tok.kind, tok.loc))
            }
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

/// Check that labels are unique within one function body and that every
/// goto names a label defined in that same body.
fn check_labels(ops: &[Op]) -> Result<(), ParseError> {
    let mut defined = HashSet::new();
    let mut duplicate = None;
    walk_ops(ops, &mut |op| {
        if let Op::Label(name) = op {
            if !defined.insert(name.clone()) && duplicate.is_none() {
                duplicate = Some(name.clone());
            }
        }
    });
    if let Some(name) = duplicate {
        return Err(ParseError::DuplicateLabel(name));
    }
    let mut missing = None;
    walk_ops(ops, &mut |op| {
        if let Op::Goto(name) = op {
            if !defined.contains(name) && missing.is_none() {
                missing = Some(name.clone());
            }
        }
    });
    match missing {
        Some(name) => Err(ParseError::UndefinedLabel(name)),
        None => Ok(()),
    }
}
