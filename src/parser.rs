//! Parses a stream of tokens into an AST.

use std::borrow::Cow;

use tagged_box::TaggableContainer;
//use smallvec::SmallVec;
use thiserror::Error;
use rustc_hash::FxHashMap;
use smartstring::alias::CompactString;

use crate::{
    lexer::{Literal, PeekableTokenStream, Symbol, Token, TokenContent, TokenStream},
    values::*
};

/// Parses a `TokenStream` into an AST.
pub fn parse_expression(tokens: &mut PeekableTokenStream<impl TokenStream>) -> Result<Expression, ParseError> {
    let expr = match tokens.next() {
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
    
    // postfix stuff!
    // TODO: Ranges
    // TODO: Units
    // TODO: RelativeTo
    // TODO: Exists
    // TODO: Dot-Indexing
    // TODO: Box-Indexing
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
                content: TokenContent::Symbol(Symbol::DoubleDot), ..
            }) => {
                drop(tokens.next());
                let subcommand = parse_command(tokens, None)?;
                cmd.pos_args.push(subcommand.into());
                break; // natural end of command, due to subcommand
            },
            
            Some(Token {
                content: TokenContent::Symbol(s), ..
            })
                if s.is_end_delimiter()
                || terminator.map_or(false, |t| t == s)
            => {
                drop(tokens.next());
                break // natural end of command, due to delimiter or terminator.
            },
            
            None => if terminator.is_some() {
                // abnormal end of command, due to missing terminator.
                return Err(ParseError::ExpectButEnd("a delimiter"))
            } else {
                break; // natural end of command, due to EOS.
            },
            
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
                                _ => return Err(ParseError::ExpectButGot("a parameter name".into(), "something else".into())),
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
            Expression::Value(l) => l.fmt(f),
            Expression::Invoke(c) => write!(f, "({})", c),
        }
    }
}

/// A command to be evaluated.
#[derive(Debug, Clone, PartialEq)]
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
    #[error("Unrecognized character at {0}: {1}")]
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
        eprintln!("-> {}", parse_command(&mut tokenize("= 1 2 3"), None)?);
        eprintln!("-> {}", parse_command(&mut tokenize("+ 1 2 3"), None)?);
        eprintln!("-> {}", parse_command(&mut tokenize("- 1 2 3"), None)?);
        eprintln!("-> {}", parse_command(&mut tokenize("* 1 2 3"), None)?);
        eprintln!("-> {}", parse_command(&mut tokenize("/ 1 2 3"), None)?);
        eprintln!("-> {}", parse_command(&mut tokenize("test 1 2 3"), None)?);
        eprintln!("-> {}", parse_command(&mut tokenize("test 1 2 3 a=4"), None)?);
        eprintln!("-> {}", parse_command(&mut tokenize("mul 2 (+ 1 2 3)"), None)?);
        eprintln!("-> {}", parse_command(&mut tokenize("test foo: bar baz"), None)?);
        eprintln!("-> {}", parse_command(&mut tokenize("test [1 2 3 4 5]"), None)?);
        eprintln!("-> {}", parse_command(&mut tokenize("test {a = 1, b=2, c=-3}"), None)?);
        Ok(())
    }
    
    #[test]
    #[should_panic]
    fn should_fail() {
        eprintln!("-> {}", parse_command(&mut tokenize("test 1 a=2 3 b=4"), None).unwrap());
    }
}
