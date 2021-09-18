//! Parses a stream of tokens into an AST.

use std::{borrow::Cow, fmt::Debug, marker::PhantomData};

use smallvec::{SmallVec, smallvec};
use tagged_box::TaggableContainer;
use rustc_hash::FxHashMap;
use smartstring::alias::CompactString;

use crate::{lexer::*, values::*};

pub mod expression;
pub use expression::*;

pub mod command;
pub use command::*;

pub mod error;
pub use error::*;

#[cfg(test)]
mod tests;
