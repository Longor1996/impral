//! Lexer Tests

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
