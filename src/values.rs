//! Runtime value representation using NaN-Tagging.
#![allow(missing_docs)]

use std::{convert::TryFrom, fmt::Debug, marker::PhantomData};
use rustc_hash::FxHashMap;
use tagged_box::{tagged_box, TaggableContainer};
use smartstring::alias::CompactString;
use crate::parser::Invoke;

#[derive(Debug, Clone, PartialEq)]
#[repr(transparent)]
/// A by-name reference to a global variable.
pub struct GlobalVar(pub CompactString);

#[derive(Debug, Clone, PartialEq)]
#[repr(transparent)]
/// A by-name reference to a local variable.
pub struct LocalVar(pub CompactString);

tagged_box! {
    /// A NaN-tagged container for values at runtime.
    #[derive(Clone)]
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
        ContextVar(PhantomData<()>),
        Bytes(Vec<u8>),
        List(Vec<ValContainer>),
        Map(FxHashMap<CompactString, ValContainer>),
        Invoke(Box<Invoke>),
        //Dyn(Box<dyn std::any::Any + PartialEq<dyn std::any::Any>>),
    }
}

impl PartialEq for ValItem {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Nothing(l0), Self::Nothing(r0)) => l0 == r0,
            (Self::Decimal(l0), Self::Decimal(r0)) => l0 == r0,
            (Self::Integer(l0), Self::Integer(r0)) => l0 == r0,
            (Self::Boolean(l0), Self::Boolean(r0)) => l0 == r0,
            (Self::String(l0), Self::String(r0)) => l0 == r0,
            (Self::GlobalVar(l0), Self::GlobalVar(r0)) => l0 == r0,
            (Self::LocalVar(l0), Self::LocalVar(r0)) => l0 == r0,
            (Self::ResultVar(l0), Self::ResultVar(r0)) => l0 == r0,
            (Self::ContextVar(l0), Self::ContextVar(r0)) => l0 == r0,
            (Self::Bytes(l0), Self::Bytes(r0)) => l0 == r0,
            //(Self::List(l0), Self::List(r0)) => l0 == r0,
            //(Self::Map(l0), Self::Map(r0)) => l0 == r0,
            (Self::Invoke(l0), Self::Invoke(r0)) => l0 == r0,
            _ => false
        }
    }
}

impl std::fmt::Debug for ValItem {
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
            ValItem::ContextVar(_) => write!(f, "$"),
            ValItem::Bytes(_v) => write!(f, "Bytes"),
            ValItem::List(v) => {
                write!(f, "[")?;
                for i in v {
                    write!(f, "{:?} ", i)?;
                }
                write!(f, "]")?;
                Ok(())
            },
            ValItem::Map(v) => {
                write!(f, "{{")?;
                for (k, v) in v {
                    write!(f, "{} = {:?} ", k, v)?;
                }
                write!(f, "}}")?;
                Ok(())
            },
            ValItem::Invoke(v) => write!(f, "{:?}", v),
        }
    }
}

impl std::fmt::Debug for ValContainer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self.clone().into_inner(), f)
    }
}

impl From<&crate::lexer::Literal> for ValContainer {
    fn from(literal: &crate::lexer::Literal) -> Self {
        use crate::lexer::Literal;
        match literal {
            Literal::Nil => Self::from(()),
            Literal::Bool(v) => Self::from(*v),
            Literal::Char(v) => Self::from(*v as i32),
            Literal::Int(v) => Self::from(*v as i32),
            Literal::Dec(v) => Self::from(*v),
            Literal::Str(v) => Self::from(v.clone()),
            Literal::Byt(v) => Self::from(v.clone()),
        }
    }
}

impl From<&crate::parser::Expression> for ValContainer {
    fn from(expr: &crate::parser::Expression) -> Self {
        use crate::lexer::Literal;
        use crate::parser::Expression;
        use crate::parser::ReferenceRoot;
        use crate::parser::Structure;
        Self::from(match expr {
            Expression::Invoke(i) => ValItem::Invoke(i.clone()),
            Expression::Value(Literal::Nil) => ValItem::Nothing(()),
            Expression::Value(Literal::Bool(v)) => ValItem::Boolean(*v),
            Expression::Value(Literal::Char(v)) => ValItem::Integer(*v as i32),
            Expression::Value(Literal::Int(v)) => ValItem::Integer(*v as i32),
            Expression::Value(Literal::Dec(v)) => ValItem::Decimal(*v),
            Expression::Value(Literal::Str(v)) => ValItem::String(v.clone()),
            Expression::Value(Literal::Byt(v)) => ValItem::Bytes(v.clone()),
            Expression::Structure(Structure::List(l)) => ValItem::List(l.iter().map(|i| i.into()).collect()),
            Expression::Structure(Structure::Dict(d)) => ValItem::Map(d.iter().map(|(k,v)| (k.clone(),v.into())).collect()),
            Expression::Reference(ReferenceRoot::Ctx) => ValItem::ContextVar(PhantomData::default()),
            Expression::Reference(ReferenceRoot::Res) => ValItem::ResultVar(PhantomData::default()),
            Expression::Reference(ReferenceRoot::Local(l)) => ValItem::LocalVar(LocalVar(l.clone())),
            Expression::Reference(ReferenceRoot::Global(g)) => ValItem::GlobalVar(GlobalVar(g.clone())),
        })
    }
}

