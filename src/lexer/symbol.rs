//! Symbol representation.

use std::convert::TryFrom;
use std::fmt::Write;
use smartstring::alias::CompactString;

/// A enum of the set of known symbols.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Symbol {
    /// `(`
    ParenLeft,
    /// `)`
    ParenRight,
    /// `[`
    BraketLeft,
    /// `]`
    BraketRight,
    /// `{`
    CurlyLeft,
    /// `}`
    CurlyRight,
    /// `<`
    AngleLeft,
    /// `>`
    AngleRight,
    /// `+`
    Plus,
    /// `-`
    Dash,
    /// `*`
    Star,
    /// `#`
    Hash,
    /// `/`
    Slash,
    /// `~`
    Tilde,
    /// `,`
    Comma,
    /// `.`
    Dot,
    /// `:`
    DoubleDot,
    /// `;`
    Semicolon,
    /// `_`
    Underscore,
    /// `=`
    EqualSign,
    /// `?`
    QuestionMark,
    /// `!`
    ExclamationMark,
    /// `$`
    DollarSign,
    /// `%`
    Percentage,
    /// `&`
    Ampersand,
    /// `|`
    Pipe,
    /// `^`
    Caret,
    /// `@`
    At,
    /// `..`
    Range,
    /// `<=`
    EqLess,
    /// `>=`
    EqGreater,
    /// `++`
    Incr,
    /// `--`
    Decr,
    /// `$$`
    DoubleDollar,
}

impl Symbol {
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
}

impl From<&Symbol> for CompactString {
    fn from(symbol: &Symbol) -> Self {
        let mut cs = CompactString::new_const();
        write!(cs, "{}", symbol).unwrap(); // infallible
        cs
    }
}

impl std::fmt::Display for Symbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            Symbol::ParenLeft => "(",
            Symbol::ParenRight => ")",
            Symbol::BraketLeft => "[",
            Symbol::BraketRight => "]",
            Symbol::CurlyLeft => "{",
            Symbol::CurlyRight => "}",
            Symbol::AngleLeft => "<",
            Symbol::AngleRight => ">",
            Symbol::Plus => "+",
            Symbol::Dash => "-",
            Symbol::Star => "*",
            Symbol::Hash => "#",
            Symbol::Slash => "/",
            Symbol::Tilde => "~",
            Symbol::Comma => ",",
            Symbol::Dot => ".",
            Symbol::DoubleDot => ":",
            Symbol::Semicolon => ";",
            Symbol::Underscore => "_",
            Symbol::EqualSign => "=",
            Symbol::QuestionMark => "?",
            Symbol::ExclamationMark => "!",
            Symbol::DollarSign => "$",
            Symbol::Percentage => "%",
            Symbol::Ampersand => "&",
            Symbol::Pipe => "|",
            Symbol::Caret => "^",
            Symbol::At => "@",
            Symbol::Range => "..",
            Symbol::EqLess => "<=",
            Symbol::EqGreater => ">=",
            Symbol::Incr => "++",
            Symbol::Decr => "++",
            Symbol::DoubleDollar => "$$",
        };
        
        f.write_str(str)
    }
}

impl std::fmt::Debug for Symbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, " {} ", self)
    }
}

impl TryFrom<char> for Symbol {
    type Error = ();

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            '(' => Ok(Symbol::ParenLeft),
            ')' => Ok(Symbol::ParenRight),
            '[' => Ok(Symbol::BraketLeft),
            ']' => Ok(Symbol::BraketRight),
            '{' => Ok(Symbol::CurlyLeft),
            '}' => Ok(Symbol::CurlyRight),
            '<' => Ok(Symbol::AngleLeft),
            '>' => Ok(Symbol::AngleRight),
            '+' => Ok(Symbol::Plus),
            '-' => Ok(Symbol::Dash),
            '*' => Ok(Symbol::Star),
            '/' => Ok(Symbol::Slash),
            '~' => Ok(Symbol::Tilde),
            '#' => Ok(Symbol::Hash),
            ',' => Ok(Symbol::Comma),
            '.' => Ok(Symbol::Dot),
            ':' => Ok(Symbol::DoubleDot),
            ';' => Ok(Symbol::Semicolon),
            '_' => Ok(Symbol::Underscore),
            '=' => Ok(Symbol::EqualSign),
            '?' => Ok(Symbol::QuestionMark),
            '!' => Ok(Symbol::ExclamationMark),
            '$' => Ok(Symbol::DollarSign),
            '%' => Ok(Symbol::Percentage),
            '&' => Ok(Symbol::Ampersand),
            '|' => Ok(Symbol::Pipe),
            '^' => Ok(Symbol::Caret),
            '@' => Ok(Symbol::At),
            _ => Err(())
        }
    }
}


impl TryFrom<(char, char)> for Symbol {
    type Error = ();

    fn try_from(value: (char, char)) -> Result<Self, Self::Error> {
        match value {
            ('.', '.') => Ok(Symbol::Range),
            ('<', '=') => Ok(Symbol::EqLess),
            ('>', '=') => Ok(Symbol::EqGreater),
            ('+', '+') => Ok(Symbol::Incr),
            ('-', '-') => Ok(Symbol::Decr),
            ('$', '$') => Ok(Symbol::DoubleDollar),
            _ => Err(())
        }
    }
}
