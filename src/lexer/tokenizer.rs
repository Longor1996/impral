//! The tokenizer.

use super::*;
use peekmore::PeekMoreIterator;

/// A peekable stream of tokens.
#[allow(type_alias_bounds)]
pub type PeekableTokenStream<TS: TokenStream> = PeekMoreIterator<TS>;

/// A stream of tokens.
pub trait TokenStream: Iterator<Item = Token> {}
impl<T> TokenStream for T where T: Iterator<Item = Token> {}

/// Creates an iterator of `Token`'s from the given string slice.
/// 
/// This does not create `Group`-tokens.
pub fn tokenize(input: &str) -> PeekableTokenStream<impl TokenStream + '_> {
    
    let mut input = PosIter::from(input.char_indices()).peekmore();
    let mut encode = [0; std::mem::size_of::<char>() * 2];
    
    std::iter::from_fn(move || {
        // Skip any and all whitespace...
        let current = loop {
            match input.next() {
                Some(current) => if current.is_whitespace() {
                    // keep going
                } else {
                    break current;
                },
                None => return None
            };
        };
        
        let index = current.idx;
        let mut last_idx = current.idx;
        
        // Turn both the current char and the current with the next char into string slices.
        let (currstr, peekstr) = {
            let cl0 = current.encode_utf8(&mut encode).len();
            let cl1 = input.peek()
                .map(|c|c.char)
                .unwrap_or(' ')
                .encode_utf8(&mut encode[cl0..]).len();
            (
                std::str::from_utf8(&encode[..(cl0)]).unwrap(),
                std::str::from_utf8(&encode[..(cl0+cl1)]).unwrap()
            )
        };
        
        // Check for symbol pairings...
        if let Ok(symbol) = peekstr.parse::<Symbol>() {
            input.next(); // drop the paired char
            
            if symbol == Symbol::DoubleDollar {
                // Context Reference
                return Some((index, index+2, Literal::RefCtx).into());
            }
            
            return Some((index, index+2, symbol).into());
        }
        
        // Check for individual symbols...
        if let Ok(symbol) = currstr.parse::<Symbol>() {
            
            if symbol == Symbol::At { // '@' object references
                
                // TODO: Parse Object Reference (ID)
                // (requires integer lexing to be extracted)
                
                if let Some((start, end, uuid)) = try_lex_uuid(&mut input, current.idx) {
                    return Some((start, end, Literal::ObjUid(uuid)).into());
                }
                
                // Parse Object Key Reference
                if let Some(PosChar { char, .. }) = input.peek().cloned() {
                    // Check for start of bareword...
                    if is_bareword_start(char) {
                        input.next(); // drop start
                        let (start, end, bareword)
                            = try_lex_bareword(&mut input, index, char);
                        
                        return Some((start, end, Literal::ObjKey(bareword)).into());
                    }
                    
                    // Check for start of double-quoted string...
                    if char == '"' {
                        input.next(); // drop start
                        let (start, end, string) = try_lex_string(&mut input, index, char, '"');
                        return Some((start, end, Literal::ObjKey(string)).into());
                    }
                    
                    // Check for start of single-quoted string...
                    if char == '\'' {
                        input.next(); // drop start
                        let (start, end, string) = try_lex_string(&mut input, index, char, '\'');
                        return Some((start, end, Literal::ObjKey(string)).into());
                    }
                }
                
                panic!("Failed to parse object reference (@) at {}:{}", current.line, current.col);
            }
            
            if symbol == Symbol::DollarSign {
                // Parse Local Reference
                if let Some(PosChar { char, .. }) = input.peek().cloned() {
                    // Check for start of bareword...
                    if is_bareword_start(char) {
                        input.next(); // drop start
                        let (start, end, bareword)
                            = try_lex_bareword(&mut input, index, char);
                        
                        return Some((start, end, Literal::RefVar(bareword)).into());
                    }
                }
                
                // Result Reference
                return Some((index, index+1, Literal::RefRes).into());
            }
            
            // '+' and '-' are handled elsewhere...
            if ! (symbol == Symbol::Plus || symbol == Symbol::Dash) {
                // ...everything else is immediately turned into a token.
                return Some((index, index+1, symbol).into());
            }
        }
        
        // Check for start of UUID...
        if *current == 'U' {
            if let Some((start, end, uuid)) = try_lex_uuid(&mut input, current.idx) {
                return Some((start, end, Literal::Uid(uuid)).into());
            }
        }
        
        // Check for start of bareword...
        if is_bareword_start(*current) {
            let (start, end, bareword) = try_lex_bareword(&mut input, index, *current);
            
            return Some((start, end,
                try_into_constant(bareword.as_str()).unwrap_or(Literal::Str(bareword))
            ).into());
        }
        
        // Check for start of double-quoted string...
        if *current == '"' {
            let (start, end, string) = try_lex_string(&mut input, index, *current, '"');
            return Some((start, end, Literal::Str(string)).into());
        }
        
        // Check for start of single-quoted string...
        if *current == '\'' {
            let (start, end, string) = try_lex_string(&mut input, index, *current, '\'');
            return Some((start, end, Literal::Str(string)).into());
        }
        
        // NOTE: This is the worst code of this lexer!
        // Check for start of number...
        if current.is_ascii_digit() || *current == '+' || *current == '-' {
            let peeked = input.peek().copied().map(|c|c.char).unwrap_or('0');
            
            let (current, sign, bsign) = match *current {
                '-' => {
                    if !peeked.is_ascii_digit() {
                        return Some((index, index+1, Symbol::Dash).into());
                    }
                    
                    (input.next().map(|c|c.char).unwrap_or('0'), -1.0f64, true)
                },
                '+' => {
                    if !peeked.is_ascii_digit() {
                        return Some((index, index+1, Symbol::Plus).into());
                    }
                    
                    (input.next().map(|c|c.char).unwrap_or('0'), 1.0f64, true)
                },
                _ => (*current, 1.0f64, false)
            };
            
            let mut buffer = CompactString::new();
            buffer.push(current);
            
            // Check radix.
            let mut radix = 10;
            if current == '0' {
                // Peek the next char and check if it is a RADIX indicator...
                radix = match input.peek().copied().map(|c|c.char).unwrap_or(' ') {
                    // If there is a match, eat it and return a different radix...
                    'x' => {input.next(); 16},
                    'd' => {input.next(); 10},
                    'o' => {input.next(); 8},
                    'b' => {input.next(); 2},
                    _ => radix
                };
            }
            
            // TODO: Break numeric array parsing out into a new function.
            if !bsign {
                if let Some(PosChar { char: '[', .. }) = input.peek().copied() {
                    // Array of numbers!
                    input.next(); // eat [
                    let mut array: Vec<Token> = vec![]; // TODO: Make this an i64-vec
                    
                    loop {
                        if let Some(PosChar { char: ']', idx: index , .. }) = input.peek().copied() {
                            last_idx = index;
                            input.next(); // eat ]
                            break; // end of array
                        }
                        
                        buffer.clear();
                        
                        let mut sign = false;
                        
                        if let Some(PosChar { char: '-', .. }) = input.peek().copied() {
                            sign = true;
                            input.next();
                        } else if let Some(PosChar { char: '+', .. }) = input.peek().copied() {
                            sign = false;
                            input.next();
                        }
                        
                        // Eat all the INTEGER digits...
                        while let Some(PosChar { char: peeked, idx: index , .. }) = input.peek().copied() {
                            if peeked == ']' { break }
                            last_idx = index;
                            
                            if ! is_digit_valid(peeked, radix) {
                                input.next(); // eat unknown
                                break
                            }
                            
                            buffer.push(peeked);
                            input.next(); // eat digit
                        }
                        
                        if buffer.is_empty() {
                            continue;
                        }
                        
                        let integer = match i64::from_str_radix(&buffer, radix) {
                            Ok(i) => i,
                            Err(err) => {
                                panic!("Failed to parse '{}' with radix {}: {}", buffer, radix, err)
                                //return Some((index, last_idx, TokenContent::Remainder(buffer.into())).into());
                            },
                        };
                        
                        let integer = if sign { -integer } else { integer };
                        array.push((last_idx, last_idx, Literal::Int(integer)).into());
                    }
                    
                    return Some((index, last_idx, TokenContent::Group(Symbol::BraketLeft, array)).into());
                }
            }
            
            // Eat all the INTEGER digits...
            while let Some(PosChar { char: peeked, .. }) = input.peek().copied() {
                if ! is_digit_valid(peeked, radix) {
                    break
                }
                
                buffer.push(peeked);
                input.next(); // eat digit
            }
            
            let integer = match i64::from_str_radix(&buffer, radix) {
                Ok(i) => i,
                Err(err) => panic!("Failed to parse '{}' with radix {}: {}", buffer, radix, err),
            };
            
            let decimal = if radix == 10
                && '.' == input.peek_nth(0).copied().map(|c|c.char).unwrap_or(' ')
                
                // This is here so to allow member-access on numbers.
                && input.peek_nth(1).copied().map(|c|c.char).unwrap_or(' ').is_ascii_digit()
                
                // This is here so that two dot's in a row, a range, are not eaten.
                && '.' != input.peek_nth(1).copied().map(|c|c.char).unwrap_or(' ')
            {
                // Eat all the DECIMALS...
                buffer.clear(); // reuse the buffer, cuz why not
                buffer.push_str("0.");
                input.next(); // eat the `.`
                
                while let Some(PosChar { char: peeked, .. }) = input.peek().copied() {
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
            
            let pow10: f64 = if radix == 10 && 'e' == input.peek().copied().map(|c|c.char).unwrap_or(' ') {
                input.next(); // eat the `e`
                
                let sign = match input.peek().copied().map(|c|c.char).unwrap_or(' ') {
                    '+' => {input.next(); false}, // eat sign
                    '-' => {input.next(); true}, // eat sign
                    _ => false, // dont eat
                };
                
                buffer.clear(); // reuse the buffer, cuz why not
                while let Some(PosChar { char: peeked , .. }) = input.peek().copied() {
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
        
        let remainder: String = input.clone().map(|p| p.char).collect();
        Some((index, index+remainder.len(), TokenContent::Remainder(remainder)).into())
    }).peekmore()
}

fn try_lex_bareword(input: &mut PosInput, start: usize, current: char) -> (usize, usize, CompactString) {
    let mut end = start;
    let mut buffer = CompactString::new();
    buffer.push(current);
    
    while let Some(PosChar { char: peeked, idx: index , .. }) = input.peek().copied() {
        end = index;
        if is_bareword_part(peeked) {
            buffer.push(peeked);
            input.next(); // eat char
            // NOTE: This could be implemented using `input.next_if`.
        } else {
            break;
        }
    }
    
    (start, end, buffer)
}

fn try_lex_string(input: &mut PosInput, start: usize, current: char, delimiter: char) -> (usize, usize, CompactString) {
    let mut buffer = CompactString::new();
    let mut last = current;
    let mut end = start;
    
    #[allow(clippy::while_let_loop)]
    loop {
        let PosChar { char: peeked, idx: index , .. } = match input.peek().copied() {
            Some(i) => i,
            None => break // TODO: Return error?
        };
        
        end = index;
        if peeked == delimiter && last != '\\' {
            input.next(); // drop the `"`
            break;
        } else {
            buffer.push(peeked);
            input.next(); // eat char
            last = peeked;
        }
    }
    
    (start, end, buffer)
}

/// Try lex uuid.
fn try_lex_uuid(input: &mut PosInput, start: usize) -> Option<(usize, usize, uuid::Uuid)> {
    
    // A uuid is always 36 ascii chars long...
    let mut uuid_str: [u8; 36] = [b' '; 36];
    
    // TODO: Make this simpler/safer?
    // Try to peek-convert-collect 36 chars into our buffer...
    let len = input // Peek
        .peek_amount(36)
        .iter()
        .flatten()
        // Convert
        .filter_map(|c| std::convert::TryInto::<u8>::try_into(c.char).ok())
        // Collect
        .enumerate()
        .inspect(|(i,c)| uuid_str[*i] = *c)
        .count()
    ;
    
    if let Ok(uuid) = uuid::Uuid::try_parse_ascii(&uuid_str) {
        // We must consume the chars we've read!
        for _ in 0..len { input.next(); }
        return Some((start, start + len, uuid));
    }
    
    None
}

/// Attempts to convert a bareword into a constant literal.
fn try_into_constant(str: &str) -> Option<Literal> {
    Some(match str {
        "null" => Literal::Nil,
        "true" => Literal::Bool(true),
        "false" => Literal::Bool(false),
        "NaN" => Literal::Dec(f64::NAN),
        "inf" => Literal::Dec(f64::INFINITY),
        "infinity" => Literal::Dec(f64::INFINITY),
        "PI" => Literal::Dec(std::f64::consts::PI),
        "TAU" => Literal::Dec(std::f64::consts::TAU),
        "EULER" => Literal::Dec(std::f64::consts::E),
        "SQRT2" => Literal::Dec(std::f64::consts::SQRT_2),
        _ => return None
    })
}

/// Checks if a given digit is valid under the provided radix.
fn is_digit_valid(peeked: char, radix: u32) -> bool {
    match radix {
        2 if peeked == '0' || peeked == '1' => true,
        8 if ('0'..='7').contains(&peeked) => true,
        10 if peeked.is_ascii_digit() => true,
        16 if peeked.is_ascii_hexdigit() => true,
        _ => false
    }
}
