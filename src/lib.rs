#![doc = include_str!("../README.md")]
#![deny(missing_docs)]

//pub mod values;
pub mod lexer;
pub mod parser;

// The IMPRAL language guide.
#[cfg(debug_assertions)]
pub mod guide {
    #![doc = include_str!("./guide/_index.md")]
    pub use crate::lexer::*;
    pub use crate::parser::*;
    
    pub mod literals {
        #![doc = include_str!("./guide/literals.md")]
        pub use super::*;
    }
    
    pub mod expressions {
        #![doc = include_str!("./guide/expressions.md")]
        pub use super::*;
    }
    
    pub mod commands {
        #![doc = include_str!("./guide/commands.md")]
        pub use super::*;
    }
    
    pub mod pipes {
        #![doc = include_str!("./guide/pipes.md")]
        pub use super::*;
    }
    
    pub mod references {
        #![doc = include_str!("./guide/references.md")]
        pub use super::*;
    }
    
    pub mod members {
        #![doc = include_str!("./guide/members.md")]
        pub use super::*;
    }
    
}
