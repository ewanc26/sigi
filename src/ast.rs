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

impl std::fmt::Display for SourceLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "line {}, column {}", self.line, self.col)
    }
}

/// The numeric type of a typed value.
///
/// Untyped numbers (`!5`, `!2.5`) are `F64`. Suffixed integer literals,
/// `i.T` casts and typed variables carry the other types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NumType {
    F64,
    F32,
    /// `long double`; its width is the C target's.
    LongDouble,
    I8,
    U8,
    I16,
    U16,
    I32,
    U32,
    I64,
    U64,
}

impl NumType {
    /// Look up a type name written after `i.` (cast), `z.` (sizeof) or `t.` (typedef).
    pub fn from_name(name: &str) -> Option<Self> {
        Some(match name {
            "f64" => Self::F64,
            "f32" => Self::F32,
            "ld" => Self::LongDouble,
            "i8" => Self::I8,
            "u8" => Self::U8,
            "i16" => Self::I16,
            "u16" => Self::U16,
            "i32" => Self::I32,
            "u32" => Self::U32,
            "i64" => Self::I64,
            "u64" => Self::U64,
            _ => return None,
        })
    }

    /// The C type this sigi type lowers to.
    pub fn c_type(self) -> &'static str {
        match self {
            Self::F64 => "double",
            Self::F32 => "float",
            Self::LongDouble => "long double",
            Self::I8 => "int8_t",
            Self::U8 => "uint8_t",
            Self::I16 => "int16_t",
            Self::U16 => "uint16_t",
            Self::I32 => "int32_t",
            Self::U32 => "uint32_t",
            Self::I64 => "int64_t",
            Self::U64 => "uint64_t",
        }
    }

    /// The enumerator for this type in the runtime prelude.
    pub fn c_tag(self) -> &'static str {
        match self {
            Self::F64 => "SG_F64",
            Self::F32 => "SG_F32",
            Self::LongDouble => "SG_LD",
            Self::I8 => "SG_I8",
            Self::U8 => "SG_U8",
            Self::I16 => "SG_I16",
            Self::U16 => "SG_U16",
            Self::I32 => "SG_I32",
            Self::U32 => "SG_U32",
            Self::I64 => "SG_I64",
            Self::U64 => "SG_U64",
        }
    }
}

/// A single operation in the sigi IR.
///
/// Most tokens lower to `Simple(kind)`.  Compound constructs (blocks,
/// loops, conditionals) carry child op vectors.
#[derive(Debug, Clone, PartialEq)]
pub enum Op {
    /// Push an untyped numeric literal onto the stack.
    Push(f64),
    /// Push a typed integer literal, written with a suffix such as `!5u`.
    PushInt { value: i128, ty: NumType },
    /// Pop a value and convert it to a type, written `i.T`.
    Cast(NumType),
    /// Pop a value and push its size in bytes, written `z`.
    SizeOf,
    /// Push the size in bytes of a type, written `z.T`.
    SizeOfType(NumType),
    /// Push the size in bytes of a struct, written `z.Name`.
    SizeOfAgg(String),
    /// Pop a pointer to a struct and push the address of one of its members, written `n.name`.
    Member(String),
    /// Push the value of a variable (indexed or named).
    Var(VarName),
    /// Push a pointer to a variable cell, written `j.name` or `j<n>`.
    AddrOf(VarName),
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

/// The type of a struct member or object: a scalar, or a struct defined earlier.
#[derive(Debug, Clone, PartialEq)]
pub enum TypeRef {
    Scalar(NumType),
    Aggregate(String),
}

/// One member of a struct.
#[derive(Debug, Clone, PartialEq)]
pub struct Field {
    pub name: String,
    pub ty: TypeRef,
}

/// A struct definition, written `r.Name{ .x .f64 .y .i32 }`.
#[derive(Debug, Clone, PartialEq)]
pub struct AggregateDef {
    pub name: String,
    pub fields: Vec<Field>,
    /// True for `u.Name{...}`: all members share storage.
    pub is_union: bool,
}

/// A constant initializer for a struct: a number for a scalar member, or a
/// nested list for a struct member.
#[derive(Debug, Clone, PartialEq)]
pub enum Init {
    Num(Op),
    List(Vec<Init>),
}

/// A named object declared with `r .T [ !N ... ] .name`, optionally with
/// `= ...`.  It is a C variable (or array) of the scalar or struct type T.
#[derive(Debug, Clone, PartialEq)]
pub struct ObjectDecl {
    pub name: String,
    pub elem: TypeRef,
    /// Array dimensions, outermost first; empty for a single object.
    pub dims: Vec<u64>,
    pub init: Option<Init>,
    /// Declared `.const`: the object cannot be stored to, directly or through a pointer.
    pub is_const: bool,
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
    pub aggregates: Vec<AggregateDef>,
    pub objects: Vec<ObjectDecl>,
}
