//! The groupenizer takes a stream of tokens and converts it into a *tree* of token streams.
use super::*;

/// Find and stack groups from the given stream of tokens.
pub fn groupenize(tokens: &mut PeekableTokenStream<impl TokenStream>, delimiter: Option<Symbol>) -> PeekableTokenStream<impl TokenStream + '_> {
    std::iter::from_fn(move || {
        match tokens.next() {
            Some(Token {
                content: TokenContent::Symbol(
                    symbol @ (
                        Symbol::ParenLeft |
                        Symbol::CurlyLeft |
                        Symbol::BraketLeft
                    )
                ),
                start,
                end
            }) => {
                let group: Vec<Token> = groupenize(tokens, symbol.get_delimiter()).collect();
                let end = group.last().map(|t| t.end).unwrap_or_else(|| end+1);
                let group = TokenContent::Group(symbol, group);
                //tokens.next();
                Some(Token {
                    content: group,
                    start,
                    end
                })
            },
            Some(Token {
                content: TokenContent::Symbol(
                    symbol
                ),
                ..
            }) if delimiter.map(|d| d == symbol).unwrap_or(false) => {
                None // end of current group
            },
            Some(token) => Some(token),
            
            // TODO: Check for unmatched delimiters by `if let None = delimiter`
            None => None, // natural end
        }
    }).peekmore()
}
