//! The tokenizer.

use super::*;

/// A peekable stream of tokens.
#[allow(type_alias_bounds)]
pub type PeekableTokenStream<TS: TokenStream> = Peekable<TS>;

/// A stream of tokens.
pub trait TokenStream: Iterator<Item = Token> {}
impl<T> TokenStream for T where T: Iterator<Item = Token> {}

/// Creates an iterator of `Token`'s from the given string slice.
/// 
/// This does not create `Group`-tokens.
pub fn tokenize(input: &str) -> PeekableTokenStream<impl TokenStream + '_> {
    
    let mut input = input.char_indices().peekmore();
    let mut encode = [0; 4*2];
    
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
        
        let mut last_idx = index;
        
        // Turn both the current char and the current with the next char into string slices.
        let (currstr, peekstr) = {
            let cl0 = current.encode_utf8(&mut encode).len();
            let cl1 = input.peek()
                .map(|c|c.1)
                .unwrap_or(' ')
                .encode_utf8(&mut encode[cl0..]).len();
            (
                std::str::from_utf8(&encode[..(cl0)]).unwrap(),
                std::str::from_utf8(&encode[..(cl0+cl1)]).unwrap()
            )
        };
        
        // Check for symbol pairings...
        if let Ok(symbol) = peekstr.parse::<Symbol>() {
            input.next(); // drop the next char too
            return Some((index, index+2, symbol).into());
        }
        
        // Check for individual symbols...
        if let Ok(symbol) = currstr.parse::<Symbol>() {
            // '+' and '-' are handled elsewhere...
            if ! (symbol == Symbol::Plus || symbol == Symbol::Dash) {
                // ...everything else is immediately turned into a token.
                return Some((index, index+1, symbol).into());
            }
        }
        
        // Check for start of bareword...
        if current.is_alphabetic() || current == '_' {
            let mut buffer = CompactString::new();
            buffer.push(current);
            
            while let Some((index, peeked)) = input.peek().copied() {
                last_idx = index;
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
                "null" => return Some((index, last_idx, Literal::Nil).into()),
                "true" => return Some((index, last_idx, Literal::Bool(true)).into()),
                "false" => return Some((index, last_idx, Literal::Bool(false)).into()),
                "NaN" => return Some((index, last_idx, Literal::Dec(f64::NAN)).into()),
                "infinity" => return Some((index, last_idx, Literal::Dec(f64::INFINITY)).into()),
                _ => ()
            }
            
            return Some((index, last_idx, Literal::Str(buffer)).into());
        }
        
        // Check for start of string...
        if current == '"' {
            let mut buffer = CompactString::new();
            let mut last = current;
            while let Some((_index, peeked)) = input.peek().copied() {
                last_idx = index;
                if peeked == '"' && last != '\\' {
                    input.next(); // drop the `"`
                    break;
                } else {
                    buffer.push(peeked);
                    input.next(); // eat char
                    last = peeked;
                }
            }
            
            return Some((index, last_idx, Literal::Str(buffer)).into());
        }
        
        // Check for start of string...
        if current == '\'' {
            let mut buffer = CompactString::new();
            let mut last = current;
            while let Some((index, peeked)) = input.peek().copied() {
                last_idx = index;
                if peeked == '\'' && last != '\\' {
                    input.next(); // drop the `"`
                    break;
                } else {
                    buffer.push(peeked);
                    input.next(); // eat char
                    last = peeked;
                }
            }
            
            return Some((index, last_idx, Literal::Str(buffer)).into());
        }
        
        // Check for start of number...
        if current.is_ascii_digit() || current == '+' || current == '-' {
            let peeked = input.peek().copied().map(|c|c.1).unwrap_or('0');
            
            let (current, sign) = match current {
                '-' => {
                    if !peeked.is_ascii_digit() {
                        return Some((index, index+1, Symbol::Dash).into());
                    }
                    
                    (input.next().map(|c|c.1).unwrap_or('0'), -1.0f64)
                },
                '+' => {
                    if !peeked.is_ascii_digit() {
                        return Some((index, index+1, Symbol::Plus).into());
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
            
            let decimal = if radix == 10
                && '.' == input.peek_nth(0).copied().map(|c|c.1).unwrap_or(' ')
                
                // This is here so that two dot's in a row, a range, are not eaten.
                && '.' != input.peek_nth(1).copied().map(|c|c.1).unwrap_or(' ')
            {
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
                    return Some((index, index, Literal::Int((sign as i64) * integer * (pow10 as i64))).into());
                }
                return Some((index, index, Literal::Dec((sign) * (integer as f64) * (pow10 as f64))).into());
            }
            
            let value = (sign) * (integer as f64 + decimal) * (pow10 as f64);
            return Some((index, index, Literal::Dec(value)).into());
        }
        
        let remainder: String = input.clone().map(|(_, c)| c).collect();
        Some((index, index+remainder.len(), TokenContent::Remainder(remainder)).into())
        //panic!("Unable to parse token starting with '{}' at position {}", current, index)
    }).peekable()
}

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
                tokens.next();
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
    }).peekable()
}
