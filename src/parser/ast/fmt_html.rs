//! HTML formatter for [`Block`]s
use super::*;
use std::fmt::*;
use crate::lexer::bareword_format;

/// Prints blocks with some formatter.
pub struct BlockHtmlPrinter<'b> {
    pub(crate) block: &'b Block
}

impl<'b> From<&'b Block> for BlockHtmlPrinter<'b> {
    fn from(block: &'b Block) -> Self {
        Self {block}
    }
}

/// The type to write to.
pub type HtmlOutput = String;

impl BlockHtmlPrinter<'_> {
    /// Formats the block as html.
    pub fn to_string(&self) -> std::result::Result<String, std::fmt::Error> {
        let mut str = String::new();
        self.fmt(&mut str)?;
        Ok(str)
    }
    
    /// Formats the block as html.
    pub fn fmt(&self, f: &mut HtmlOutput) -> std::fmt::Result {
        let last = self.block.entry.unwrap_or_else(|| self.block.last().unwrap());
        write!(f, "<span class=expression>")?;
        self.fmt_ref(f, last, true)?;
        write!(f, "</span>")?;
        Ok(())
    }
    
    fn fmt_ref(&self, f: &mut HtmlOutput, br: BlockRef, in_place: bool) -> std::fmt::Result {
        write!(f, "<span class=expression title='#{br:?}'>")?;
        match self.block.get(br) {
            Expression::Empty => write!(f, "<span class=empty>_</span>")?,
            
            Expression::Value(lit) => self.fmt_lit(f, lit)?,
            
            Expression::Range(start, end, inclusive) => {
                write!(f, "<span class=range>")?;
                write!(f,  "<span class=start>")?;
                self.fmt_ref(f, *start, false)?;
                write!(f,  "</span>")?;
                if *inclusive {
                    write!(f, "<span class=operator>..=</span>")?;
                } else {
                    write!(f, "<span class=operator>..</span>")?;
                }
                write!(f,  "<span class=end>")?;
                self.fmt_ref(f, *end, false)?;
                write!(f,  "</span>")?;
                write!(f, "</span>")?;
            },
            
            Expression::Try(target, abort) => {
                write!(f, "<span class=range>")?;
                write!(f,  "<span class=target>")?;
                self.fmt_ref(f, *target, false)?;
                write!(f,  "</span>")?;
                write!(f,  "<span class=operator>?</span>")?;
                if *abort {write!(f, "<span class=operator>!</span>")?}
                write!(f, "</span>")?;
            },
            
            Expression::Field(target, member) => {
                write!(f, "<span class=field>")?;
                write!(f,  "<span class=target>")?;
                self.fmt_ref(f, *target, false)?;
                write!(f,  "</span>")?;
                write!(f,  "<span class=operator>.</span>")?;
                write!(f,  "<span class=member>{member}</span>")?;
                write!(f, "</span>")?;
            },
            
            Expression::Index(target, member) => {
                write!(f, "<span class=index>")?;
                write!(f,  "<span class=target>")?;
                self.fmt_ref(f, *target, false)?;
                write!(f,  "</span>")?;
                write!(f,  "<span class=operator>.</span>")?;
                write!(f,  "<span class=separator>[</span>")?;
                write!(f,  "<span class=member>")?;
                self.fmt_ref(f, *member, false)?;
                write!(f,  "</span>")?;
                write!(f,  "<span class=separator>]</span>")?;
                write!(f, "</span>")?;
            },
            
            Expression::Method(target, member) => {
                write!(f, "<span class=method>")?;
                write!(f,  "<span class=target>")?;
                self.fmt_ref(f, *target, false)?;
                write!(f,  "</span>")?;
                write!(f,  "<span class=operator>.</span>")?;
                write!(f,  "<span class=separator>(</span>")?;
                write!(f,  "<span class=invoke>")?;
                self.fmt_ref(f, *member, true)?;
                write!(f,  "</span>")?;
                write!(f,  "<span class=separator>)</span>")?;
                write!(f, "</span>")?;
            }
            
            Expression::FnCall(call) => {
                write!(f, "<span class=invoke>")?;
                if !in_place {write!(f, "<span class=separator>(</span>")?}
                write!(f,  "<span class=name>{}</span>", call.name)?;
                for arg in call.pos_args.iter() {
                    write!(f, " <span class=val>")?;
                    self.fmt_ref(f, *arg, false)?;
                    write!(f, "</span>")?;
                }
                for (key, arg) in call.nom_args.iter() {
                    write!(f, " <span class=key-val>")?;
                    write!(f,  "<span class=key>{key}</span>")?;
                    write!(f,  "<span class=separator>=</span>")?;
                    write!(f,  "<span class=val>")?;
                    self.fmt_ref(f, *arg, false)?;
                    write!(f,  "</span>")?;
                    write!(f, "</span>")?;
                }
                if !in_place {write!(f, "<span class=separator>)</span>")?}
                write!(f, "</span>")?;
            },
            
            Expression::Pipe(pipe) => {
                write!(f, "<span class=pipe>")?;
                write!(f,  "<span class=source>")?;
                self.fmt_ref(f, pipe.source, true)?;
                write!(f,  "</span>")?;
                for seg in pipe.stages.iter() {
                    write!(f, " <span class=separator>|</span>")?;
                    match seg {
                        PipeSeg::Collect { collector } => {
                            write!(f, "<span class='segment collect'>")?;
                            write!(f, "<span class='operator collect'>&gt;</span> ")?;
                            self.fmt_ref(f, *collector, true)?;
                            write!(f, "</span>")?;
                        },
                        PipeSeg::Mapping { mapper } => {
                            write!(f, " <span class='segment mapping'>")?;
                            self.fmt_ref(f, *mapper, true)?;
                            write!(f, "</span>")?;
                        },
                        PipeSeg::Folding { initial, reducer } => {
                            write!(f, "<span class='segment folding'>")?;
                            write!(f,  "<span class=operator>!</span> ")?;
                            self.fmt_ref(f, *initial, false)?;
                            write!(f,  "&nbsp;")?;
                            self.fmt_ref(f, *reducer, true)?;
                            write!(f, "</span>")?;
                        },
                        PipeSeg::Exclude { predicate } => {
                            write!(f, "<span class='segment exclude'>")?;
                            write!(f,  "<span class=operator>?</span> ")?;
                            self.fmt_ref(f, *predicate, true)?;
                            write!(f, "</span>")?;
                        },
                        PipeSeg::Finding { predicate } => {
                            write!(f, "<span class='segment finding'>")?;
                            write!(f,  "<span class=operator>?!</span> ")?;
                            self.fmt_ref(f, *predicate, true)?;
                            write!(f, "</span>")?;
                        },
                    };
                }
                write!(f, "</span>")?;
            },
        };
        write!(f, "</span>")
    }
    
    fn fmt_lit(&self, f: &mut String, lit: &Literal) -> std::fmt::Result {
        match lit {
            Literal::Nil => write!(f, "<span class='literal null'>null</span>"),
            Literal::Bool(l) => write!(f, "<span class='literal bool'>{l:?}</span>"),
            Literal::Int(l)  => write!(f, "<span class='literal int'>{l:?}</span>"),
            Literal::Dec(l)  => write!(f, "<span class='literal dec'>{l:?}</span>"),
            Literal::Uid(l)  => write!(f, "<span class='literal uid'>U{l:?}</span>"),
            Literal::Str(l)  => write!(f, "<span class='literal str'>{}</span>", bareword_format(l)),
            Literal::Byt(_l) => write!(f, "<span class='literal byt'>BINARY DATA</span>"),
            Literal::RefRes  => write!(f, "<span class='literal ref-res'>$</span>"),
            Literal::RefCtx  => write!(f, "<span class='literal ref-ctx'>$$</span>"),
            Literal::RefVar(l) => write!(f, "<span class='literal ref-var'>${}</span>", bareword_format(l)),
            Literal::ObjIdx(l) => write!(f, "<span class='literal obj-idx'>@{l:?}</span>"),
            Literal::ObjUid(l) => write!(f, "<span class='literal obj-uid'>@{l:?}</span>"),
            Literal::ObjKey(l) => write!(f, "<span class='literal obj-key'>@{}</span>", bareword_format(l)),
        }
    }
}
