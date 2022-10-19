//! Parsing of tokens into Expressions and Structures.

use peekmore::PeekMore;

use super::*;

/// Parses a `TokenStream` into an AST.
pub fn parse_expression(
    tokens: &mut PeekableTokenStream<impl TokenStream>,
    start_cmd: bool,
    start_pipe: bool
) -> Result<Expression, ParseError> {
    
    if consume_symbol(tokens, Symbol::EqualSign) {
        // TODO: Infix expression parsing mode?
        return Err(ParseError::Unexpected("reserved symbol".into()))
    }
    
    // Try to parse an expression item...
    let mut expr = parse_item(tokens, start_cmd)?;
    
    // Postfix operator parsing...
    loop {
        // Braket? Index!
        if let Some(mut tokens) = consume_group(tokens, Symbol::BraketLeft) {
            let index = parse_expression(&mut tokens, true, true)?;
            
            if tokens.peek().is_some() {
                return Err(ParseError::ExpectButGot("end of expression".into(), "more tokens".into()))
            }
            
            expr = Expression::Index(Box::new(expr), Box::new(index));
            continue;
        }
        
        // Dot? Field!
        if consume_symbol(tokens, Symbol::Dot) {
            let member = if let Some(name) = consume_string(tokens) {
                name
            } else {
                return Err(ParseError::ExpectButGot("member name".into(), "something else".into()))
            };
            
            expr = Expression::Field(Box::new(expr), member);
            continue;
        }
        
        // Range? Parse Range!
        if consume_symbol(tokens, Symbol::Range) {
            if let Expression::Range(_, _, _) = expr {
                return Err(ParseError::ExpectButGot("a start that is not a range".into(), "a start that is a range".into()))
            }
            
            let inclusive = consume_symbol(tokens, Symbol::EqualSign);
            let end = parse_expression(tokens, false, false)?;
            
            if let Expression::Range(_, _, _) = end {
                return Err(ParseError::ExpectButGot("an end that is not a range".into(), "an end that is a range".into()))
            }
            
            expr = Expression::Range(Box::new(expr), Box::new(end), inclusive);
            continue;
        }
        
        // QuestionMark? Try unwrapping!
        if consume_symbol(tokens, Symbol::QuestionMark) {
            let throw = consume_symbol(tokens, Symbol::ExclamationMark);
            expr = Expression::Try(Box::new(expr), throw);
            continue;
        }
        
        // Tilde? Relation!
        if consume_symbol(tokens, Symbol::Tilde) {
            let to = parse_expression(tokens, false, false)?;
            expr = Expression::Invoke(Invoke {
                name: "relative".into(),
                pos_args: smallvec![expr, to],
                nom_args: Default::default(),
            }.into());
            continue;
        }
        
        // Circle? deg2rad!
        if consume_symbol(tokens, Symbol::Circle) {
            expr = Expression::Invoke(Invoke {
                name: "deg2rad".into(),
                pos_args: smallvec![expr],
                nom_args: Default::default(),
            }.into());
            continue;
        }
        
        // Pipe? Pipe!
        if start_pipe && consume_symbol(tokens, Symbol::Pipe) {
            expr = Expression::Pipe(parse_pipe(tokens, expr)?);
            continue;
        }
        
        break;
    }
    
    Ok(expr)
}

/// Parses a `TokenStream` into a pipe.
pub fn parse_pipe(
    tokens: &mut PeekableTokenStream<impl TokenStream>,
    expr: Expression
) -> Result<Box<Pipe>, ParseError> {
    let mut pipe = if let Expression::Pipe(pipe) = expr {
        pipe
    } else {
        Box::new(Pipe {
            source: expr,
            stages: Default::default(),
        })
    };
    
    loop {
        let segment = if consume_symbol(tokens, Symbol::QuestionMark) {
            PipeSeg::Exclude {
                predicate: parse_expression(tokens, true, false)?,
            }
        }
        else if consume_symbol(tokens, Symbol::ExclamationMark) {
            if match_symbol(tokens, Symbol::Pipe) || tokens.peek().is_none() {
                PipeSeg::Collect
            }
            else {
                PipeSeg::Folding {
                    initial: parse_expression(tokens, true, false)?,
                    reducer: parse_expression(tokens, true, false)?,
                }
            }
        }
        else {
            PipeSeg::Mapping {
                mapper: parse_expression(tokens, true, false)?,
            }
        };
        
        pipe.stages.push(segment);
        
        if consume_symbol(tokens, Symbol::Pipe) {
            continue;
        }
        
        break;
    }
    
    Ok(pipe)
}

/// Parses a `TokenStream` into an item (piece of an expression).
pub fn parse_item(
    tokens: &mut PeekableTokenStream<impl TokenStream>,
    start_cmd: bool
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
    if start_cmd {
        if let Ok(command_name) = try_into_command_name(&token) {
            return parse_command_body(command_name, tokens, None).map(|i|i.into())
        }
    }
    
    // Is it a underscore?
    
    // Underscore? Return an empty!
    if let TokenContent::Symbol(Symbol::Underscore) = token.content {
        return Ok(Expression::Empty)
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
                true,
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
    
    Err(ParseError::Unexpected(format!("token content: {:?}", token.content).into()))
}
