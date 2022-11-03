//! Symbol representation.

use std::fmt::Write;
use smartstring::alias::CompactString;
use strum_macros::{Display, EnumString, EnumIter, EnumDiscriminants, IntoStaticStr};

/// A enum of the set of known symbols.
/// 
/// **Note:** Symbols can only be a single character or a pair of characters.
#[derive(Clone, Copy, PartialEq, Eq, Display, EnumString, EnumIter, EnumDiscriminants, IntoStaticStr)]
#[strum_discriminants(name(SymbolName))]
#[strum_discriminants(derive(Display))]
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
    
    /// `&`
    #[strum(to_string="&")]
    Ampersand,
    
    /// `%`
    #[strum(to_string="%")]
    Percentage,
    
    /// `°`
    #[strum(to_string="°")]
    Circle,
    
    /// `²`
    #[strum(to_string="²")]
    Pow2,
    
    /// `³`
    #[strum(to_string="³")]
    Pow3,
    
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
    
    /// `==`
    #[strum(to_string="==")]
    EqEq,
    
    /// `!=`
    #[strum(to_string="!=")]
    NotEq,
    
    /// `<=`
    #[strum(to_string="<=")]
    EqLess,
    
    /// `>=`
    #[strum(to_string=">=")]
    EqGreater,
    
    /// `<>`
    #[strum(to_string="<>")]
    Compare,
    
    /// `++`
    #[strum(to_string="++")]
    Incr,
    
    /// `--`
    #[strum(to_string="--")]
    Decr,
    
    /// `$$`
    #[strum(to_string="$$")]
    DoubleDollar,
    
    /// `&&`
    #[strum(to_string="&&")]
    DoubleAmpersand,
    
    /// `||`
    #[strum(to_string="||")]
    DoublePipe,
    
    /// `**`
    #[strum(to_string="**")]
    DoubleStar,
    
    /// `->`
    #[strum(to_string="->")]
    ThinArrow,
    
    /// `~>`
    #[strum(to_string="~>")]
    WaveArrow,
    
    /// `+>`
    #[strum(to_string="+>")]
    PlusArrow,
    
    /// `=>`
    #[strum(to_string="=>")]
    BindArrow,
    
    /// `#>`
    #[strum(to_string="#>")]
    GridArrow,
}

impl Symbol {
    /// Is the symbol a operator?
    /// 
    /// Operators can be used in place of function names.
    pub fn is_operator(&self) -> bool {
        matches!(self
            , Self::EqEq
            | Self::NotEq
            | Self::EqGreater
            | Self::EqLess
            | Self::Compare
            | Self::AngleLeft
            | Self::AngleRight
            | Self::Plus
            | Self::Dash
            | Self::Star
            | Self::Incr
            | Self::Decr
            | Self::DoubleStar
            | Self::Slash
            | Self::Tilde
            | Self::Caret
            | Self::QuestionMark
            | Self::ExclamationMark
        )
    }
    
    /// Is the symbol a postfix operator?
    pub fn is_infix_operator(&self) -> bool {
        matches! {self
            , Self::Plus
            | Self::Dash
            | Self::Star
            | Self::Slash
            | Self::Percentage
            | Self::DoubleStar
        }
    }
    
    /// Is the symbol a postfix operator?
    pub fn is_postop(&self) -> Option<&'static str> {
        match self {
            Self::Percentage => Some("into_percent"),
            Self::Circle => Some("into_radians"),
            Self::Pow2 => Some("into_squared"),
            Self::Pow3 => Some("into_cubed"),
            _ => None
        }
    }
    
    /// Is the symbol a arrow operator?
    pub fn is_arrow(&self) -> bool {
        matches!{self
            , Self::ThinArrow
            | Self::WaveArrow
            | Self::PlusArrow
            | Self::BindArrow
            | Self::GridArrow
        }
    }
    
    /// Is the symbol a delimiter?
    pub fn is_delimiter(&self) -> bool {
        self.is_start_delimiter() || self.is_end_delimiter()
    }
    
    /// Is the symbol a delimiter start?
    pub fn is_start_delimiter(&self) -> bool {
        matches!(self
            , Self::ParenLeft
            | Self::BraketLeft
            | Self::CurlyLeft
            | Self::AngleLeft
        )
    }
    
    /// Is the symbol a delimiter end?
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
    
    /// Returns the precendence for this symbol, or 0.
    pub fn get_precedence(&self) -> u8 {
        match self {
            // ASSIGN => 1,
            // CONDITION => 2,
            
            // SUM
            Self::Plus | Self::Dash => 3,
            
            // PRODUCT
            Self::Star | Self::Slash | Self::Percentage => 4,
            
            // EXPONENT
            Self::DoubleStar => 5,
            
            // PREFIX => 6,
            // POSTFIX => 7,
            Self::Dot | Self::Range | Self::QuestionMark | Self::Tilde | Self::ThinArrow => 7,
            _ if self.is_postop().is_some() => 7,
            
            // CALL & GROUP => 8,
            
            _ => 0
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
