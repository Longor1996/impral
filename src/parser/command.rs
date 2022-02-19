//! Parsing of an initial token-stream into a Abstract Syntax Tree.

use super::*;

/// Parses the stream of tokens into a command-expression.
pub fn parse_command(
    tokens: &mut PeekableTokenStream<impl TokenStream>,
    terminator: Option<Symbol>
) -> Result<Invoke, ParseError> {
    let name = match tokens.next() {
        Some(n) => n,
        None => return Err(ParseError::ExpectButEnd("a command name")),
    };
    
    let name: CompactString = name.try_into_command_name()?;
    
    // At this point, we have a name.
    parse_command_body(name, tokens, terminator)
}

/// Parses the stream of tokens into a command-expression.
pub fn parse_command_body(
    name: CompactString,
    tokens: &mut PeekableTokenStream<impl TokenStream>,
    terminator: Option<Symbol>
) -> Result<Invoke, ParseError> {
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
                            pos_args: smallvec![Expression::Reference(ReferenceRoot::Res)],
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
            Some(t) => {
                // ...starting with what may just be a expression...
                let expr = parse_expression(tokens, false)?;
                
                if let Some(Token {
                    content: TokenContent::Symbol(Symbol::EqualSign), ..
                }) = tokens.peek() {
                    // (l)expr into key
                    let lexpr = match expr {
                        Expression::Value(val) => match val {
                            Literal::Str(s) => s,
                            val => return Err(ParseError::ExpectButGot("a parameter name".into(), format!("{:?}", val).into())),
                        },
                        _ => return Err(ParseError::ExpectButGot("a parameter name".into(), "a symbol or command".into())),
                    };
                    
                    // consume the '='
                    drop(tokens.next());
                    
                    // parse value
                    let rexpr = parse_expression(tokens, false)?;
                    
                    cmd.nom_args.insert(lexpr, rexpr);
                    no_more_pos_args = true;
                } else {
                    if no_more_pos_args {
                        return Err(ParseError::PosArgAfterNomArg(t.start))
                    }
                    
                    // Don't care, push arg, go to next iter.
                    cmd.pos_args.push(expr);
                }
            },
        };
    }
    
    Ok(cmd)
}

/// A command to be evaluated.
#[derive(Clone, Default, PartialEq)]
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

impl std::fmt::Debug for Invoke {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.name)?;
        for arg in &self.pos_args {
            write!(f, " {:?}", arg)?;
        }
        for (key, arg) in &self.nom_args {
            write!(f, " {}={:?}", key, arg)?;
        }
        write!(f, "")
    }
}
