//! Literal representation.

use std::borrow::Cow;

use smartstring::alias::CompactString;

/// A literal / value.
#[derive(Clone)]
#[repr(u8)]
pub enum Literal {
    /// Nothing
    Nil,
    
    /// Boolean
    Bool(bool),
    
    /// Signed 64-bit Integer Number
    Int(i64),
    
    /// 64-bit Floating Point Number
    Dec(f64),
    
    /// Uid (`U67e55044-10b1-426f-9247-bb680e5fe0c8`)
    Uid(uuid::Uuid),
    
    /// String | Bareword
    Str(CompactString),
    
    /// Bytes
    Byt(Box<Byt>),
    
    /// Result Reference (`$`)
    RefRes,
    
    /// Context Reference (`$$`)
    RefCtx,
    
    /// Local Reference (`$NAME`)
    RefVar(CompactString),
    
    /// Object Idx Reference (`@0`)
    /// 
    /// A global reference to an object identified via a plain integer.
    ObjIdx(usize),
    
    /// Object Uid Reference (`@67e55044-10b1-426f-9247-bb680e5fe0c8`)
    /// 
    /// A global reference to an object identified via UUID.
    ObjUid(uuid::Uuid),
    
    /// Object Key Reference (`@NAME` / `@'NAME'` / `@"NAME"`)
    /// 
    /// A global reference to a named object.
    ObjKey(CompactString)
}

impl std::cmp::PartialEq for Literal {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Nil, Self::Nil) => true,
            (Self::Bool(l), Self::Bool(r)) => l == r,
            (Self::Int(l), Self::Int(r)) => l == r,
            (Self::Dec(l), Self::Dec(r)) => l.to_bits() == r.to_bits(),
            (Self::Uid(l), Self::Uid(r)) => l == r,
            (Self::Str(l), Self::Str(r)) => l == r,
            (Self::Byt(l), Self::Byt(r)) => l == r,
            (Self::RefRes, Self::RefRes) => true,
            (Self::RefCtx, Self::RefCtx) => true,
            (Self::RefVar(l), Self::RefVar(r)) => l == r,
            (Self::ObjIdx(l), Self::ObjIdx(r)) => l == r,
            (Self::ObjUid(l), Self::ObjUid(r)) => l == r,
            (Self::ObjKey(l), Self::ObjKey(r)) => l == r,
            _ => false
        }
    }
}

impl std::cmp::Eq for Literal {}

/// A possibly-typed buffer of bytes.
#[derive(Clone, PartialEq, Eq)]
pub struct Byt {
    /// The type of the data, or an empty string.
    pub kind: CompactString,
    /// The data.
    pub data: Vec<u8>
}

impl Literal {
    /// Returns the type of the literal as static str.
    pub const fn get_type_str(&self) -> &str {
        match self {
            Literal::Nil => "nil",
            Literal::Bool(_) => "boolean",
            Literal::Int(_) => "integer-number",
            Literal::Dec(_) => "decimal-number",
            Literal::Uid(_) => "unique-identifier",
            Literal::Str(_) => "char-string",
            Literal::Byt(_) => "byte-string",
            Literal::RefRes => "ref-res",
            Literal::RefCtx => "ref-ctx",
            Literal::RefVar(_) => "ref-var",
            Literal::ObjIdx(_) => "obj-idx",
            Literal::ObjUid(_) => "obj-uid",
            Literal::ObjKey(_) => "obj-key",
        }
    }
}

impl std::fmt::Debug for Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Literal::Nil => write!(f, "null"),
            Literal::Bool(true) => write!(f, "true"),
            Literal::Bool(false) => write!(f, "false"),
            Literal::Int(v) => write!(f, "{v}i"),
            Literal::Dec(v) => write!(f, "{v}f"),
            Literal::Uid(v) => write!(f, "U{v}"),
            Literal::Str(v) => write!(f, "{}", bareword_format(v)),
            Literal::Byt(v) => {
                write!(f, "0x[")?;
                let mut tail = false;
                for byte in &v.data {
                    if tail {
                        write!(f, " ")?;
                    }
                    write!(f, "{byte:02X}")?;
                    tail = true;
                }
                write!(f, "]")
            },
            Literal::RefRes => write!(f, "$"),
            Literal::RefCtx => write!(f, "$$"),
            Literal::RefVar(v) => write!(f, "${v}"),
            Literal::ObjIdx(v) => write!(f, "@{v}"),
            Literal::ObjUid(v) => write!(f, "@{v}"),
            Literal::ObjKey(v) => write!(f, "@{}", bareword_format(v)),
        }
    }
}

/// Format the given string as bareword, if possible.
pub fn bareword_format(input: &str) -> Cow<str> {
    for (i, ch) in input.char_indices() {
        if i == 0 {
            if is_bareword_start(ch) {
                continue;
            }
        } else if is_bareword_part(ch) {
            continue;
        }
        
        let mut escaped = String::with_capacity(input.len()+3);
        escaped.push('"');
        escaped.push_str(&input[..i]);
        for ch in input[i..].chars() {
            if ch == '"' {
                escaped.push_str("\\\"");
            } else {
                escaped.push(ch);
            }
        }
        escaped.push('"');
        return Cow::Owned(escaped);
    };
    
    Cow::Borrowed(input)
}

/// Check if the given string is a bareword.
pub fn is_bareword(input: &str) -> bool {
    for (i, ch) in input.char_indices() {
        if i == 0 {
            if is_bareword_start(ch) {
                continue;
            }
        } else if is_bareword_part(ch) {
            continue;
        }
        
        return false
    }
    
    true
}

/// Check if the given character indicates the start of a bareword.
pub fn is_bareword_start(ch: char) -> bool {
    ch.is_alphabetic() || ch == '_'
}

/// Check if the given character may be part of a bareword.
pub fn is_bareword_part(ch: char) -> bool {
    ch.is_alphanumeric() || ch == '_' || ch == '-'
}
