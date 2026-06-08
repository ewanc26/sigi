use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SourceLocation {
    pub line: usize,
    pub col: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Op {
    Push(f64),
    Var(VarName),
    Call(TargetName),
    StoreNamed(String),
    String(String),
    Block(Vec<Op>),
    While(Vec<Op>),
    IfElse { then_body: Vec<Op>, else_body: Vec<Op> },
    Simple(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum VarName {
    Index(usize),
    Named(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum TargetName {
    Index(usize),
    Named(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub name: TargetName,
    pub body: Vec<Op>,
    pub loc: SourceLocation,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub functions: Vec<Function>,
    pub main_code: Vec<Op>,
}
