//! Lexer that `tokenize`s a string slice into an iterator of `Token`'s.
use std::{convert::TryFrom, iter::Peekable};

use smartstring::alias::CompactString;

/// A peekable stream of tokens.
#[allow(type_alias_bounds)]
pub type PeekableTokenStream<TS: TokenStream> = Peekable<TS>;

/// A stream of tokens.
pub trait TokenStream: Iterator<Item = Token> {}
impl<T> TokenStream for T where T: Iterator<Item = Token> {}

/// Tokenizes a string slice into an iterator of `Token`'s.
pub fn tokenize(input: &str) -> PeekableTokenStream<impl TokenStream + '_> {
    
    let mut input = input.char_indices().peekable();
    
    std::iter::from_fn(move || {
        // Skip any and all whitespace...
        let (index, current) = loop {
            match input.next() {
                Some(c) => if c.1.is_whitespace() {
                    // keep going
                } else {
                    break c;
                },
                None => return None
            };
        };
        
        // Check for individual symbols...
        if let Ok(symbol) = Symbol::try_from(current) {
            // '+' and '-' are handled elsewhere...
            if ! (symbol == Symbol::Plus || symbol == Symbol::Dash) {
                // ...everything else is immediately turned into a token.
                return Some((index, symbol).into());
            }
        }
        
        // Check for start of bareword...
        if current.is_ascii_alphabetic() || current == '_' {
            let mut buffer = CompactString::new();
            buffer.push(current);
            
            while let Some((_index, peeked)) = input.peek().copied() {
                if peeked.is_alphanumeric() || peeked == '_' || peeked == '-' {
                    buffer.push(peeked);
                    input.next(); // eat char
                    // NOTE: This could be implemented using `input.next_if`.
                } else {
                    break;
                }
            }
            
            // Check for literals...
            match buffer.as_str() {
                "null" => return Some((index, Literal::Nil).into()),
                "true" => return Some((index, Literal::Bool(true)).into()),
                "false" => return Some((index, Literal::Bool(false)).into()),
                "NaN" => return Some((index, Literal::Dec(f64::NAN)).into()),
                "infinity" => return Some((index, Literal::Dec(f64::INFINITY)).into()),
                _ => ()
            }
            
            return Some((index, Literal::Str(buffer)).into());
        }
        
        // Check for start of string...
        if current == '"' {
            let mut buffer = CompactString::new();
            let mut last = current;
            while let Some((_index, peeked)) = input.peek().copied() {
                if peeked == '"' && last != '\\' {
                    input.next(); // drop the `"`
                    break;
                } else {
                    buffer.push(peeked);
                    input.next(); // eat char
                    last = peeked;
                }
            }
            
            return Some((index, Literal::Str(buffer)).into());
        }
        
        // Check for start of string...
        if current == '\'' {
            let mut buffer = CompactString::new();
            let mut last = current;
            while let Some((_index, peeked)) = input.peek().copied() {
                if peeked == '\'' && last != '\\' {
                    input.next(); // drop the `"`
                    break;
                } else {
                    buffer.push(peeked);
                    input.next(); // eat char
                    last = peeked;
                }
            }
            
            return Some((index, Literal::Str(buffer)).into());
        }
        
        // Check for start of number...
        if current.is_ascii_digit() || current == '+' || current == '-' {
            let peeked = input.peek().copied().map(|c|c.1).unwrap_or('0');
            
            let (current, sign) = match current {
                '-' => {
                    if !peeked.is_ascii_digit() {
                        return Some((index, Symbol::Dash).into());
                    }
                    
                    (input.next().map(|c|c.1).unwrap_or('0'), -1.0f64)
                },
                '+' => {
                    if !peeked.is_ascii_digit() {
                        return Some((index, Symbol::Plus).into());
                    }
                    
                    (input.next().map(|c|c.1).unwrap_or('0'), 1.0f64)
                },
                _ => (current, 1.0f64)
            };
            
            let mut buffer = CompactString::new();
            buffer.push(current);
            
            // Check radix.
            let mut radix = 10;
            if current == '0' {
                // Peek the next char and check if it is a RADIX indicator...
                radix = match input.peek().copied().map(|c|c.1).unwrap_or(' ') {
                    // If there is a match, eat it and return a different radix...
                    'x' => {input.next(); 16},
                    'd' => {input.next(); 10},
                    'o' => {input.next(); 8},
                    'b' => {input.next(); 2},
                    _ => radix
                };
            }
            
            // Eat all the INTEGER digits...
            while let Some((_index, peeked)) = input.peek().copied() {
                match radix {
                    2 if peeked == '0' || peeked == '1' => (),
                    8 if ('0'..='7').contains(&peeked) => (),
                    10 if peeked.is_ascii_digit() => (),
                    16 if peeked.is_ascii_hexdigit() => (),
                    _ => break
                }
                
                buffer.push(peeked);
                input.next(); // eat digit
            }
            
            let integer = match i64::from_str_radix(&buffer, radix) {
                Ok(i) => i,
                Err(err) => panic!("Failed to parse '{}' with radix {}: {}", buffer, radix, err),
            };
            
            let decimal = if radix == 10 && '.' == input.peek().copied().map(|c|c.1).unwrap_or(' ') {
                // Eat all the DECIMALS...
                buffer.clear(); // reuse the buffer, cuz why not
                buffer.push_str("0.");
                input.next(); // eat the `.`
                while let Some((_index, peeked)) = input.peek().copied() {
                    if peeked.is_ascii_digit() {
                        buffer.push(peeked);
                        input.next(); // eat digit
                    } else {
                        break;
                    }
                }
                
                buffer.parse().unwrap()
            } else {
                0f64
            };
            
            let pow10: f64 = if radix == 10 && 'e' == input.peek().copied().map(|c|c.1).unwrap_or(' ') {
                input.next(); // eat the `e`
                
                let sign = match input.peek().copied().map(|c|c.1).unwrap_or(' ') {
                    '+' => {input.next(); false}, // eat sign
                    '-' => {input.next(); true}, // eat sign
                    _ => false, // dont eat
                };
                
                buffer.clear(); // reuse the buffer, cuz why not
                while let Some((_index, peeked)) = input.peek().copied() {
                    if peeked.is_ascii_digit() {
                        buffer.push(peeked);
                        input.next(); // eat digit
                    } else {
                        break;
                    }
                }
                
                let mut pow10: i32 = buffer.parse().unwrap();
                if sign {pow10 *= -1;}
                10f64.powi(pow10)
            } else {
                1.0
            };
            
            if decimal == 0.0 {
                if pow10 > 0.0 {
                    return Some((index, Literal::Int((sign as i64) * integer * (pow10 as i64))).into());
                }
                return Some((index, Literal::Dec((sign) * (integer as f64) * (pow10 as f64))).into());
            }
            
            let value = (sign) * (integer as f64 + decimal) * (pow10 as f64);
            return Some((index, Literal::Dec(value)).into());
        }
        
        let remainder: String = input.clone().map(|(_, c)| c).collect();
        return Some((index, TokenContent::Remainder(remainder)).into());
        //panic!("Unable to parse token starting with '{}' at position {}", current, index)
    }).peekable()
}

