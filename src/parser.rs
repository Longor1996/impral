//! Parses a stream of tokens into an AST.

use std::{borrow::Cow, fmt::Debug, marker::PhantomData};

use smallvec::{SmallVec, smallvec};
use tagged_box::TaggableContainer;
//use smallvec::SmallVec;
use thiserror::Error;
use rustc_hash::FxHashMap;
use smartstring::alias::CompactString;

use crate::{lexer::{Literal, PeekableTokenStream, Symbol, Token, TokenContent, TokenStream}, values::*};

pub mod expression;
pub use expression::*;

/// Parses the stream of tokens into a command-expression.
pub fn parse_command(
    tokens: &mut PeekableTokenStream<impl TokenStream>,
    terminator: Option<Symbol>
) -> Result<Invoke, ParseError> {
    
    let name = match tokens.next() {
        Some(n) => n,
        None => return Err(ParseError::ExpectButEnd("a command name")),
    };
    
    let name: CompactString = match name.content {
        TokenContent::Remainder(r )
            => return Err(ParseError::Unrecognized(name.position, r)),
        
        // Every kind of symbol BUT delimiters can be a command name...
        TokenContent::Symbol(s ) if s.is_delimiter()
            => return Err(ParseError::ExpectButGot("a command name".into(), format!("a '{}'", s).into())),
        TokenContent::Symbol(s) => (&s).into(),
        
        // Every kind of literal BUT strings cannot be a command name...
        TokenContent::Literal(Literal::Str(s)) => s,
        TokenContent::Literal(l)
            => return Err(ParseError::ExpectButGot("a command name".into(), format!("a {}", l.get_type_str()).into())),
    };
    
    // At this point, we have a name.
    
    let mut cmd = Invoke {
        name,
        pos_args: Default::default(),
        nom_args: Default::default(),
    };
    
    let mut no_more_pos_args = false;
    
    loop {
        match tokens.peek().cloned() {
            Some(Token {
                content: TokenContent::Symbol(s), ..
            })
                if terminator.map_or(false, |t| t == s)
            => {
                // We do NOT drop the terminator here...
                break // natural end of command, due to terminator.
            },
            
            Some(Token {
                content: TokenContent::Symbol(Symbol::DoubleDot), ..
            }) => {
                drop(tokens.next());
                let subcommand = parse_command(tokens, None)?;
                cmd.pos_args.push(subcommand.into());
                break; // natural end of command, due to subcommand
            },
            
            Some(Token {
                content: TokenContent::Symbol(Symbol::Ampersand), ..
            }) => {
                drop(tokens.next());
                
                let previous = std::mem::replace(&mut cmd, Invoke {
                    name: "if-then".into(),
                    pos_args: Default::default(),
                    nom_args: Default::default(),
                });
                
                cmd.pos_args.push(previous.into());
                
                let subcommand = parse_command(tokens, None)?;
                cmd.pos_args.push(subcommand.into());
                break; // natural end of command, due to subcommand
            },
            
            Some(Token {
                content: TokenContent::Symbol(s), ..
            })
                if s.is_end_delimiter()
            => {
                drop(tokens.next());
                break // natural end of command, due to delimiter.
            },
            
            Some(Token {
                content: TokenContent::Symbol(Symbol::Pipe), ..
            }) => {
                let previous = std::mem::replace(&mut cmd, Invoke {
                    name: "pipe".into(),
                    pos_args: Default::default(),
                    nom_args: Default::default(),
                });
                
                cmd.pos_args.push(previous.into());
                
                // parse further commands as long as there are pipe symbols
                while let Some(Token { content: TokenContent::Symbol(Symbol::Pipe), .. }) = tokens.peek() {
                    drop(tokens.next()); // drop the pipe symbol
                    
                    if tokens.peek().is_none() {
                        return Err(ParseError::ExpectButEnd("another command in the pipe"));
                    }
                    
                    if let Some(Token { content: TokenContent::Symbol(Symbol::QuestionMark), .. }) = tokens.peek() {
                        drop(tokens.next()); // drop the question-mark
                    } else {
                        cmd.pos_args.push(Invoke {
                            name: "nonull".into(),
                            pos_args: smallvec![Expression::Value(ValContainer::from(PhantomData::<Result<(),()>>::default()))],
                            ..Default::default()
                        }.into());
                    }
                    
                    let subcommand = parse_command(tokens, Some(Symbol::Pipe))?;
                    cmd.pos_args.push(subcommand.into());
                }
                
                break; // natural end of command, no more pipes.
            },
            
            None => break, // natural end of command, due to EOS.
            
            // Attempt parsing arguments...
            Some(_) => {
                // ...starting with what may just be a expression...
                let expr = parse_expression(tokens)?;
                
                match tokens.peek().cloned() {
                    Some(Token {
                        content: TokenContent::Symbol(Symbol::EqualSign), ..
                    }) => {
                        // (l)expr into key
                        let lexpr = match expr {
                            Expression::Value(val) => match val.into_inner() {
                                ValItem::String(s) => s,
                                val => return Err(ParseError::ExpectButGot("a parameter name".into(), format!("{}", val).into())),
                            },
                            _ => return Err(ParseError::ExpectButGot("a parameter name".into(), "a symbol or command".into())),
                        };
                        
                        // consume the '='
                        drop(tokens.next());
                        
                        // parse value
                        let rexpr = parse_expression(tokens)?;
                        
                        cmd.nom_args.insert(lexpr, rexpr);
                        no_more_pos_args = true;
                    },
                    _ => {
                        if no_more_pos_args {
                            return Err(ParseError::PosArgAfterNomArg)
                        }
                        
                        // Don't care, push arg, go to next iter.
                        cmd.pos_args.push(expr);
                    },
                }
            },
        };
    }
    
    Ok(cmd)
}

