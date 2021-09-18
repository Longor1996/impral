//! Parses a stream of tokens into an AST.

use std::{borrow::Cow, fmt::Debug, marker::PhantomData};

use tagged_box::TaggableContainer;
//use smallvec::SmallVec;
use thiserror::Error;
use rustc_hash::FxHashMap;
use smartstring::alias::CompactString;

use crate::{lexer::{Literal, PeekableTokenStream, Symbol, Token, TokenContent, TokenStream}, values::*};

/// Parses a `TokenStream` into an AST.
pub fn parse_expression(tokens: &mut PeekableTokenStream<impl TokenStream>) -> Result<Expression, ParseError> {
    let mut expr = match tokens.next() {
        // Empty case? Error!
        None => return Err(ParseError::Empty),
        
        // Remainder? Error!
        Some(Token {
            content: TokenContent::Remainder(r),
            position
        }) => return Err(ParseError::Unrecognized(position, r)),
        
        // Literal? Pass thru directly!
        Some(Token {
            content: TokenContent::Literal(l), ..
        }) => Expression::Value(l.into()),
        
        // Parentheses? Parse a command!
        Some(Token {
            content: TokenContent::Symbol(Symbol::ParenLeft), ..
        }) => parse_command(tokens, Some(Symbol::ParenRight)).map(|i| i.into())?,
        
        // Brackets? Parse a list!
        Some(Token {
            content: TokenContent::Symbol(Symbol::BraketLeft), ..
        }) => parse_list(tokens).map(|v| Expression::Value(ValContainer::from(v)))?,
        
        // Curlies? Parse a map!
        Some(Token {
            content: TokenContent::Symbol(Symbol::CurlyLeft), ..
        }) => parse_map(tokens).map(|v| Expression::Value(ValContainer::from(v)))?,
        
        // Global Variable!
        Some(Token {
            content: TokenContent::Symbol(Symbol::At), ..
        }) => match tokens.next() {
            Some(Token {
                content: TokenContent::Literal(Literal::Str(s)), ..
            }) => Expression::Value(ValContainer::from(GlobalVar(s))),
            Some(t) => return Err(ParseError::ExpectButGot("a global variable name".into(), format!("{}", t).into())),
            None => return Err(ParseError::ExpectButEnd("a global variable name")),
        },
        
        // Local Variable!
        Some(Token {
            content: TokenContent::Symbol(Symbol::DollarSign), ..
        }) => match tokens.next() {
            Some(Token {
                content: TokenContent::Symbol(Symbol::DollarSign), ..
            }) => Expression::Value(ValItem::ResultVar(PhantomData::default()).into()),
            Some(Token {
                content: TokenContent::Literal(Literal::Str(s)), ..
            }) => Expression::Value(ValContainer::from(LocalVar(s))),
            Some(Token {
                content: TokenContent::Literal(Literal::Int(i)), ..
            }) => Expression::Value(ValContainer::from(LocalVar(i.to_string().into()))),
            Some(t) => return Err(ParseError::ExpectButGot("a local variable name".into(), format!("{}", t).into())),
            None => return Err(ParseError::ExpectButEnd("a local variable name")),
        },
        
        // Doubledot? Invalid!
        Some(Token {
            content: TokenContent::Symbol(Symbol::DoubleDot), ..
        }) => return Err(ParseError::Unexpected("double-dot in expression".into())),
        
        // Symbols means more complex parsing...
        Some(Token {
            content: TokenContent::Symbol(s), ..
        }) => {
            return Err(ParseError::Unexpected(format!("symbol in expression: {}", s).into()))
        }
    };
    
    if let Some(token) = tokens.peek() {
        match token {
            // Dot? Subset access!
            Token {
                content: TokenContent::Symbol(Symbol::Dot), ..
            } => {
                drop(tokens.next()); // drop the dot
                let mut get = Invoke {
                    name: "idx".into(),
                    pos_args: vec![expr],
                    nom_args: Default::default(),
                };
                
                if let Some(Token {
                    content: TokenContent::Symbol(Symbol::QuestionMark), ..
                }) = tokens.peek() {
                    get.name = "idxn".into();
                    drop(tokens.next()); // drop the question-mark
                }
                
                let inner = parse_expression(tokens)?;
                get.pos_args.push(inner);
                
                expr = Expression::Invoke(get.into());
            },
            
            // QuestionMark? Existence check!
            Token {
                content: TokenContent::Symbol(Symbol::QuestionMark), ..
            } => {
                drop(tokens.next()); // drop the dot
                expr = Expression::Invoke(Invoke {
                    name: "exists".into(),
                    pos_args: vec![expr],
                    nom_args: Default::default(),
                }.into());
            },
            
            // Tilde? Relation!
            Token {
                content: TokenContent::Symbol(Symbol::Tilde), ..
            } => {
                drop(tokens.next()); // drop the dot
                let to = parse_expression(tokens)?;
                expr = Expression::Invoke(Invoke {
                    name: "rel".into(),
                    pos_args: vec![expr, to],
                    nom_args: Default::default(),
                }.into());
            },
            
            // Ignore everything else...
            _ => ()
        }
    }
    
    // postfix stuff!
    // TODO: Ranges
    // TODO: Units
    // etc. etc.
    
    Ok(expr)
}

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
                            pos_args: vec![Expression::Value(ValContainer::from(PhantomData::<Result<(),()>>::default()))],
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

