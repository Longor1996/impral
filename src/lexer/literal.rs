//! Literal representation.

use smartstring::alias::CompactString;

/// A literal / value.
#[derive(Clone, PartialEq)]
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
    
    /// Uid (`@67e55044-10b1-426f-9247-bb680e5fe0c8`)
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
    ObjIdx(usize),
    
    /// Object Uid Reference (`@67e55044-10b1-426f-9247-bb680e5fe0c8`)
    ObjUid(uuid::Uuid),
    
    /// Object Key Reference (`@NAME` / `@'NAME'` / `@"NAME"`)
    ObjKey(CompactString)
}

/// A possibly-typed buffer of bytes.
#[derive(Clone, PartialEq)]
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
            Literal::Str(v) => write!(f, "{v}"),
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
            Literal::ObjKey(v) => write!(f, "@\"{v}\""),
        }
    }
}
