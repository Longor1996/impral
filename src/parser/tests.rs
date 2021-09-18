//! Tests to ensure stuff works.

use super::*;
use crate::lexer::tokenize;

fn chk(input: &str) -> Result<(), ParseError> {
    let output = parse_command(&mut tokenize(input), None)?;
    eprintln!("INPUT:  {},\t PARSED:  {}", input, output);
    Ok(())
}

#[test]
fn sizes() {
    use std::mem::size_of;
    eprintln!("SizeOf LEX.L = {}", size_of::<Literal>());
    eprintln!("SizeOf LEX.S = {}", size_of::<Symbol>());
    eprintln!("SizeOf AST.V = {}", size_of::<ValContainer>());
    eprintln!("SizeOf AST.E = {}", size_of::<Expression>());
    eprintln!("SizeOf AST.C = {}", size_of::<Invoke>());
    
    assert!(size_of::<ValContainer>() == 8, "The size of a ValContainer-struct should be exactly 8 bytes.");
    assert!(size_of::<Expression>() == 16, "The size of a Expression-struct should be exactly 16 bytes.");
    assert!(size_of::<Invoke>() <= 96, "The size of an Invoke-struct should be below 96 bytes.");
}

#[test]
fn should_succeed() -> Result<(), ParseError> {
    chk("= 1 2 3")?;
    chk("+ 1 2 3")?;
    chk("- 1 2 3")?;
    chk("* 1 2 3")?;
    chk("/ 1 2 3")?;
    chk("test 1 2 3")?;
    chk("test 1 2 3 a=4")?;
    chk("mul 2 (+ 1 2 3)")?;
    chk("test foo: bar baz")?;
    chk("test [1 2 3 4 5]")?;
    chk("test {a = 1, b=2, c=-3}")?;
    chk("testA 1 2 3 | testB 4 5 6 | testC 7 8 9")?;
    chk("maybe-null |? accepts-null")?;
    chk("conditional & execution")?;
    chk("echo \"Hello, World!\" @s.chat ")?;
    chk("tp @a 0 0 0")?;
    chk("tp @a @world.spawn")?;
    chk("tp @a 0 100 0 rel=@self")?;
    chk("for @a: tp [0 100 0]~$$")?;
    Ok(())
}

#[test]
#[should_panic]
fn should_fail() {
    chk("test 1 a=2 3 b=4").expect("positional arguments cannot be written after nominal arguments");
}