/// A command to be evaluated.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Invoke {
    /// The name of the command.
    pub name: CompactString,
    
    /// The positional arguments.
    ///
    /// As long as there is only one positional argument, there will be no direct heap allocation.
    pub pos_args: SmallVec<[Expression; 1]>,
    
    /// The nominal/named arguments.
    pub nom_args: FxHashMap<CompactString, Expression>,
}

impl std::fmt::Display for Invoke {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.name)?;
        for arg in &self.pos_args {
            write!(f, " {}", arg)?;
        }
        for (key, arg) in &self.nom_args {
            write!(f, " {}={}", key, arg)?;
        }
        write!(f, "")
    }
}
pub mod error;
pub use error::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::tokenize;
    
    fn chk(input: &str) -> Result<(), ParseError> {
        let output = parse_command(&mut tokenize(input), None)?;
        eprintln!("INPUT:  {},\t PARSED:  {}", input, output);
        Ok(())
    }
    
    #[test]
    fn sizes() {
        use std::mem::size_of;
        eprintln!("SizeOf LEX.L = {}", size_of::<Literal>());
        eprintln!("SizeOf LEX.S = {}", size_of::<Symbol>());
        eprintln!("SizeOf AST.V = {}", size_of::<ValContainer>());
        eprintln!("SizeOf AST.E = {}", size_of::<Expression>());
        eprintln!("SizeOf AST.C = {}", size_of::<Invoke>());
        
        assert!(size_of::<ValContainer>() == 8, "The size of a ValContainer-struct should be exactly 8 bytes.");
        assert!(size_of::<Expression>() == 16, "The size of a Expression-struct should be exactly 16 bytes.");
        assert!(size_of::<Invoke>() <= 96, "The size of an Invoke-struct should be below 96 bytes.");
    }
    
    #[test]
    fn should_succeed() -> Result<(), ParseError> {
        chk("= 1 2 3")?;
        chk("+ 1 2 3")?;
        chk("- 1 2 3")?;
        chk("* 1 2 3")?;
        chk("/ 1 2 3")?;
        chk("test 1 2 3")?;
        chk("test 1 2 3 a=4")?;
        chk("mul 2 (+ 1 2 3)")?;
        chk("test foo: bar baz")?;
        chk("test [1 2 3 4 5]")?;
        chk("test {a = 1, b=2, c=-3}")?;
        chk("testA 1 2 3 | testB 4 5 6 | testC 7 8 9")?;
        chk("maybe-null |? accepts-null")?;
        chk("conditional & execution")?;
        chk("echo \"Hello, World!\" @s.chat ")?;
        chk("tp @a 0 0 0")?;
        chk("tp @a @world.spawn")?;
        chk("tp @a 0 100 0 rel=@self")?;
        chk("for @a: tp [0 100 0]~$$")?;
        Ok(())
    }
    
    #[test]
    #[should_panic]
    fn should_fail() {
        chk("test 1 a=2 3 b=4").expect("positional arguments cannot be written after nominal arguments");
    }
}
