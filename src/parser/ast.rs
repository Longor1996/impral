//! The abstract-syntax-tree.

use super::*;

/// A expression node.
#[derive(Clone, PartialEq)]
pub enum Expression {
    /// A value: A lone piece of data.
    Value(Literal),
    
    // TODO: Convert list & dict structures into invokes and erase this variant.
    /// A structure: ???
    Structure(Structure),
    
    /// A command.
    Invoke(Box<Invoke>),
    
    /// A pipe.
    Pipe(Box<Pipe>),
}

/// A (data-)structure node.
#[derive(Clone, PartialEq)]
pub enum Structure {
    /// A list.
    List(Vec<Expression>),
    /// A dict.
    Dict(FxHashMap<CompactString, Expression>),
}

/// A command (-node) to be evaluated.
#[derive(Clone, Default, PartialEq)]
pub struct Invoke {
    /// The name of the command.
    pub name: CompactString,
    
    /// The positional arguments.
    ///
    /// As long as there is only one positional argument, there will be no direct heap allocation.
    pub pos_args: SmallVec<[Expression; 1]>,
    
    /// The nominal/named arguments.
    pub nom_args: FxHashMap<CompactString, Expression>,
}

/// A pipe.
#[derive(Clone, PartialEq)]
pub struct Pipe {
    /// The expression yielding pipe items.
    pub source: Expression,
    
    /// The segments of the pipe.
    pub stages: Vec<PipeSeg>,
}

/// A segment of a pipe.
#[derive(Clone, PartialEq)]
pub struct PipeSeg {
    /// If true, the invoke result is used as filter.
    pub filter: bool,
    
    /// This segments invoke.
    pub invoke: Expression,
}



// ----------------------------------------------------------------------------



impl std::fmt::Debug for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expression::Value(l) => std::fmt::Debug::fmt(l, f),
            Expression::Structure(s) => std::fmt::Debug::fmt(s, f),
            Expression::Invoke(c) => write!(f, "({:?})", c),
            Expression::Pipe(p) => {
                write!(f, "({:?}", p.source)?;
                for seg in &p.stages {
                    write!(f, " |")?;
                    if seg.filter {write!(f, "?")?}
                    write!(f, " {:?}", seg.invoke)?;
                }
                write!(f, ")")
            },
        }
    }
}

impl std::fmt::Debug for Structure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::List(l) => std::fmt::Debug::fmt(l, f),
            Self::Dict(s) => std::fmt::Debug::fmt(s, f),
        }
    }
}

impl From<Invoke> for Expression {
    fn from(i: Invoke) -> Self {
        Self::Invoke(i.into())
    }
}

impl std::fmt::Debug for Invoke {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.name)?;
        for arg in &self.pos_args {
            write!(f, " {:?}", arg)?;
        }
        for (key, arg) in &self.nom_args {
            write!(f, " {}={:?}", key, arg)?;
        }
        write!(f, "")
    }
}