pub mod token;
pub use token::*;

pub mod symbol;
pub use symbol::*;

pub mod literal;
pub use literal::*;

#[cfg(test)]
mod tests {
    #![allow(unused_imports)]
    use super::tokenize;
    
    #[test]
    fn lex_delimiters() {
        tokenize("( () )").count();
        tokenize("[ [] ]").count();
        tokenize("{ {} }").count();
        tokenize("< <> >").count();
    }
    
    #[test]
    fn lex_operators() {
        tokenize("+ - * / ~ # , . : ; _ = ? ! $ % & | ^").count();
    }
    
    #[test]
    fn lex_strings() {
        tokenize("abc-123 null test true false NaN infinity").count();
        tokenize("\"\" \"Hello, World!\" '' 'Hello, World!'").count();
    }
    
    #[test]
    fn lex_numbers() {
        tokenize("0 1 2 3 4 5 6 7 8 9 12345 +123 -123").count();
        tokenize("0b101010 0o10 0d10 0x1F 0.1 0.001 0. 1e3 -0xFF").count();
        tokenize("0.1 0.001 0. 1e3 10e-3 -0.5").count();
    }
    
    #[test]
    #[ignore]
    fn lex_example() {
        tokenize("blocks (b box 0 0 0 15 15 15) set air").inspect(|t| println!("{:?}",t)).count();
    }
}