impl From<&str> for ValContainer {
    fn from(str: &str) -> Self {
        Self::from(ValItem::String(str.into()))
    }
}

impl TryFrom<&ValContainer> for i32 {
    type Error = &'static str;
    
    fn try_from(val: &ValContainer) -> Result<Self, Self::Error> {
        let mut res = Err("uninitialized");
        unsafe {
            tagged_box::TaggableInner::ref_from_tagged_box(&val.value, |val| {
                res = match val {
                    ValItem::Nothing(_) => Err("unable to cast nothing to i32"),
                    ValItem::Decimal(d) => Ok((*d).floor() as Self),
                    ValItem::Integer(i) => Ok(*i),
                    ValItem::Boolean(_) => Err("unable to cast bool to i32"),
                    ValItem::String(_) => Err("unable to cast string to i32"),
                    ValItem::GlobalVar(_) => Err("unable to cast global-var to i32"),
                    ValItem::LocalVar(_) => Err("unable to cast local-var to i32"),
                    ValItem::ResultVar(_) => Err("unable to cast result-var to i32"),
                    ValItem::ContextVar(_) => Err("unable to cast context-var to i32"),
                    ValItem::Bytes(_) => Err("unable to cast bytes to i32"),
                    ValItem::List(_) => Err("unable to cast list to i32"),
                    ValItem::Map(_) => Err("unable to cast map to i32"),
                    ValItem::Invoke(_) => Err("unable to cast invocation to i32"),
                }
            })
        };
        
        res
    }
}

impl TryFrom<&ValContainer> for f64 {
    type Error = &'static str;
    
    fn try_from(val: &ValContainer) -> Result<Self, Self::Error> {
        let mut res = Err("uninitialized");
        unsafe {
            tagged_box::TaggableInner::ref_from_tagged_box(&val.value, |val| {
                res = match val {
                    ValItem::Nothing(_) => Err("unable to cast nothing to i32"),
                    ValItem::Decimal(d) => Ok(*d),
                    ValItem::Integer(i) => Ok(*i as Self),
                    ValItem::Boolean(_) => Err("unable to cast bool to i32"),
                    ValItem::String(_) => Err("unable to cast string to i32"),
                    ValItem::GlobalVar(_) => Err("unable to cast global-var to i32"),
                    ValItem::LocalVar(_) => Err("unable to cast local-var to i32"),
                    ValItem::ResultVar(_) => Err("unable to cast result-var to i32"),
                    ValItem::ContextVar(_) => Err("unable to cast context-var to i32"),
                    ValItem::Bytes(_) => Err("unable to cast bytes to i32"),
                    ValItem::List(_) => Err("unable to cast list to i32"),
                    ValItem::Map(_) => Err("unable to cast map to i32"),
                    ValItem::Invoke(_) => Err("unable to cast invocation to i32"),
                }
            });
        };
        
        res
    }
}

impl From<&ValContainer> for String {
    fn from(val: &ValContainer) -> Self {
        format!("{:?}", &val)
    }
}

impl TryFrom<&ValContainer> for Vec<ValContainer> {
    type Error = &'static str;
    
    fn try_from(val: &ValContainer) -> Result<Self, Self::Error> {
        use std::iter::once;
        
        let mut res = Err("uninitialized");
        unsafe {
            tagged_box::TaggableInner::ref_from_tagged_box(&val.value, |val| {
                res = match val {
                    ValItem::Nothing(_) => Ok(Default::default()),
                    ValItem::Decimal(_) => Err("unable to cast decimal to list"),
                    ValItem::Integer(_) => Err("unable to cast decimal to list"),
                    ValItem::Boolean(_) => Err("unable to cast bool to list"),
                    ValItem::String(_) => Err("unable to cast string to list"),
                    ValItem::GlobalVar(_) => Err("unable to cast global-var to list"),
                    ValItem::LocalVar(_) => Err("unable to cast local-var to list"),
                    ValItem::ResultVar(_) => Err("unable to cast result-var to list"),
                    ValItem::ContextVar(_) => Err("unable to cast context-var to list"),
                    
                    ValItem::Bytes(bytes) => Ok(bytes.iter().map(|b| ValContainer::from(*b as i32)).collect()),
                    
                    ValItem::List(list) => Ok(list.clone()),
                    
                    ValItem::Map(map) => {
                        Ok(map
                            .iter()
                            .map(|(k,v)| once(ValItem::String(k.clone()).into()).chain(once(v.clone())))
                            .flatten()
                            .collect()
                        )
                    },
                    
                    ValItem::Invoke(cmd) => {
                        let pos_args = cmd.pos_args
                            .iter()
                            .map(|arg| arg.into())
                            .collect::<Vec<ValContainer>>()
                            .into();
                        
                        let nom_args = cmd.nom_args
                            .iter()
                            .map(|(k,v)| once(ValContainer::from(k.clone())).chain(once(v.into())))
                            .flatten()
                            .collect::<Vec<ValContainer>>()
                            .into();
                        
                        Ok(vec![
                            cmd.name.clone().into(),
                            pos_args,
                            nom_args,
                        ])
                    },
                };
            })
        };
        
        res
    }
}
