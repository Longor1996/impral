//! Parsing of tokens into Expressions and Structures.

use peekmore::PeekMore;

use super::*;

/// Parses a `TokenStream` into an AST.
pub fn parse_expression(
    tokens: &mut PeekableTokenStream<impl TokenStream>,
    first: bool
) -> Result<Expression, ParseError> {
    // Try to parse an expression item...
    let mut expr = parse_item(tokens, first)?;
    
    while let Some(token) = tokens.peek() {
        match token {
            // Dot? Subset access!
            Token {
                content: TokenContent::Symbol(Symbol::Dot), ..
            } => {
                drop(tokens.next()); // drop the first dot
                
                let mut get = Invoke {
                    name: "get".into(),
                    pos_args: smallvec![expr],
                    nom_args: Default::default(),
                };
                
                if let Some(Token {
                    content: TokenContent::Symbol(Symbol::QuestionMark), ..
                }) = tokens.peek() {
                    get.name = "get-unwrap".into();
                    drop(tokens.next()); // drop the question-mark
                }
                
                let inner = parse_expression(tokens, false)?;
                get.pos_args.push(inner);
                
                expr = Expression::Invoke(get.into());
            },
            
            // Range? Parse Range!
            Token {
                content: TokenContent::Symbol(Symbol::Range), ..
            } => {
                drop(tokens.next()); // drop the range
                
                let mut range = Invoke {
                    name: "range".into(),
                    pos_args: smallvec![expr],
                    nom_args: Default::default(),
                };
                
                let inner = parse_expression(tokens, false)?;
                range.pos_args.push(inner);
                
                expr = Expression::Invoke(range.into());
            },
            
            // QuestionMark? Existence check!
            Token {
                content: TokenContent::Symbol(Symbol::QuestionMark), ..
            } => {
                drop(tokens.next()); // drop the questionmark
                expr = Expression::Invoke(Invoke {
                    name: "exists".into(),
                    pos_args: smallvec![expr],
                    nom_args: Default::default(),
                }.into());
            },
            
            // Tilde? Relation!
            Token {
                content: TokenContent::Symbol(Symbol::Tilde), ..
            } => {
                drop(tokens.next()); // drop the tilde
                let to = parse_expression(tokens, false)?;
                expr = Expression::Invoke(Invoke {
                    name: "relative".into(),
                    pos_args: smallvec![expr, to],
                    nom_args: Default::default(),
                }.into());
            },
            
            // Circle? deg2rad!
            Token {
                content: TokenContent::Symbol(Symbol::Circle), ..
            } => {
                drop(tokens.next()); // drop the circle
                expr = Expression::Invoke(Invoke {
                    name: "deg2rad".into(),
                    pos_args: smallvec![expr],
                    nom_args: Default::default(),
                }.into());
            },
            
            // Pipe? Pipe!
            Token {
                content: TokenContent::Symbol(Symbol::Pipe), ..
            }  if first => {
                let mut pipe = Pipe {
                    source: expr,
                    stages: vec![],
                };
                
                while let Some(Token {
                    content: TokenContent::Symbol(Symbol::Pipe), ..
                }) = tokens.peek() {
                    drop(tokens.next());
                    
                    let filter = matches!(tokens.peek(), Some(Token {content: TokenContent::Symbol(Symbol::QuestionMark),..}));
                    if filter {drop(tokens.next())}
                    
                    let invoke = parse_command(tokens, None)?;
                    
                    let seg = PipeSeg {
                        filter,
                        invoke,
                    };
                    pipe.stages.push(seg);
                }
                
                expr = Expression::Pipe(pipe.into());
            },
            
            // Ignore everything else...
            _ => break
        }
    }
    
    Ok(expr)
}

/// Parses a `TokenStream` into an item (piece of an expression).
pub fn parse_item(
    tokens: &mut PeekableTokenStream<impl TokenStream>,
    first: bool
) -> Result<Expression, ParseError> {
    // Fetch the next token...
    let token = match tokens.next() {
        // Empty case? Error!
        None => return Err(ParseError::Empty),
        Some(t) => t
    };
    
    // Remainder? Error!
    if let Token {
        content: TokenContent::Remainder(r),
        start, end: _
    } = token {
        return Err(ParseError::Unrecognized(start, r))
    };
    
    // Is it a command?
    if first {
        if let Ok(command_name) = try_into_command_name(&token) {
            return parse_command_body(command_name, tokens, None).map(|i|i.into())
        }
    }
    
    // Literal? Pass thru directly!
    if let TokenContent::Literal(l) = token.content {
        return Ok(Expression::Value(l))
    }
    
    // A group? Parse a subset!
    if let TokenContent::Group(kind, subtokens) = token.content {
        return Ok(match kind {
            Symbol::ParenLeft => parse_expression(
                &mut subtokens.into_iter().peekmore(),
                true
            )?,
            
            Symbol::BraketLeft => parse_list(
                &mut subtokens.into_iter().peekmore()
            ).map(|l| Expression::Structure(Structure::List(l)))?,
            
            Symbol::CurlyLeft => parse_map(
                &mut subtokens.into_iter().peekmore()
            ).map(|d| Expression::Structure(Structure::Dict(d)))?,
            
            _ => unreachable!("encountered a token-group of unknown kind")
        })
    }
    
    // A global variable?
    if let TokenContent::Symbol(Symbol::At) = token.content {
        return Ok(match tokens.next() {
            Some(Token {
                content: TokenContent::Literal(Literal::Str(s)), ..
            }) => Expression::Reference(ReferenceRoot::Global(s)),
            
            Some(t) => return Err(ParseError::ExpectButGot("a global variable name".into(), format!("{}", t).into())),
            
            None => return Err(ParseError::ExpectButEnd("a global variable name")),
        })
    }
    
    // A local variable?
    if let TokenContent::Symbol(Symbol::DollarSign) = token.content {
        
        if let Some(peek) = tokens.peek().cloned() {
            // Named local variable.
            if let TokenContent::Literal(Literal::Str(s)) = peek.content {
                let _ = tokens.next();
                return Ok(Expression::Reference(ReferenceRoot::Local(s)))
            }
            
            // Numeric local variable.
            if let TokenContent::Literal(Literal::Int(i)) = peek.content {
                let _ = tokens.next();
                return Ok(Expression::Reference(ReferenceRoot::Local(i.to_string().into())))
            }
        }
        
        return Ok(Expression::Reference(ReferenceRoot::Res))
    }
    
    // A context variable?
    if let TokenContent::Symbol(Symbol::DoubleDollar) = token.content {
        return Ok(Expression::Reference(ReferenceRoot::Ctx))
    }
    
    return Err(ParseError::Unexpected(format!("token content: {:?}", token.content).into()))
}
