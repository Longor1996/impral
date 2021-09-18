//! Literal representation.

use smartstring::alias::CompactString;

/// A literal / value.
#[derive(Debug, Clone)]
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
    pub fn get_type_str(&self) -> &str {
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
