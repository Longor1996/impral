//! The abstract-syntax-tree.

use super::*;

/// A linearized tree of expression nodes.
pub struct Block {
    /// Expression items table.
    pub(crate) items: Vec<Expression>,
    
    /// Expression spans table.
    pub(crate) spans: Vec<std::ops::Range<usize>>,
    
    /// Entrypoint
    pub(crate) entry: Option<BlockRef>,
}

/// A block-internal reference.
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct BlockRef(usize);

impl Debug for BlockRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "#{}", self.0)
    }
}

impl Default for Block {
    fn default() -> Self {
        Self {
            items: Vec::with_capacity(16),
            spans: Vec::with_capacity(16),
            entry: None
        }
    }
}

impl Block {
    /// Emplaces an expression into the block, returning a BlockRef.
    pub fn emplace(&mut self, expr: Expression, span: std::ops::Range<usize>) -> BlockRef {
        if let Expression::Value(val) = &expr {
            if let Some(r) = self.items.iter()
                .enumerate()
                .filter_map(|(i,v)|
                    if let Expression::Value(v) = v {
                        Some((i,v))
                    } else {None}
                )
                .find(|(_, v)| v == &val)
                .map(|(i,_)| i) {
                return BlockRef(r)
            }
        }
        
        let blockref = self.items.len();
        self.items.push(expr);
        self.spans.push(span);
        BlockRef(blockref)
    }
    
    /// Return an emplaced expression.
    pub fn get(&self, br: BlockRef) -> &Expression {
        &self.items[br.0 as usize]
    }
    
    /// Return an emplaced expression.
    pub fn get_mut(&mut self, br: BlockRef) -> &mut Expression {
        &mut self.items[br.0 as usize]
    }
    
    /// Returns the [`BlockRef`] for the last item.
    pub fn last(&self) -> Option<BlockRef> {
        if self.items.is_empty() {None}
        else {Some(BlockRef(self.items.len() -1))}
    }
    
    /// Returns true if the block is empty.
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
    
}

impl std::fmt::Debug for Block {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt_debug::BlockDebugPrinter::from(self).fmt(f)
    }
}

pub mod fmt_debug;

/// A expression node.
#[derive(Clone, PartialEq, Eq)]
pub enum Expression {
    /// Nothing; an empty slot.
    Empty,
    
    /// A reference into the block (linearized tree).
    //BlockRef(BlockRef),
    
    /// A value.
    Value(Literal),
    
    /// A function call / command invocation.
    FnCall(Box<FnCall>),
    
    /// A range from START to END, maybe INCLUSIVE.
    Range(BlockRef, BlockRef, bool),
    
    /// A field/member access on the left expression.
    Field(BlockRef, CompactString),
    
    /// A index/array access on the left expression.
    Index(BlockRef, BlockRef),
    
    /// A method call on the left expression.
    Method(BlockRef, BlockRef),
    
    /// Unwrap the left expression; throwing an error if bool is `true`.
    Try(BlockRef, bool),
    
    /// A pipe / generator / iterator.
    Pipe(Box<Pipe>),
}

impl Default for Expression {
    fn default() -> Self {
        Self::Empty
    }
}

/// A (small)vec of expression nodes.
pub type ExpressionVec = SmallVec<[BlockRef; 1]>;

/// A function call (-node) to be evaluated; created via [`crate::parser::command::parse_command`].
#[derive(Clone, Default, PartialEq, Eq)]
pub struct FnCall {
    /// The name of the command.
    pub name: CompactString,
    
    /// The positional arguments.
    ///
    /// As long as there is only one positional argument, there will be no direct heap allocation.
    pub pos_args: ExpressionVec,
    
    /// The nominal/named arguments.
    pub nom_args: FxHashMap<CompactString, BlockRef>,
}

/// A pipe.
#[derive(Clone, PartialEq, Eq)]
pub struct Pipe {
    /// The expression yielding pipe items.
    pub source: BlockRef,
    
    /// The segments of the pipe.
    pub stages: Vec<PipeSeg>,
}

/// A segment of a pipe.
#[derive(Clone, PartialEq, Eq)]
pub enum PipeSeg {
    /// Collect the pipe into a data-structure, depending on the receiver expression.
    Collect {
        /// An expression with a `$`-val in it, hopefully.
        collector: BlockRef,
    },
    
    /// A thing goes in, a(nother) thing comes out.
    Mapping {
        /// An expression with a `$`-val in it, hopefully.
        mapper: BlockRef,
    },
    
    /// Given an `initial` value, use the `reducer` to fold the pipe into 'one' value.
    Folding {
        /// The initial value for the reducer.
        initial: BlockRef,
        /// The expression that receives a `$`-val as input and a `$!`-val as accumulator/state.
        reducer: BlockRef,
    },
    
    /// Filtering out elements given a `predicate`.
    Exclude {
        /// An expression that returns a truthy/falsy value.
        predicate: BlockRef
    },
    
    /// Return the first item for which the given `predicate` is true.
    Finding {
        /// An expression that returns a truthy/falsy value.
        predicate: BlockRef
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
                PipeSeg::Collect { collector } => write!(f, "> {collector:?}")?,
                PipeSeg::Mapping { mapper } => write!(f, " {mapper:?}")?,
                PipeSeg::Folding { initial, reducer } => write!(f, "! {initial:?} {reducer:?}")?,
                PipeSeg::Exclude { predicate } => write!(f, "? {predicate:?}")?,
                PipeSeg::Finding { predicate } => write!(f, "?! {predicate:?}")?,
            }
        }
        write!(f, "")
    }
}

#[cfg(any(feature = "html_fmt", test))]
pub mod fmt_html;
