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
            // Dot? Dereference!
            Token {
                content: TokenContent::Symbol(Symbol::Dot), ..
            } => {
                drop(tokens.next()); // drop the dot
                expr = Expression::Deref(parse_deref(tokens, expr)?);
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
                expr = Expression::Pipe(parse_pipe(tokens, expr)?.into());
            },
            
            // Ignore everything else...
            _ => break
        }
    }
    
    Ok(expr)
}


/// Parses a `TokenStream` into a deref chain.
pub fn parse_deref(
    tokens: &mut PeekableTokenStream<impl TokenStream>,
    expr: Expression
) -> Result<Box<DerefChain>, ParseError> {
    let mut chain = if
    let Expression::Deref(deref) = expr {
        deref
    } else {
        Box::new(DerefChain {
            source: expr,
            stages: Default::default(),
        })
    };
    
    let fallible = if let Some(Token {
        content: TokenContent::Symbol(Symbol::QuestionMark), ..
    }) = tokens.peek() {
        drop(tokens.next()); // drop the question-mark
        true
    } else {
        false
    };
    
    let member = match tokens.next() {
        Some(Token {
            content: TokenContent::Literal(Literal::Str(name)), ..
        }) => name,
        Some(Token {
            content, ..
        }) => return Err(ParseError::ExpectButGot("member name".into(), format!("{content:?}").into())),
        None => return Err(ParseError::ExpectButEnd("member name")),
    };
    
    chain.stages.push(DerefSeg {
        fallible,
        member,
    });
    
    Ok(chain)
}
/// Parses a `TokenStream` into a pipe.
pub fn parse_pipe(
    tokens: &mut PeekableTokenStream<impl TokenStream>,
    source: Expression
) -> Result<Pipe, ParseError> {
    let mut pipe = Pipe {
        source,
        stages: vec![],
    };
    
    while let Some(Token {
        content: TokenContent::Symbol(Symbol::Pipe), ..
    }) = tokens.peek() {
        drop(tokens.next());
        
        let filter = matches!(tokens.peek(), Some(Token {content: TokenContent::Symbol(Symbol::QuestionMark),..}));
        if filter {drop(tokens.next())}
        
        let invoke = parse_expression(tokens, true)?;
        
        let seg = PipeSeg {
            filter,
            invoke,
        };
        pipe.stages.push(seg);
    }
    
    Ok(pipe)
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
            ).map(|l| Expression::Invoke(Box::new(Invoke {
                name: "list".into(),
                pos_args: l,
                ..Default::default()
            })))?,
            
            Symbol::CurlyLeft => parse_map(
                &mut subtokens.into_iter().peekmore()
            ).map(|d| Expression::Invoke(Box::new(Invoke {
                name: "dict".into(),
                nom_args: d,
                ..Default::default()
            })))?,
            
            _ => unreachable!("encountered a token-group of unknown kind")
        })
    }
    
    return Err(ParseError::Unexpected(format!("token content: {:?}", token.content).into()))
}
