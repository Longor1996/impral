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
