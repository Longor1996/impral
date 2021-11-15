//! Token representation.

use super::{Literal, Symbol};

/// An individual token.
#[derive(Debug, Clone)]
pub struct Token {
    /// Byte-position of the START of the token in the input string slice.
    pub start: usize,
    
    /// Byte-position of the END of the token in the input string slice.
    pub end: usize,
    
    /// The content of the token.
    pub content: TokenContent
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?} at {}", self.content, self.start)
    }
}

impl From<(usize, usize, Symbol)> for Token {
    fn from(src: (usize, usize, Symbol)) -> Self {
        Token {
            start: src.0,
            end: src.1,
            content: TokenContent::Symbol(src.2)
        }
    }
}

impl From<(usize, usize, Literal)> for Token {
    fn from(src: (usize, usize, Literal)) -> Self {
        Token {
            start: src.0,
            end: src.1,
            content: TokenContent::Literal(src.2)
        }
    }
}

impl From<(usize, usize, TokenContent)> for Token {
    fn from(src: (usize, usize, TokenContent)) -> Self {
        Token {
            start: src.0,
            end: src.1,
            content: src.2
        }
    }
}

/// The content of a token.
#[derive(Debug, Clone)]
pub enum TokenContent {
    /// A symbol.
    Symbol(Symbol),
    
    /// A literal.
    Literal(Literal),
    
    /// The remainder.
    Remainder(String),
}
