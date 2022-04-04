//! Lexer that `tokenize`s a string slice into an iterator of `Token`'s.
use smartstring::alias::CompactString;
use peekmore::*;

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
