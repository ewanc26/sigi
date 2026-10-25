//! AST types for the sigi stack language.
//!
//! The IR is a flat sequence of ops per function, plus a main body.
//! Every symbol token maps to either a primitive operation (`Op::Simple`)
//! or a compound form (Block, While, IfElse).

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
    /// Emit a literal string (printed byte-by-byte).
    String(Vec<u8>),
    /// Push a character constant (its byte value, as a C char constant).
    Char(u8),
    /// Anonymous block — scopes local ops.
    Block(Vec<Op>),
    /// While loop: repeat body while stack top is nonzero.
    While(Vec<Op>),
    /// Counted loop: pop count, run body that many times.
    For(Vec<Op>),
    /// Do-while loop: run body, then repeat while stack top is nonzero.
    DoWhile(Vec<Op>),
    /// Leave the innermost loop.
    Break,
    /// Skip to the next iteration of the innermost loop.
    Continue,
    /// Multi-way branch on the popped selector. Arms fall through like C.
    Switch(Vec<SwitchArm>),
    /// Jump target declared with `h.name`.
    Label(String),
    /// Jump to a label in the same function, written `g.name`.
    Goto(String),
    /// Push the index of a function, written `o.name` or `o<n>`.
    FuncRef(TargetName),
    /// Pop a function index and call the function it names.
    CallPtr,
    /// Conditional: pop condition, execute one of two bodies.
    IfElse {
        then_body: Vec<Op>,
        else_body: Vec<Op>,
    },
    /// Call a declared C function, popping its arguments from the stack.
    FfiCall(String),
    /// A primitive operation identified by its token name (ADD, DUP, etc.).
    Simple(String),
}

/// One arm of a `Switch`: `Some(value)` for `k!n`, `None` for `m`.
#[derive(Debug, Clone, PartialEq)]
pub struct SwitchArm {
    pub value: Option<i64>,
    pub body: Vec<Op>,
}

impl Op {
    /// Nested op lists of compound ops, for walking the tree.
    pub fn bodies(&self) -> Vec<&[Op]> {
        match self {
            Op::While(b) | Op::DoWhile(b) | Op::Block(b) | Op::For(b) => vec![b.as_slice()],
            Op::IfElse {
                then_body,
                else_body,
            } => vec![then_body.as_slice(), else_body.as_slice()],
            Op::Switch(arms) => arms.iter().map(|a| a.body.as_slice()).collect(),
            _ => Vec::new(),
        }
    }
}

/// Visit every op in `ops`, including ops nested inside compound ops.
pub fn walk_ops(ops: &[Op], f: &mut impl FnMut(&Op)) {
    for op in ops {
        f(op);
        for body in op.bodies() {
            walk_ops(body, f);
        }
    }
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

/// A C function declared with `x "header" .name !arity`.
#[derive(Debug, Clone, PartialEq)]
pub struct ForeignFn {
    pub name: String,
    pub header: String,
    pub arity: usize,
}

impl ForeignFn {
    pub fn is_user_source(&self) -> bool {
        self.header.ends_with(".c")
    }
}

/// A complete program: a set of function definitions and a main block.
#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub functions: Vec<Function>,
    pub main_code: Vec<Op>,
    pub foreign: Vec<ForeignFn>,
}
