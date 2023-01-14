//! Debug formatter for [`Block`]s
use super::*;

/// Prints blocks with some formatter.
pub struct BlockDebugPrinter<'b> {
    pub(crate) block: &'b Block
}

impl Debug for BlockDebugPrinter<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Block {{")?;
        if !self.block.is_empty() {
            write!(f, "vcount: {}, ", self.block.items.iter().filter(|i|matches!(i, Expression::Empty | Expression::Value(_))).count())?;
            write!(f, "ecount: {}, ", self.block.items.iter().filter(|i|!matches!(i, Expression::Empty | Expression::Value(_))).count())?;
            write!(f, "entry: {:?}, ", &self.block.entry)?;
            write!(f, "tree: `")?;
            self.fmt_ref(f, self.block.entry.unwrap_or_else(|| self.block.last().unwrap()))?;
            write!(f, "`")?;
        } else {
            write!(f, "empty")?;
        }
        write!(f, "}}")?;
        Ok(())
    }
}

impl BlockDebugPrinter<'_> {
    
    fn fmt_ref(&self, f: &mut std::fmt::Formatter<'_>, br: BlockRef) -> std::fmt::Result {
        match self.block.get(br) {
            Expression::Empty => write!(f, "_"),
            Expression::Value(l) => std::fmt::Debug::fmt(l, f),
            Expression::FnCall(c) => {
                write!(f, "({}", c.name)?;
                for arg in &c.pos_args {
                    write!(f, " ")?;
                    self.fmt_ref(f, *arg)?;
                }
                for (key, arg) in &c.nom_args {
                    write!(f, " {}=", key)?;
                    self.fmt_ref(f, *arg)?;
                }
                write!(f, ")")
            },
            Expression::Field(e, i) => {
                self.fmt_ref(f, *e)?;
                write!(f, ".{}", bareword_format(i))
            },
            Expression::Index(e, i) => {
                self.fmt_ref(f, *e)?;
                write!(f, ".[")?;
                self.fmt_ref(f, *i)?;
                write!(f, "]")
            },
            Expression::Method(e, i) => {
                self.fmt_ref(f, *e)?;
                write!(f, ".(")?;
                self.fmt_ref(f, *i)?;
                write!(f, ")")
            },
            Expression::Range(s, e, inc) => {
                self.fmt_ref(f, *s)?;
                if *inc {write!(f, "=")?}
                self.fmt_ref(f, *e)
            },
            Expression::Try(e, t) => {
                self.fmt_ref(f, *e)?;
                write!(f, "?")?;
                if *t {write!(f, "?!")?}
                Ok(())
            },
            Expression::Pipe(p) => {
                self.fmt_ref(f, p.source)?;
                for seg in &p.stages {
                    write!(f, " |")?; // all segments start with a `|`
                    match seg {
                        PipeSeg::Collect { collector } => {
                            write!(f, "> ")?;
                            self.fmt_ref(f, *collector)?;
                        },
                        PipeSeg::Mapping { mapper } => {
                            write!(f, " ")?;
                            self.fmt_ref(f, *mapper)?;
                        },
                        PipeSeg::Folding { initial, reducer } => {
                            write!(f, "! ")?;
                            self.fmt_ref(f, *initial)?;
                            write!(f, " ")?;
                            self.fmt_ref(f, *reducer)?;
                        },
                        PipeSeg::Exclude { predicate } => {
                            write!(f, "? ")?;
                            self.fmt_ref(f, *predicate)?;
                        },
                        PipeSeg::Finding { predicate } => {
                            write!(f, "?! ")?;
                            self.fmt_ref(f, *predicate)?;
                        },
                    }
                }
                write!(f, "")
            },
        }
    }
    
}

impl<'b> From<&'b Block> for BlockDebugPrinter<'b> {
    fn from(block: &'b Block) -> Self {
        Self {block}
    }
}