//! Helper functions for the parser.
use super::*;

/// Match token if...
pub fn match_if(
    tokens: &mut PeekableTokenStream,
    predicate: impl FnOnce(&TokenContent) -> bool
) -> bool {
    match tokens.peek() {
        Some(Token { content, .. }) => predicate(content),
        None => false
    }
}

/// Consume token if...
pub fn consume_if(
    tokens: &mut PeekableTokenStream,
    predicate: impl FnOnce(&TokenContent) -> bool
) -> Option<Token> {
    match tokens.peek() {
        Some(Token { content, .. }) => if predicate(content) {
            match tokens.next() {
                Some(token) => Some(token),
                None => unreachable!(),
            }
        } else {
            None
        },
        None => None
    }
}

/// Consume string...
pub fn consume_string(
    tokens: &mut PeekableTokenStream
) -> Option<CompactString> {
    if let Some(Token {
        content: TokenContent::Literal(Literal::Str(str)), ..
    }) = tokens.peek() {
        let str = str.clone();
        tokens.next();
        return Some(str)
    };
    
    None
}

/// Consume group...
pub fn consume_group<'ipit, 'rpit>(
    tokens: &'ipit mut PeekableTokenStream,
    symbol: Symbol,
) -> Option<PeekableTokenStream<'rpit>> {
    if let Some(Token {
        content: TokenContent::Group(peeked, _), ..
    }) = tokens.peek() {
        if symbol != *peeked {
            return None;
        }
        
        if let TokenContent::Group(_delim, tokens)
            = tokens.next().unwrap().content
        {
            use peekmore::PeekMore;
            let tokens: Box<dyn TokenStream> = Box::new(tokens.into_iter());
            return Some(tokens.peekmore())
        }
    };
    
    None
}

/// Match a symbol.
pub fn match_symbol(
    tokens: &mut PeekableTokenStream,
    symbol: Symbol,
) -> bool {
    match_if(tokens, |tc|
        matches!(tc, TokenContent::Symbol(peeked) if *peeked == symbol)
    )
}

/// Consume a symbol.
pub fn consume_symbol(
    tokens: &mut PeekableTokenStream,
    symbol: Symbol,
) -> bool {
    consume_if(tokens, |tc|
        matches!(tc, TokenContent::Symbol(peeked) if *peeked == symbol)
    ).is_some()
}
