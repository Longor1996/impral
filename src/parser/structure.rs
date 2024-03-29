//! Parsing of token-stream into data-structures.

use super::*;

/// Parses the stream of tokens into a list.
pub fn parse_list(
    parser: &mut Parser,
    tokens: &mut PeekableTokenStream
) -> Result<ExpressionVec, ParseError> {
    let mut list = ExpressionVec::default();
    
    loop {
        if tokens.peek().is_none() {
            break;
        }
        
        if consume_symbol(tokens, Symbol::BraketRight) {
            break;
        }
        
        if consume_symbol(tokens, Symbol::Comma) {
            continue;
        }
        
        let expr = parse_expression(parser, tokens, false, true)?;
        list.push(expr);
    }
    
    Ok(list)
}

/// Parses the stream of tokens into a key/value-map.
pub fn parse_map(
    parser: &mut Parser,
    tokens: &mut PeekableTokenStream
) -> Result<FxHashMap<CompactString, BlockRef>, ParseError> {
    let mut map = FxHashMap::default();
    
    loop {
        if tokens.peek().is_none() {
            break;
        }
        
        if consume_symbol(tokens, Symbol::CurlyRight) {
            break;
        }
        
        if consume_symbol(tokens, Symbol::Comma) {
            continue;
        }
        
        if let Some(key) = consume_string(tokens) {
            if ! consume_symbol(tokens, Symbol::EqualSign) {
                return Err(ParseError::ExpectButGot("equal-sign".into(), "something else".into()));
            }
            // else: everything checks out, continue on...
            
            let expr = parse_expression(parser, tokens, false, true)?;
            map.insert(key, expr);
            continue;
        }
    }
    
    Ok(map)
}
