//! Symbol representation.

use std::fmt::Write;
use smartstring::alias::CompactString;
use strum_macros::{Display, EnumString, IntoStaticStr};

/// A enum of the set of known symbols.
#[derive(Clone, Copy, PartialEq, Eq, Display, EnumString, IntoStaticStr)]
pub enum Symbol {
    /// `(`
    #[strum(to_string="(")]
    ParenLeft,
    
    /// `)`
    #[strum(to_string=")")]
    ParenRight,
    
    /// `[`
    #[strum(to_string="[")]
    BraketLeft,
    
    /// `]`
    #[strum(to_string="]")]
    BraketRight,
    
    /// `{`
    #[strum(to_string="{")]
    CurlyLeft,
    
    /// `}`
    #[strum(to_string="}")]
    CurlyRight,
    
    /// `<`
    #[strum(to_string="<")]
    AngleLeft,
    
    /// `>`
    #[strum(to_string=">")]
    AngleRight,
    
    /// `+`
    #[strum(to_string="+")]
    Plus,
    
    /// `-`
    #[strum(to_string="-")]
    Dash,
    
    /// `*`
    #[strum(to_string="*")]
    Star,
    
    /// `#`
    #[strum(to_string="#")]
    Hash,
    
    /// `/`
    #[strum(to_string="/")]
    Slash,
    
    /// `~`
    #[strum(to_string="~")]
    Tilde,
    
    /// `,`
    #[strum(to_string=",")]
    Comma,
    
    /// `.`
    #[strum(to_string=".")]
    Dot,
    
    /// `:`
    #[strum(to_string=":")]
    DoubleDot,
    
    /// `;`
    #[strum(to_string=";")]
    Semicolon,
    
    /// `_`
    #[strum(to_string="_")]
    Underscore,
    
    /// `=`
    #[strum(to_string="=")]
    EqualSign,
    
    /// `?`
    #[strum(to_string="?")]
    QuestionMark,
    
    /// `!`
    #[strum(to_string="!")]
    ExclamationMark,
    
    /// `$`
    #[strum(to_string="$")]
    DollarSign,
    
    /// `%`
    #[strum(to_string="%")]
    Percentage,
    
    /// `&`
    #[strum(to_string="&")]
    Ampersand,
    
    /// `°`
    #[strum(to_string="°")]
    Circle,
    
    /// `|`
    #[strum(to_string="|")]
    Pipe,
    
    /// `^`
    #[strum(to_string="^")]
    Caret,
    
    /// `@`
    #[strum(to_string="@")]
    At,
    
    /// `..`
    #[strum(to_string="..")]
    Range,
    
    /// `<=`
    #[strum(to_string="<=")]
    EqLess,
    
    /// `>=`
    #[strum(to_string=">=")]
    EqGreater,
    
    /// `++`
    #[strum(to_string="++")]
    Incr,
    
    /// `--`
    #[strum(to_string="--")]
    Decr,
    
    /// `$$`
    #[strum(to_string="$$")]
    DoubleDollar,
}

impl Symbol {
    /// Is the symbol a operator?
    pub fn is_operator(&self) -> bool {
        matches!(self
            , Self::EqualSign
            | Self::EqGreater
            | Self::EqLess
            | Self::AngleLeft
            | Self::AngleRight
            | Self::Plus
            | Self::Dash
            | Self::Star
            | Self::Incr
            | Self::Decr
            | Self::Slash
            | Self::Tilde
            | Self::Caret
            | Self::QuestionMark
            | Self::ExclamationMark
        )
    }
    
    /// Is the symbol a delimiter?
    pub fn is_delimiter(&self) -> bool {
        matches!(self
            , Self::ParenLeft | Self::ParenRight
            | Self::BraketLeft | Self::BraketRight
            | Self::CurlyLeft | Self::CurlyRight
            | Self::AngleLeft | Self::AngleRight
        )
    }
    
    /// Is the symbol a delimiter?
    pub fn is_end_delimiter(&self) -> bool {
        matches!(self
            , Self::ParenRight
            | Self::BraketRight
            | Self::CurlyRight
            | Self::AngleRight
        )
    }
    
    /// Get the delimiter for the given symbol, or `None`.
    pub fn get_delimiter(&self) -> Option<Symbol> {
        match self {
            Self::ParenLeft => Some(Self::ParenRight),
            Self::BraketLeft => Some(Self::BraketRight),
            Self::CurlyLeft => Some(Self::CurlyRight),
            Self::AngleLeft => Some(Self::AngleRight),
            _ => None
        }
    }
}

impl From<&Symbol> for CompactString {
    fn from(symbol: &Symbol) -> Self {
        let mut cs = CompactString::new_const();
        write!(cs, "{}", symbol).unwrap(); // infallible
        cs
    }
}

impl std::fmt::Debug for Symbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, " {} ", self)
    }
}
