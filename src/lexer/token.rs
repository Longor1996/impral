//! Token representation.

use crate::parser::ParseError;
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
    
    /// A group.
    Group(Symbol, Vec<Token>),
    
    /// The remainder.
    Remainder(String),
}

impl Token {
    
    /// Try to convert the given TokenContent into a command-name...
    pub fn try_into_command_name(&self) -> Result<smartstring::alias::CompactString, ParseError> {
        match self.content.clone() {
            TokenContent::Remainder(r )
                => Err(ParseError::Unrecognized(self.start, r)),
            
            // Every kind of symbol BUT delimiters can be a command name...
            TokenContent::Symbol(s ) if !s.is_operator()
                => Err(ParseError::ExpectButGot("a command name".into(), format!("a '{}'", s).into())),
            TokenContent::Symbol(s) => Ok((&s).into()),
            
            // Every kind of literal BUT strings cannot be a command name...
            TokenContent::Literal(Literal::Str(s)) => Ok(s),
            TokenContent::Literal(l)
                => Err(ParseError::ExpectButGot("a command name".into(), format!("a {}", l.get_type_str()).into())),
            
            TokenContent::Group(_, _)
                => Err(ParseError::ExpectButGot("a command name".into(), "a group".to_string().into())),
        }
    }
    
}