/// Parses the stream of tokens into a list.
pub fn parse_list(
    tokens: &mut PeekableTokenStream<impl TokenStream>
) -> Result<Vec<ValContainer>, ParseError> {
    let mut list = Vec::default();
    
    loop {
        match tokens.peek().cloned() {
            Some(t) => {
                if let TokenContent::Symbol(s) = t.content {
                    if s == Symbol::BraketRight {
                        drop(tokens.next());
                        break;
                    }
                    
                    if s == Symbol::Comma {
                        drop(tokens.next());
                        continue;
                    }
                }
                
                let expr = parse_expression(tokens)?;
                list.push(expr.into());
            },
            None => return Err(ParseError::ExpectButEnd("end of list ']'")),
        }
    }
    
    Ok(list)
}

/// Parses the stream of tokens into a key/value-map.
pub fn parse_map(
    tokens: &mut PeekableTokenStream<impl TokenStream>
) -> Result<FxHashMap<CompactString, ValContainer>, ParseError> {
    let mut map = FxHashMap::default();
    
    loop {
        match tokens.peek().cloned() {
            Some(t) => {
                match t.content {
                    TokenContent::Remainder(r )
                        => return Err(ParseError::Unrecognized(t.position, r)),
                    
                    TokenContent::Symbol(s) => {
                        if s == Symbol::CurlyRight {
                            drop(tokens.next());
                            break;
                        }
                        
                        if s == Symbol::Comma {
                            drop(tokens.next());
                            continue;
                        }
                        
                        return Err(ParseError::Unexpected(format!("symbol '{}'", s).into()));
                    },
                    TokenContent::Literal(Literal::Str(s)) => {
                        let key = s;
                        drop(tokens.next()); // eat key
                        
                        let next = tokens.next();
                        if let Some(Token {content: TokenContent::Symbol(Symbol::EqualSign), ..}) = next {
                            // everything checks out, continue on...
                        } else {
                            return Err(ParseError::Unexpected("token".into()));
                        }
                        
                        let expr = parse_expression(tokens)?;
                        map.insert(key, expr.into());
                    },
                    TokenContent::Literal(l) => return Err(ParseError::Unexpected(format!("literal {:?}", l).into()))
                };
            },
            None => return Err(ParseError::ExpectButEnd("end of list ']'")),
        }
    }
    
    Ok(map)
}

/// A expression node of an abstract syntax tree.
#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    /// A literal.
    Value(ValContainer),
    
    /// A command.
    Invoke(Box<Invoke>),
}

impl From<Invoke> for Expression {
    fn from(i: Invoke) -> Self {
        Self::Invoke(i.into())
    }
}

impl std::fmt::Display for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expression::Value(l) => std::fmt::Display::fmt(l, f),
            Expression::Invoke(c) => write!(f, "({})", c),
        }
    }
}

/// A command to be evaluated.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Invoke {
    /// The name of the command.
    pub name: CompactString,
    
    /// The positional arguments.
    pub pos_args: Vec<Expression>,
    
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

/// A parsing error.
#[derive(Error, Debug)]
pub enum ParseError {
    /// The stream of tokens is empty.
    #[error("The stream of tokens is empty")]
    Empty,
    
    /// There was a character that could not be tokenized/lexed.
    #[error("Unrecognized token at {0}: {1}")]
    Unrecognized(usize, String),
    
    /// The stream of tokens ended unexpectedly.
    #[error("Expected {0}, but reached end of stream")]
    ExpectButEnd(&'static str),
    
    /// An unexpected thing appeared.
    #[error("Unexpected {0}")]
    Unexpected(Cow<'static, str>),
    
    /// Expected one thing, but got another.
    #[error("Expected {0}, but {1}")]
    ExpectButGot(Cow<'static, str>, Cow<'static, str>),
    
    /// Positional args cannot be written after nominal args.
    #[error("Positional args cannot be written after nominal args")]
    PosArgAfterNomArg,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::tokenize;
    
    #[test]
    fn sizes() {
        use std::mem::size_of;
        eprintln!("SizeOf LEX.L = {}", size_of::<Literal>());
        eprintln!("SizeOf LEX.S = {}", size_of::<Symbol>());
        eprintln!("SizeOf AST.V = {}", size_of::<ValContainer>());
        eprintln!("SizeOf AST.E = {}", size_of::<Expression>());
        eprintln!("SizeOf AST.C = {}", size_of::<Invoke>());
    }
    
    #[test]
    fn should_succeed() -> Result<(), ParseError> {
        fn chk(input: &str) -> Result<(), ParseError> {
            let output = parse_command(&mut tokenize(input), None)?;
            eprintln!("INPUT:  {},\t PARSED:  {}", input, output);
            Ok(())
        }
        
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
        eprintln!("-> {}", parse_command(&mut tokenize("test 1 a=2 3 b=4"), None).unwrap());
    }
}
