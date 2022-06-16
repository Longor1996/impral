//! The abstract-syntax-tree.

use super::*;

/// A expression node.
#[derive(Clone, PartialEq)]
pub enum Expression {
    /// Nothing.
    Empty,
    
    /// A value: A lone piece of data.
    Value(Literal),
    
    /// A command.
    Invoke(Box<Invoke>),
    
    /// A field access on the left expression.
    Field(Box<Expression>, CompactString),
    
    /// A index access on the left expression.
    Index(Box<Expression>, Box<Expression>),
    
    /// Unwrap the result; throwing on error if bool is `true`.
    Try(Box<Expression>, bool),
    
    /// A pipe.
    Pipe(Box<Pipe>),
}

/// A (small)vec of expression nodes.
pub type ExpressionVec = SmallVec<[Expression; 1]>;

/// A command (-node) to be evaluated.
#[derive(Clone, Default, PartialEq)]
pub struct Invoke {
    /// The name of the command.
    pub name: CompactString,
    
    /// The positional arguments.
    ///
    /// As long as there is only one positional argument, there will be no direct heap allocation.
    pub pos_args: ExpressionVec,
    
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
            Expression::Empty => write!(f, "_"),
            Expression::Value(l) => std::fmt::Debug::fmt(l, f),
            Expression::Invoke(c) => write!(f, "({:?})", c),
            Expression::Field(e, i) => write!(f, "{e:?}.{}", bareword_format(i)),
            Expression::Index(e, i) => write!(f, "{e:?}[{i:?}]"),
            Expression::Try(e, t) => if *t {
                write!(f, "{e:?}?!")
            } else {
                write!(f, "{e:?}?")
            },
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

