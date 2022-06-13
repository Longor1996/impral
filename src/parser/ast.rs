//! The abstract-syntax-tree.

use super::*;

/// A expression node.
#[derive(Clone, PartialEq)]
pub enum Expression {
    /// A value: A lone piece of data.
    Value(Literal),
    
    /// A command.
    Invoke(Box<Invoke>),
    
    /// A dereference.
    Deref(Box<DerefChain>),
    
    /// A pipe.
    Pipe(Box<Pipe>),
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

/// A dereference chain.
#[derive(Clone, PartialEq)]
pub struct DerefChain {
    /// The expression that is being dereferenced.
    pub source: Expression,
    
    /// The segments of the deref chain.
    pub stages: SmallVec<[DerefSeg; 1]>,
}

/// A segment of a dereference chain.
#[derive(Clone, PartialEq)]
pub struct DerefSeg {
    /// If true, the deref may fail.
    pub fallible: bool,
    
    /// This segments deref.
    pub member: CompactString,
}


// ----------------------------------------------------------------------------



impl std::fmt::Debug for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expression::Value(l) => std::fmt::Debug::fmt(l, f),
            Expression::Invoke(c) => write!(f, "({:?})", c),
            Expression::Deref(d) => write!(f, "{:?}", d),
            Expression::Pipe(p) => write!(f, "{:?}", p),
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

impl std::fmt::Debug for Pipe {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.source)?;
        for seg in &self.stages {
            write!(f, " |")?;
            if seg.filter {write!(f, "?")?}
            write!(f, " {:?}", seg.invoke)?
        }
        write!(f, "")
    }
}

impl std::fmt::Debug for DerefChain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.source)?;
        for seg in &self.stages {
            write!(f, ".")?;
            if seg.fallible { write!(f, "?")? }
            write!(f, "{}", seg.member)?
        }
        Ok(())
    }
}
