//! Parses a stream of tokens into an AST.

use std::{borrow::Cow, fmt::Debug};

use smallvec::{SmallVec, smallvec};
use rustc_hash::FxHashMap;
use smartstring::alias::CompactString;

use crate::lexer::*;

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

#[cfg(test)]
mod tests;
