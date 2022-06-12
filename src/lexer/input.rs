//! Tokenizer input.

use std::str::CharIndices;

/// A stream of positioned characters.
pub type PosInput<'s> = peekmore::PeekMoreIterator<PosIter<'s>>;

/// The input, tied to some source str.
#[derive(Debug, Clone)]
pub struct PosIter<'s> {
    iter: std::str::CharIndices<'s>,
    lpos: usize,
    line: usize,
}

impl<'s> From<CharIndices<'s>> for PosIter<'s> {
    /// Creates a new [`PosIter`] from the given [`CharIndices`].
    fn from(iter: CharIndices<'s>) -> Self {
        Self { iter, lpos: 0, line: 0 }
    }
}

impl Iterator for PosIter<'_> {
    type Item = PosChar;

    fn next(&mut self) -> Option<Self::Item> {
        let (index, current) = self.iter.next()?;
        
        self.lpos += 1;
        
        if current == '\n' {
            self.line += 1;
            self.lpos = 0;
        }
        
        Some(PosChar {
            char: current,
            line: self.line,
            col: self.lpos,
            idx: index
        })
    }
}

/// A char with a position.
#[derive(Debug, Clone, Copy)]
pub struct PosChar {
    /// The current character.
    pub char: char,
    /// Line-number.
    pub line: usize,
    /// Position on current line.
    pub col: usize,
    /// Absolute byte position.
    pub idx: usize,
}

// ...for convenience.
impl std::ops::Deref for PosChar {
    type Target = char;
    fn deref(&self) -> &Self::Target {
        &self.char
    }
}
