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

impl PartialEq<Symbol> for Token {
    fn eq(&self, other: &Symbol) -> bool {
        if let TokenContent::Symbol(symbol) = &self.content {
            symbol == other
        } else { false }
    }
}

/// The content of a token.
#[derive(Debug, Clone)]
pub enum TokenContent {
    /// A symbol.
    Symbol(Symbol),
    
    /// A literal.
    Literal(Literal),
    
    /// A group.
    Group(Symbol, Vec<Token>),
    
    /// The remainder.
    Remainder(String),
}

impl TryFrom<&TokenContent> for Symbol {
    type Error = ();

    fn try_from(value: &TokenContent) -> Result<Self, Self::Error> {
        match value {
            TokenContent::Symbol(s) => Ok(*s),
            TokenContent::Literal(_) => Err(()),
            TokenContent::Group(_, _) => Err(()),
            TokenContent::Remainder(_) => Err(()),
        }
    }
}
