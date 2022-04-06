//! Parsing of token-stream into data-structures.

use super::*;

/// Parses the stream of tokens into a list.
pub fn parse_list(
    tokens: &mut PeekableTokenStream<impl TokenStream>
) -> Result<Vec<Expression>, ParseError> {
    let mut list = Vec::default();

    while let Some(t) = tokens.peek() {
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
    
        let expr = parse_expression(tokens, false)?;
        list.push(expr);
    }

    Ok(list)
}

/// Parses the stream of tokens into a key/value-map.
pub fn parse_map(
    tokens: &mut PeekableTokenStream<impl TokenStream>
) -> Result<FxHashMap<CompactString, Expression>, ParseError> {
    let mut map = FxHashMap::default();
    
    while let Some(t) = tokens.peek().cloned() {
        match t.content {
            TokenContent::Remainder(r )
                => return Err(ParseError::Unrecognized(t.start, r)),
            
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
                
                let expr = parse_expression(tokens, false)?;
                map.insert(key, expr);
            },
            TokenContent::Literal(l) => return Err(ParseError::Unexpected(format!("literal {:?}", l).into())),
            g @ TokenContent::Group(_, _) => return Err(ParseError::Unexpected(format!("group {:?}", g).into()))
        };
    }
    
    Ok(map)
}
