//! AST types for the sigi stack language.
//!
//! The IR is a flat sequence of ops per function, plus a main body.
//! Every symbol token maps to either a primitive operation (`Op::Simple`)
//! or a compound form (Block, While, IfElse).

use std::fmt;

/// Source position used in error messages across all pipeline stages.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SourceLocation {
    pub line: usize,
    pub col: usize,
}

/// A single operation in the sigi IR.
///
/// Most tokens lower to `Simple(kind)`.  Compound constructs (blocks,
/// loops, conditionals) carry child op vectors.
#[derive(Debug, Clone, PartialEq)]
pub enum Op {
    /// Push a numeric literal onto the stack.
    Push(f64),
    /// Push the value of a variable (indexed or named).
    Var(VarName),
    /// Call a function by index or name.
    Call(TargetName),
    /// Pop and store to a named variable.
    StoreNamed(String),
    /// Emit a literal string (printed char-by-char).
    String(String),
    /// Anonymous block — scopes local ops.
    Block(Vec<Op>),
    /// While loop: repeat body while stack top is nonzero.
    While(Vec<Op>),
    /// Conditional: pop condition, execute one of two bodies.
    IfElse {
        then_body: Vec<Op>,
        else_body: Vec<Op>,
    },
    /// A primitive operation identified by its token name (ADD, DUP, etc.).
    Simple(String),
}

/// Reference to a variable slot.
#[derive(Debug, Clone, PartialEq)]
pub enum VarName {
    /// Numeric variable slot (0--99).
    Index(usize),
    /// User-defined named variable.
    Named(String),
}

/// Reference to a callable target.
#[derive(Debug, Clone, PartialEq)]
pub enum TargetName {
    /// Numbered function (0--99).
    Index(usize),
    /// Named function (prefixed with `.` in source).
    Named(String),
}

/// A function definition with its IR body and source location.
#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub name: TargetName,
    pub body: Vec<Op>,
    pub loc: SourceLocation,
}

/// A complete program: a set of function definitions and a main block.
#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub functions: Vec<Function>,
    pub main_code: Vec<Op>,
}
