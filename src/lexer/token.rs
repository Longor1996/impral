//! Token representation.

use super::{Literal, Symbol};

/// An individual token.
#[derive(Debug, Clone)]
pub struct Token {
    /// Byte-position of the token in the input string slice.
    pub position: usize,
    
    /// The content of the token.
    pub content: TokenContent
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?} at {}", self.content, self.position)
    }
}

impl From<(usize, Symbol)> for Token {
    fn from(src: (usize, Symbol)) -> Self {
        Token {
            position: src.0,
            content: TokenContent::Symbol(src.1)
        }
    }
}

impl From<(usize, Literal)> for Token {
    fn from(src: (usize, Literal)) -> Self {
        Token {
            position: src.0,
            content: TokenContent::Literal(src.1)
        }
    }
}

impl From<(usize, TokenContent)> for Token {
    fn from(src: (usize, TokenContent)) -> Self {
        Token {
            position: src.0,
            content: src.1
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
