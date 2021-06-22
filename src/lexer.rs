use std::fmt::Write;

pub fn tokenize(input: &str) -> impl Iterator<Item = Token> + '_ {
    
    let mut input = input.char_indices().peekable();
    
    std::iter::from_fn(move || {
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
        
        match current {
            '(' => return Some((index, Symbol::ParenLeft).into()),
            ')' => return Some((index, Symbol::ParenRight).into()),
            '[' => return Some((index, Symbol::BraketLeft).into()),
            ']' => return Some((index, Symbol::BraketRight).into()),
            '{' => return Some((index, Symbol::CurlyLeft).into()),
            '}' => return Some((index, Symbol::CurlyRight).into()),
            '<' => return Some((index, Symbol::AngleLeft).into()),
            '>' => return Some((index, Symbol::AngleRight).into()),
            //'+' => return Some((index, Symbol::Plus).into()), // HANDLED ELSEWHERE
            //'-' => return Some((index, Symbol::Dash).into()), // HANDLED ELSEWHERE
            '*' => return Some((index, Symbol::Star).into()),
            '/' => return Some((index, Symbol::Slash).into()),
            '~' => return Some((index, Symbol::Tilde).into()),
            '#' => return Some((index, Symbol::Hash).into()),
            ',' => return Some((index, Symbol::Comma).into()),
            '.' => return Some((index, Symbol::Dot).into()),
            ':' => return Some((index, Symbol::DoubleDot).into()),
            ';' => return Some((index, Symbol::Semicolon).into()),
            '_' => return Some((index, Symbol::Underscore).into()),
            '=' => return Some((index, Symbol::EqualSign).into()),
            '?' => return Some((index, Symbol::QuestionMark).into()),
            '!' => return Some((index, Symbol::ExclamationMark).into()),
            '$' => return Some((index, Symbol::DollarSign).into()),
            '%' => return Some((index, Symbol::Percentage).into()),
            '&' => return Some((index, Symbol::Ampersand).into()),
            '|' => return Some((index, Symbol::Pipe).into()),
            '^' => return Some((index, Symbol::Caret).into()),
            _ => () // continue on...
        }
        
        // Check for start of bareword...
        if current.is_ascii_alphabetic() || current == '_' {
            let mut buffer = String::from(current);
            
            while let Some((_index, peeked)) = input.peek().copied() {
                if peeked.is_alphanumeric() || peeked == '_' || peeked == '-' {
                    buffer.push(peeked);
                    input.next(); // eat char
                } else {
                    break;
                }
            }
            
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
            let mut buffer = String::new();
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
            let mut buffer = String::new();
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
            
            let mut buffer = String::from(current);
            
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
        
        panic!("Unable to parse token starting with '{}' at position {}", current, index)
    })
}

#[derive(Debug, Clone)]
pub struct Token {
    pub position: usize,
    pub content: TokenContent
}

impl From<(usize, Symbol)> for Token {
    fn from(src: (usize, Symbol)) -> Self {
        Token {
            position: src.0,
            content: TokenContent::Symbol(src.1)
        }
    }
}

impl From<(usize, Literal)> for Token {
    fn from(src: (usize, Literal)) -> Self {
        Token {
            position: src.0,
            content: TokenContent::Literal(src.1)
        }
    }
}

impl From<(usize, TokenContent)> for Token {
    fn from(src: (usize, TokenContent)) -> Self {
        Token {
            position: src.0,
            content: src.1
        }
    }
}

#[derive(Debug, Clone)]
pub enum TokenContent {
    Symbol(Symbol),
    Literal(Literal),
}

#[derive(Clone, Copy)]
pub enum Symbol {
    ParenLeft,
    ParenRight,
    BraketLeft,
    BraketRight,
    CurlyLeft,
    CurlyRight,
    AngleLeft,
    AngleRight,
    Plus,
    Dash,
    Star,
    Hash,
    Slash,
    Tilde,
    Comma,
    Dot,
    DoubleDot,
    Semicolon,
    Underscore,
    EqualSign,
    QuestionMark,
    ExclamationMark,
    DollarSign,
    Percentage,
    Ampersand,
    Pipe,
    Caret,
}

impl std::fmt::Display for Symbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let char = match self {
            Symbol::ParenLeft => '(',
            Symbol::ParenRight => ')',
            Symbol::BraketLeft => '[',
            Symbol::BraketRight => ']',
            Symbol::CurlyLeft => '{',
            Symbol::CurlyRight => '}',
            Symbol::AngleLeft => '<',
            Symbol::AngleRight => '>',
            Symbol::Plus => '+',
            Symbol::Dash => '-',
            Symbol::Star => '*',
            Symbol::Hash => '#',
            Symbol::Slash => '/',
            Symbol::Tilde => '~',
            Symbol::Comma => ',',
            Symbol::Dot => '.',
            Symbol::DoubleDot => ':',
            Symbol::Semicolon => ';',
            Symbol::Underscore => '_',
            Symbol::EqualSign => '=',
            Symbol::QuestionMark => '?',
            Symbol::ExclamationMark => '!',
            Symbol::DollarSign => '$',
            Symbol::Percentage => '%',
            Symbol::Ampersand => '&',
            Symbol::Pipe => '|',
            Symbol::Caret => '^',
        };
        
        f.write_char(char)
    }
}

impl std::fmt::Debug for Symbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, " {} ", self)
    }
}

#[derive(Debug, Clone)]
pub enum Literal {
    Nil,
    Bool(bool),
    Char(char),
    Int(i64),
    Dec(f64),
    Str(String),
    Byt(Vec<u8>)
}

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
        tokenize("0 1 2 3 4 5 6 7 8 9 12345 -123").count();
        tokenize("0b101010 0o10 0d10 0x1F 0.1 0.001 0. 1e3 -0xFF").count();
        tokenize("0.1 0.001 0. 1e3 10e-3 -0.5").count();
    }
    
    #[test]
    fn lex_example() {
        tokenize("blocks (b box 0 0 0 15 15 15) set air").inspect(|t| println!("{:?}",t)).count();
    }
}