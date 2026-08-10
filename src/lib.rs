//! sigi — compiler pipeline for a pure symbolic stack language.
//!
//! The pipeline is: lex → parse → codegen (emit C).
//! Every token is a single punctuation character — no alphanumeric keywords.

pub mod ast;
pub mod codegen;
pub mod lexer;
pub mod parser;
