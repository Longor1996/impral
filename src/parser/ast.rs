//! The abstract-syntax-tree.

use super::*;

/// A expression node.
#[derive(Clone, PartialEq)]
pub enum Expression {
    /// Nothing; an empty slot.
    Empty,
    
    /// A value.
    Value(Literal),
    
    /// A function call / command invocation.
    FnCall(Box<FnCall>),
    
    /// A range from START to END, maybe INCLUSIVE.
    Range(Box<Expression>, Box<Expression>, bool),
    
    /// A field/member access on the left expression.
    Field(Box<Expression>, CompactString),
    
    /// A index/array access on the left expression.
    Index(Box<Expression>, Box<Expression>),
    
    /// A method call on the left expression.
    Method(Box<Expression>, Box<FnCall>),
    
    /// Unwrap the left expression; throwing an error if bool is `true`.
    Try(Box<Expression>, bool),
    
    /// A pipe / generator / iterator.
    Pipe(Box<Pipe>),
}

impl Default for Expression {
    fn default() -> Self {
        Self::Empty
    }
}

/// A (small)vec of expression nodes.
pub type ExpressionVec = SmallVec<[Expression; 1]>;

/// A function call (-node) to be evaluated; created via [`crate::parser::command::parse_command`].
#[derive(Clone, Default, PartialEq)]
pub struct FnCall {
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
pub enum PipeSeg {
    /// Collect the pipe into a data-structure, possibly depending on the receiver expression.
    Collect,
    
    /// A thing goes in, a(nother) thing comes out.
    Mapping {
        /// An expression with a `$`-val in it, hopefully.
        mapper: Expression,
    },
    
    /// Given an `initial` value, use the `reducer` to fold the pipe into 'one' value.
    Folding {
        /// The initial value for the reducer.
        initial: Expression,
        /// The expression that receives a `$`-val as input and a `$!`-val as accumulator/state.
        reducer: Expression,
    },
    
    /// Filtering out elements given a `predicate`.
    Exclude {
        /// An expression that returns a truthy/falsy value.
        predicate: Expression
    },
}

// ----------------------------------------------------------------------------



impl std::fmt::Debug for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expression::Empty => write!(f, "_"),
            Expression::Value(l) => std::fmt::Debug::fmt(l, f),
            Expression::FnCall(c) => write!(f, "({:?})", c),
            Expression::Field(e, i) => write!(f, "{e:?}.{}", bareword_format(i)),
            Expression::Index(e, i) => write!(f, "{e:?}.[{i:?}]"),
            Expression::Method(e, i) => write!(f, "{e:?}.({i:?})"),
            Expression::Range(s, e, inc) => if *inc {
                write!(f, "{s:?}..={e:?}")
            } else {
                write!(f, "{s:?}..{e:?}")
            },
            Expression::Try(e, t) => if *t {
                write!(f, "{e:?}?!")
            } else {
                write!(f, "{e:?}?")
            },
            Expression::Pipe(p) => write!(f, "{:?}", p),
        }
    }
}

impl From<FnCall> for Expression {
    fn from(i: FnCall) -> Self {
        Self::FnCall(i.into())
    }
}

impl std::fmt::Debug for FnCall {
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
            write!(f, " |")?; // all segments start with a `|`
            match seg {
                PipeSeg::Collect => write!(f, "!")?,
                PipeSeg::Mapping { mapper } => write!(f, " {mapper:?}")?,
                PipeSeg::Folding { initial, reducer } => write!(f, "! {initial:?} {reducer:?}")?,
                PipeSeg::Exclude { predicate } => write!(f, "? {predicate:?}")?,
            }
        }
        write!(f, "")
    }
}

#[cfg(any(feature = "html_fmt", test))]
pub mod html;
