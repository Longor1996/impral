//! Parser errors.

use super::*;
use thiserror::Error;

/// A parsing error.
#[derive(Error, Debug)]
pub enum ParseError {
    /// The stream of tokens is empty.
    #[error("The stream of tokens is empty")]
    Empty,
    
    /// There was a character that could not be tokenized/lexed.
    #[error("Unrecognized token at {0}: {1}")]
    Unrecognized(usize, String),
    
    /// The stream of tokens ended unexpectedly.
    #[error("Expected {0}, but reached end of stream")]
    ExpectButEnd(&'static str),
    
    /// An unexpected thing appeared.
    #[error("Unexpected {0}")]
    Unexpected(Cow<'static, str>),
    
    /// Expected one thing, but got another.
    #[error("Expected {0}, but got {1}")]
    ExpectButGot(Cow<'static, str>, Cow<'static, str>),
    
    /// Positional args cannot be written after nominal args.
    #[error("Positional args cannot be written after nominal args")]
    PosArgAfterNomArg,
}
