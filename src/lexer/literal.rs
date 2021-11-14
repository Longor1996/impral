//! Literal representation.

use smartstring::alias::CompactString;

/// A literal / value.
#[derive(Clone, PartialEq)]
pub enum Literal {
    /// Nothing
    Nil,
    
    /// Boolean
    Bool(bool),
    
    /// UTF-Character
    Char(char),
    
    /// Signed 64-bit Integer Number
    Int(i64),
    
    /// 64-bit Floating Point Number
    Dec(f64),
    
    /// String
    Str(CompactString),
    
    /// Bytes
    Byt(Vec<u8>)
}

impl Literal {
    /// Returns the type of the literal as static str.
    pub const fn get_type_str(&self) -> &str {
        match self {
            Literal::Nil => "nil",
            Literal::Bool(_) => "boolean",
            Literal::Char(_) => "character",
            Literal::Int(_) => "integer-number",
            Literal::Dec(_) => "decimal-number",
            Literal::Str(_) => "char-string",
            Literal::Byt(_) => "byte-string",
        }
    }
}

impl std::fmt::Debug for Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Literal::Nil => write!(f, "null"),
            Literal::Bool(true) => write!(f, "true"),
            Literal::Bool(false) => write!(f, "false"),
            Literal::Char(v) => write!(f, "{}", v),
            Literal::Int(v) => write!(f, "{}i", v),
            Literal::Dec(v) => write!(f, "{}f", v),
            Literal::Str(v) => write!(f, "{}", v),
            Literal::Byt(_v) => write!(f, "BYTES"),
        }
    }
}
