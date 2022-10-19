//! Parsing of an initial token-stream into a Abstract Syntax Tree.

use super::*;

/// Try to convert the given TokenContent into a command-name...
pub fn try_into_command_name(token: &Token) -> Result<smartstring::alias::CompactString, ParseError> {
    match token.content.clone() {
        TokenContent::Remainder(r )
            => Err(ParseError::Unrecognized(token.start, r)),
        
        // Every kind of symbol BUT delimiters can be a command name...
        TokenContent::Symbol(s ) if !s.is_operator()
            => Err(ParseError::ExpectButGot("a command name".into(), format!("a '{}'", s).into())),
        TokenContent::Symbol(s) => Ok((&s).into()),
        
        // Every kind of literal BUT strings cannot be a command name...
        TokenContent::Literal(Literal::Str(s)) => Ok(s),
        TokenContent::Literal(l)
            => Err(ParseError::ExpectButGot("a command name".into(), format!("a {}", l.get_type_str()).into())),
        
        TokenContent::Group(_, _)
            => Err(ParseError::ExpectButGot("a command name".into(), "a group".to_string().into())),
    }
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
    
    let name: CompactString = try_into_command_name(&name)?;
    
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
        if let Some(terminator) = terminator {
            // We MATCH, but NOT drop, the terminator...
            if match_symbol(tokens, terminator) {
                break; // natural end of command, due to terminator
            }
        }
        
        if match_symbol(tokens, Symbol::Semicolon) {
            // We MATCH, but NOT drop, the semicolon...
            break; // natural end of command, due to semicolon
        }
        
        if consume_symbol(tokens, Symbol::DoubleDot) {
            let subcommand = parse_command(tokens, None)?;
            cmd.pos_args.push(subcommand.into());
            break; // natural end of command, due to subcommand
        }
        
        if consume_symbol(tokens, Symbol::DoubleAmpersand) {
            let previous = std::mem::replace(&mut cmd, Invoke {
                name: "if-then".into(),
                pos_args: Default::default(),
                nom_args: Default::default(),
            });
            
            cmd.pos_args.push(previous.into());
            
            let subcommand = parse_command(tokens, None)?;
            cmd.pos_args.push(subcommand.into());
            break; // natural end of command, due to IF-THEN wrapper command
        }
        
        if consume_symbol(tokens, Symbol::DoublePipe) {
            let previous = std::mem::replace(&mut cmd, Invoke {
                name: "if-else".into(),
                pos_args: Default::default(),
                nom_args: Default::default(),
            });
            
            cmd.pos_args.push(previous.into());
            
            let subcommand = parse_command(tokens, None)?;
            cmd.pos_args.push(subcommand.into());
            break; // natural end of command, due to IF-ELSE wrapper command
        }
        
        if consume_if(tokens, |tc|
            matches!(tc, TokenContent::Symbol(peeked) if peeked.is_end_delimiter())
        ).is_some() {
            break // natural end of command, due to delimiter.
        }
        
        if match_symbol(tokens, Symbol::Pipe) {
            break; // Encountered a pipe; command must end here.
        }
        
        if consume_symbol(tokens, Symbol::Dash) {
            if let Some(s) = consume_string(tokens) {
                cmd.nom_args.insert(s, Expression::Value(Literal::Bool(false)));
                no_more_pos_args = true;
                continue;
            } else {
                return Err(ParseError::ExpectButGot("a flag".into(), "something else".into()))
            }
        }
        
        if consume_symbol(tokens, Symbol::Plus) {
            if let Some(s) = consume_string(tokens) {
                cmd.nom_args.insert(s, Expression::Value(Literal::Bool(true)));
                no_more_pos_args = true;
                continue;
            } else {
                return Err(ParseError::ExpectButGot("a flag".into(), "something else".into()))
            }
        }
        
        if let Some(token) = tokens.peek().cloned() {
            // Attempt parsing arguments...
            // BAREWORD=EXPRESSION
            // EXPRESSION
            
            // ...starting with what may just be a expression...
            let expr = parse_expression(tokens, false, false)?;
            
            if consume_symbol(tokens, Symbol::EqualSign) {
                // (l)expr into key
                let lexpr = match expr {
                    Expression::Value(val) => match val {
                        Literal::Str(s) => s,
                        val => return Err(ParseError::ExpectButGot("a parameter name".into(), format!("{:?}", val).into())),
                    },
                    _ => return Err(ParseError::ExpectButGot("a parameter name".into(), format!("{expr:?}").into())),
                };
                
                // parse value
                let rexpr = parse_expression(tokens, false, false)?;
                
                cmd.nom_args.insert(lexpr, rexpr);
                no_more_pos_args = true;
            } else {
                if no_more_pos_args {
                    return Err(ParseError::PosArgAfterNomArg(token.start))
                }
                
                // Don't care, push arg, go to next iter.
                cmd.pos_args.push(expr);
            }
        } else {
            break // natural end of command, due to EOS.
        }
    }
    
    Ok(cmd)
}
