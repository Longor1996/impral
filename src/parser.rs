//! Parses a stream of tokens into an AST.

use std::{borrow::Cow, fmt::Debug};

use smallvec::{SmallVec, smallvec};
use rustc_hash::FxHashMap;
use smartstring::alias::CompactString;

use crate::lexer::*;

pub mod helpers;
pub use helpers::*;

pub mod expression;
pub use expression::*;

pub mod structure;
pub use structure::*;

pub mod command;
pub use command::*;

pub mod error;
pub use error::*;

pub mod ast;
pub use ast::*;

/// An active parser.
#[derive(Default)]
pub struct Parser {
    /// The current depth of the expression tree.
    pub(crate) depth: u16,
    /// The block that will contain the fully parsed (linearized) expression tree.
    pub(crate) block: Block,
    /// When the parser cannot fully parse the input, this will contain the remainder.
    pub(crate) remainder: Option<String>,
}

#[cfg(test)]
mod tests;
