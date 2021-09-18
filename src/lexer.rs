//! Lexer that `tokenize`s a string slice into an iterator of `Token`'s.
use std::{convert::TryFrom, iter::Peekable};
use smartstring::alias::CompactString;

pub mod tokenizer;
pub use tokenizer::*;

pub mod token;
pub use token::*;

pub mod symbol;
pub use symbol::*;

pub mod literal;
pub use literal::*;

#[cfg(test)]
mod tests;
