//! Value representation.
#![allow(missing_docs)]

use std::{fmt::Debug, marker::PhantomData};

use rustc_hash::FxHashMap;
use tagged_box::{tagged_box, TaggableContainer};
use smartstring::alias::CompactString;
use crate::parser::Invoke;

#[derive(Debug, Clone, PartialEq)]
#[repr(transparent)]
pub struct GlobalVar(pub CompactString);

#[derive(Debug, Clone, PartialEq)]
#[repr(transparent)]
pub struct LocalVar(pub CompactString);

tagged_box! {
    /// A NaN-tagged container for values.
    #[derive(Debug, Clone, PartialEq)]
    pub struct ValContainer,
    pub enum ValItem {
        Nothing(()),
        Decimal(f64),
        Integer(i32),
        Boolean(bool),
        String(CompactString),
        GlobalVar(GlobalVar),
        LocalVar(LocalVar),
        ResultVar(PhantomData<Result<(),()>>),
        Bytes(Vec<u8>),
        List(Vec<ValContainer>),
        Map(FxHashMap<CompactString, ValContainer>),
        Invoke(Invoke),
        //Dyn(Box<dyn std::any::Any + PartialEq<dyn std::any::Any>>),
    }
}

impl std::fmt::Display for ValItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValItem::Nothing(_v) => write!(f, "Nil"),
            ValItem::Decimal(v) => write!(f, "{}", v),
            ValItem::Integer(v) => write!(f, "{}", v),
            ValItem::Boolean(v) => write!(f, "{}", v),
            ValItem::String(v) => write!(f, "\"{}\"", v),
            ValItem::GlobalVar(v) => write!(f, "@{}", v.0),
            ValItem::LocalVar(v) => write!(f, "${}", v.0),
            ValItem::ResultVar(_) => write!(f, "$$"),
            ValItem::Bytes(_v) => write!(f, "Bytes"),
            ValItem::List(v) => {
                write!(f, "[")?;
                for i in v {
                    write!(f, "{} ", i)?;
                }
                write!(f, "]")?;
                Ok(())
            },
            ValItem::Map(v) => {
                write!(f, "{{")?;
                for (k, v) in v {
                    write!(f, "{} = {} ", k, v)?;
                }
                write!(f, "}}")?;
                Ok(())
            },
            ValItem::Invoke(v) => write!(f, "{:?}", v),
        }
    }
}

impl std::fmt::Display for ValContainer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.clone().into_inner(), f)
    }
}

impl From<crate::lexer::Literal> for ValContainer {
    fn from(literal: crate::lexer::Literal) -> Self {
        use crate::lexer::Literal;
        match literal {
            Literal::Nil => Self::from(()),
            Literal::Bool(v) => Self::from(v),
            Literal::Char(v) => Self::from(v as i32),
            Literal::Int(v) => Self::from(v as i32),
            Literal::Dec(v) => Self::from(v),
            Literal::Str(v) => Self::from(v),
            Literal::Byt(v) => Self::from(v),
        }
    }
}

impl From<crate::parser::Expression> for ValContainer {
    fn from(expr: crate::parser::Expression) -> Self {
        match expr {
            crate::parser::Expression::Value(v) => v,
            crate::parser::Expression::Invoke(i) => Self::from(ValItem::Invoke(*i)),
        }
    }
}

impl From<&str> for ValContainer {
    fn from(str: &str) -> Self {
        Self::from(ValItem::String(str.into()))
    }
}
