//! Helper functions for the parser.
use super::*;

/// Match token if...
pub fn match_if(
    tokens: &mut PeekableTokenStream<impl TokenStream>,
    predicate: impl FnOnce(&TokenContent) -> bool
) -> bool {
    match tokens.peek() {
        Some(Token { content, .. }) => predicate(content),
        None => false
    }
}

/// Consume token if...
pub fn consume_if(
    tokens: &mut PeekableTokenStream<impl TokenStream>,
    predicate: impl FnOnce(&TokenContent) -> bool
) -> bool {
    match tokens.peek() {
        Some(Token { content, .. }) => if predicate(content) {
            tokens.next(); true
        } else {
            false
        },
        None => false
    }
}

/// Consume string...
pub fn consume_string(
    tokens: &mut PeekableTokenStream<impl TokenStream>
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
pub fn consume_group(
    tokens: &mut PeekableTokenStream<impl TokenStream>,
    symbol: Symbol,
) -> Option<PeekableTokenStream<impl TokenStream>> {
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
            return Some(tokens.into_iter().peekmore())
        }
    };
    
    None
}

/// Match a symbol.
pub fn match_symbol(
    tokens: &mut PeekableTokenStream<impl TokenStream>,
    symbol: Symbol,
) -> bool {
    match_if(tokens, |tc|
        matches!(tc, TokenContent::Symbol(peeked) if *peeked == symbol)
    )
}

/// Consume a symbol.
pub fn consume_symbol(
    tokens: &mut PeekableTokenStream<impl TokenStream>,
    symbol: Symbol,
) -> bool {
    consume_if(tokens, |tc|
        matches!(tc, TokenContent::Symbol(peeked) if *peeked == symbol)
    )
}
