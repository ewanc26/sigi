//! Recursive-descent parser for the sigi stack language.
//!
//! Peekable lexer drives an LL(1) grammar: compound forms (blocks,
//! loops, if-else, functions) are delimited by matching symbol pairs.
//! Everything else is a single-token primitive.

use crate::ast::{
    walk_ops, AggregateDef, Field, ForeignFn, Function, Init, NumType, ObjectDecl, Op, Program,
    SourceLocation, SwitchArm, TargetName, TypeRef, VarName,
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
    #[error("Unknown type {0} at {1:?}")]
    UnknownType(String, SourceLocation),
    #[error("Type name {0} is already defined at {1:?}")]
    DuplicateTypedef(String, SourceLocation),
    #[error("{0} is already defined at {1:?}")]
    DuplicateDefinition(String, SourceLocation),
    #[error("Struct {0} has no members at {1:?}")]
    EmptyAggregate(String, SourceLocation),
    #[error("Member {0} is declared twice at {1:?}")]
    DuplicateMember(String, SourceLocation),
    #[error("{0} is a struct and cannot be stored to directly at {1:?}")]
    NotAssignable(String, SourceLocation),
    #[error("Array {0} takes no initializer at {1:?}")]
    ArrayInit(String, SourceLocation),
    #[error("Array dimension must be a positive integer literal at {0:?}")]
    InvalidDim(SourceLocation),
    #[error("Too many initializers for {0} at {1:?}")]
    TooManyInitializers(String, SourceLocation),
    #[error("Object {0} needs a struct type at {1:?}")]
    ObjectNeedsStruct(String, SourceLocation),
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
    typedefs: HashMap<String, NumType>,
    aggregates: Vec<AggregateDef>,
    agg_index: HashMap<String, usize>,
    objects: HashMap<String, bool>,
    object_list: Vec<ObjectDecl>,
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
            typedefs: HashMap::new(),
            aggregates: Vec::new(),
            agg_index: HashMap::new(),
            objects: HashMap::new(),
            object_list: Vec::new(),
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
            if self.peek()?.kind == TokenKind::TYPEDEF {
                self.parse_typedef()?;
                continue;
            }
            if matches!(self.peek()?.kind, TokenKind::DECL | TokenKind::UNION) {
                self.parse_decl()?;
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
            aggregates: std::mem::take(&mut self.aggregates),
            objects: std::mem::take(&mut self.object_list),
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
        let arity = match integer_literal(&arity_tok) {
            Some(n) if (0..=MAX_FOREIGN_ARITY as i128).contains(&n) => n as usize,
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
                        value = match integer_literal(&num) {
                            Some(n) if (i32::MIN as i128..=i32::MAX as i128).contains(&n) => {
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

    // ─── Typedefs ───────────────────────────────────────────────

    /// Parse `t.Name .type`, naming a scalar type. The alias is global and
    /// is used after `i.` and `z.` like a built-in type name.
    fn parse_typedef(&mut self) -> Result<(), ParseError> {
        self.consume(TokenKind::TYPEDEF)?;
        let name_tok = self.advance()?;
        let name = match name_tok.value {
            Some(TokenValue::Ident(s)) => s,
            _ => return Err(ParseError::ExpectedToken(TokenKind::IDENT, name_tok.loc)),
        };
        if NumType::from_name(&name).is_some() || self.typedefs.contains_key(&name) {
            return Err(ParseError::DuplicateTypedef(name, name_tok.loc));
        }
        let ty = self.parse_type_name()?;
        self.typedefs.insert(name, ty);
        Ok(())
    }

    /// Parse one type name: a built-in such as `.u8` or a typedef alias.
    fn parse_type_name(&mut self) -> Result<NumType, ParseError> {
        let tok = self.advance()?;
        match &tok.value {
            Some(TokenValue::Ident(name)) => self
                .resolve_type(name)
                .ok_or_else(|| ParseError::UnknownType(name.clone(), tok.loc)),
            _ => Err(ParseError::ExpectedToken(TokenKind::IDENT, tok.loc)),
        }
    }

    /// Resolve a name used as a member or object type: a struct, or a scalar.
    fn resolve_typeref(&self, name: &str) -> Option<TypeRef> {
        if self.agg_index.contains_key(name) {
            return Some(TypeRef::Aggregate(name.to_string()));
        }
        self.resolve_type(name).map(TypeRef::Scalar)
    }

    /// The struct named by the next token, if it names one. Does not consume it.
    fn peek_aggregate_name(&mut self) -> Option<String> {
        match self.lexer.peek() {
            Some(Ok(Token {
                kind: TokenKind::IDENT,
                value: Some(TokenValue::Ident(name)),
                ..
            })) if self.agg_index.contains_key(name) => Some(name.clone()),
            _ => None,
        }
    }

    fn resolve_type(&self, name: &str) -> Option<NumType> {
        self.typedefs
            .get(name)
            .copied()
            .or_else(|| NumType::from_name(name))
    }

    /// The type named by the next token, if it is a type name. Does not consume it.
    fn peek_type_name(&mut self) -> Option<NumType> {
        let name = match self.lexer.peek() {
            Some(Ok(Token {
                kind: TokenKind::IDENT,
                value: Some(TokenValue::Ident(name)),
                ..
            })) => name.clone(),
            _ => return None,
        };
        self.resolve_type(&name)
    }

    // ─── Declarations ───────────────────────────────────────────

    /// Parse a declaration that starts with `r` (struct) or `u` (union): a
    /// definition `r.Name{ ... }`, or an object `r .T [ !N ... ] .name [= ...]`
    /// where T is a scalar or a struct and the optional `[ ]` gives array
    /// dimensions.
    fn parse_decl(&mut self) -> Result<(), ParseError> {
        let keyword = self.advance()?;
        let is_union = keyword.kind == TokenKind::UNION;
        let first = self.advance()?;
        let first_name = ident_name(first.clone())?;
        if self.peek()?.kind == TokenKind::BLOCK {
            return self.parse_struct(first_name, first.loc, is_union);
        }
        if is_union {
            return Err(ParseError::ExpectedToken(TokenKind::BLOCK, first.loc));
        }
        let elem = self
            .resolve_typeref(&first_name)
            .ok_or_else(|| ParseError::UnknownType(first_name.clone(), first.loc))?;
        let dims = if self.peek()?.kind == TokenKind::WHILE {
            self.parse_dims()?
        } else {
            Vec::new()
        };
        let name_tok = self.advance()?;
        let name = ident_name(name_tok.clone())?;
        if self.objects.contains_key(&name)
            || self.enum_consts.contains_key(&name)
            || self.named_refs.contains(&name)
        {
            return Err(ParseError::DuplicateDefinition(name, name_tok.loc));
        }
        let init = if self.peek()?.kind == TokenKind::EQ {
            let eq = self.advance()?;
            if !dims.is_empty() {
                return Err(ParseError::ArrayInit(name, eq.loc));
            }
            Some(match &elem {
                TypeRef::Scalar(_) => Init::Num(self.parse_num_op()?),
                TypeRef::Aggregate(agg) => {
                    let agg = agg.clone();
                    self.parse_init(&agg)?
                }
            })
        } else {
            None
        };
        // Only a plain scalar object is assigned with `:.name`.
        let assignable = dims.is_empty() && matches!(elem, TypeRef::Scalar(_));
        self.objects.insert(name.clone(), assignable);
        self.object_list.push(ObjectDecl {
            name,
            elem,
            dims,
            init,
        });
        Ok(())
    }

    /// Parse array dimensions `[ !N !M ... ]`. Each must be a positive
    /// integer literal.
    fn parse_dims(&mut self) -> Result<Vec<u64>, ParseError> {
        let open = self.advance()?;
        let mut dims = Vec::new();
        while self.peek()?.kind != TokenKind::WEND {
            let tok = self.advance()?;
            match integer_literal(&tok) {
                Some(n) if n > 0 && n <= u32::MAX as i128 && tok.kind == TokenKind::NUM => {
                    dims.push(n as u64)
                }
                _ => return Err(ParseError::InvalidDim(tok.loc)),
            }
        }
        self.advance()?;
        if dims.is_empty() {
            return Err(ParseError::InvalidDim(open.loc));
        }
        Ok(dims)
    }

    /// Parse one numeric literal token as a constant operation.
    fn parse_num_op(&mut self) -> Result<Op, ParseError> {
        let num = self.advance()?;
        match num.value {
            Some(TokenValue::Int(value, ty)) if num.kind == TokenKind::NUM => {
                Ok(Op::PushInt { value, ty })
            }
            Some(TokenValue::Num(n)) if num.kind == TokenKind::NUM => Ok(Op::Push(n)),
            _ => Err(ParseError::ExpectedToken(TokenKind::NUM, num.loc)),
        }
    }

    /// Parse the brace list that initializes a struct object. Each element
    /// fills the next member in order; members past the end of the list are
    /// zeroed. A scalar member takes a number, a struct member takes a list.
    fn parse_init(&mut self, agg: &str) -> Result<Init, ParseError> {
        let open = self.advance()?;
        if open.kind != TokenKind::BLOCK {
            return Err(ParseError::ExpectedToken(TokenKind::BLOCK, open.loc));
        }
        let fields = self.aggregates[self.agg_index[agg]].fields.clone();
        let mut values = Vec::new();
        loop {
            let tok = self.peek()?.clone();
            if tok.kind == TokenKind::ENDB {
                self.advance()?;
                break;
            }
            let field = fields
                .get(values.len())
                .ok_or_else(|| ParseError::TooManyInitializers(agg.to_string(), tok.loc))?;
            match &field.ty {
                TypeRef::Scalar(_) => values.push(Init::Num(self.parse_num_op()?)),
                TypeRef::Aggregate(sub) => {
                    let sub = sub.clone();
                    values.push(self.parse_init(&sub)?);
                }
            }
        }
        Ok(Init::List(values))
    }

    /// Parse the body of `r.Name{ .x .f64 .y .i32 }`. Each member is a name
    /// followed by a scalar type or a struct defined earlier.
    fn parse_struct(
        &mut self,
        name: String,
        loc: SourceLocation,
        is_union: bool,
    ) -> Result<(), ParseError> {
        if self.agg_index.contains_key(&name)
            || NumType::from_name(&name).is_some()
            || self.typedefs.contains_key(&name)
        {
            return Err(ParseError::DuplicateDefinition(name, loc));
        }
        self.consume(TokenKind::BLOCK)?;
        let mut fields: Vec<Field> = Vec::new();
        loop {
            let tok = self.advance()?;
            match tok.kind {
                TokenKind::ENDB => break,
                TokenKind::IDENT => {
                    let field = ident_name(tok.clone())?;
                    let ty_tok = self.advance()?;
                    let ty_name = ident_name(ty_tok.clone())?;
                    let ty = self
                        .resolve_typeref(&ty_name)
                        .ok_or_else(|| ParseError::UnknownType(ty_name, ty_tok.loc))?;
                    if fields.iter().any(|f| f.name == field) {
                        return Err(ParseError::DuplicateMember(field, tok.loc));
                    }
                    fields.push(Field { name: field, ty });
                }
                _ => return Err(ParseError::UnexpectedToken(tok.kind, tok.loc)),
            }
        }
        if fields.is_empty() {
            return Err(ParseError::EmptyAggregate(name, loc));
        }
        self.agg_index.insert(name.clone(), self.aggregates.len());
        self.aggregates.push(AggregateDef {
            name,
            fields,
            is_union,
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
                    let value = match integer_literal(&num) {
                        Some(n) if num.kind == TokenKind::NUM && i64::try_from(n).is_ok() => {
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
            TokenKind::NUM => match tok.value {
                Some(TokenValue::Int(value, ty)) => Ok(Op::PushInt { value, ty }),
                Some(TokenValue::Num(n)) => Ok(Op::Push(n)),
                _ => Ok(Op::Push(0.0)),
            },
            TokenKind::CAST => Ok(Op::Cast(self.parse_type_name()?)),
            TokenKind::SIZEOF => match self.peek_type_name() {
                Some(ty) => {
                    // `z.T` is the size of a type; a bare `z` measures the top value.
                    self.advance()?;
                    Ok(Op::SizeOfType(ty))
                }
                None => match self.peek_aggregate_name() {
                    Some(name) => {
                        self.advance()?;
                        Ok(Op::SizeOfAgg(name))
                    }
                    None => Ok(Op::SizeOf),
                },
            },
            TokenKind::MEMBER => {
                let name_tok = self.advance()?;
                Ok(Op::Member(ident_name(name_tok)?))
            }
            TokenKind::TYPEDEF => Err(ParseError::UnexpectedToken(tok.kind, tok.loc)),
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
                if self.objects.get(&name) == Some(&false) {
                    return Err(ParseError::NotAssignable(name, tok.loc));
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
            TokenKind::ADDRESS => {
                let target = self.advance()?;
                match target.value {
                    Some(TokenValue::Var(n)) if target.kind == TokenKind::VAR => {
                        Ok(Op::AddrOf(VarName::Index(n)))
                    }
                    Some(TokenValue::Ident(name)) if target.kind == TokenKind::IDENT => {
                        if self.enum_consts.contains_key(&name) {
                            return Err(ParseError::NameConflict(name, target.loc));
                        }
                        self.named_refs.insert(name.clone());
                        Ok(Op::AddrOf(VarName::Named(name)))
                    }
                    _ => Err(ParseError::ExpectedToken(TokenKind::IDENT, target.loc)),
                }
            }
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
            TokenKind::FfiDecl
            | TokenKind::ENUM
            | TokenKind::DECL
            | TokenKind::UNION
            | TokenKind::CASE
            | TokenKind::DEFAULT => Err(ParseError::UnexpectedToken(tok.kind, tok.loc)),
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

/// The name carried by an identifier token.
fn ident_name(tok: Token) -> Result<String, ParseError> {
    match tok.value {
        Some(TokenValue::Ident(s)) if tok.kind == TokenKind::IDENT => Ok(s),
        _ => Err(ParseError::ExpectedToken(TokenKind::IDENT, tok.loc)),
    }
}

/// The whole-number value of a numeric literal token, if it has one.
fn integer_literal(tok: &Token) -> Option<i128> {
    match tok.value {
        Some(TokenValue::Num(n)) if n.fract() == 0.0 && n.abs() < 9_007_199_254_740_992.0 => {
            Some(n as i128)
        }
        Some(TokenValue::Int(value, _)) => Some(value),
        _ => None,
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
