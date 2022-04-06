//! Tests to ensure stuff works.

use super::*;

fn chk(input: &str) -> Result<(), ParseError> {
    let mut stream = tokenize(input);
    let mut stream = groupenize(&mut stream, None);
    let output = parse_expression(&mut stream, true);
    
    let output = match output {
        Ok(o) => o,
        Err(err) => {
            println!("Failed to parse: {input}");
            println!("Because: {err}");
            println!("Tokens: {:?}", groupenize(&mut tokenize(input), None).collect::<Vec<_>>());
            
            if let Some(token) = stream.peek() {
                println!("Next token: {token}");
            }
            
            return Err(err);
        },
    };
    eprintln!("INPUT:  {},\t PARSED:  {:?}", input, output);
    Ok(())
}

#[test]
fn sizes() {
    use std::mem::size_of;
    
    eprintln!("SizeOf µSTR = {}", size_of::<CompactString>());
    eprintln!("SizeOf [()] = {}", size_of::<Vec<()>>());
    eprintln!("SizeOf LEX.L = {}", size_of::<Literal>());
    eprintln!("SizeOf LEX.S = {}", size_of::<Symbol>());
    eprintln!("SizeOf AST.E = {}", size_of::<Expression>());
    eprintln!("SizeOf [AST.E;1] = {}", size_of::<SmallVec<[Expression;1]>>());
    eprintln!("- SizeOf AST.S = {}", size_of::<Structure>());
    eprintln!("- SizeOf AST.R = {}", size_of::<ReferenceRoot>());
    eprintln!("- SizeOf AST.I = {}", size_of::<Invoke>());
    //eprintln!("SizeOf EXE.V = {}", size_of::<crate::values::ValContainer>());
    
    //assert!(size_of::<crate::values::ValContainer>() == 8, "The size of a ValContainer-struct should be exactly 8 bytes.");
    assert!(size_of::<Invoke>() <= 128, "The size of an Invoke-struct should be below 128 bytes.");
}

#[test]
fn should_succeed() -> Result<(), ParseError> {
    chk("null")?;
    chk("true")?;
    chk("false")?;
    chk("12345")?;
    chk("3.141")?;
    chk("NaN")?;
    chk("inf")?;
    chk("infinity")?;
    chk("PI")?;
    chk("TAU")?;
    chk("EULER")?;
    chk("SQRT2")?;
    chk("180°")?;
    chk("180.foobar")?;
    chk("\"Hello, World!\"")?;
    chk("\"Oooops...")?;
    chk("0x[FF 01 02 03 04]")?;
    chk("0d[-1 +1 -1 +1 -1]")?;
    chk("[1, 2, 3, 4, 5,]")?;
    chk("[1 2 3 4 5]")?;
    chk("[foo bar baz]")?;
    chk("{a=1, b=2, c=-3,}")?;
    chk("{a=1 b=2 c=-3}")?;
    chk("= 1 2 3")?;
    chk("+ 1 2 3")?;
    chk("- 1 2 3")?;
    chk("* 1 2 3")?;
    chk("/ 1 2 3")?;
    chk("++ 1 2 3")?;
    chk("-- 1 2 3")?;
    chk("chk $$")?;
    chk("ß ßß")?;
    chk("<= 1 2 3")?;
    chk(">= 1 2 3")?;
    chk("test 1.234 2.345 1.99999 0.000001")?;
    chk("test 1 2 3")?;
    chk("test 1 2 3 a=4")?;
    chk("mul 2 (+ 1 2 3)")?;
    chk("test foo: bar baz")?;
    chk("test foo; bar baz")?;
    chk("test [1 2 3 4 5]")?;
    chk("test {a = 1, b=2, c=-3}")?;
    chk("testA 1 2 3 | testB 4 5 6 | testC 7 8 9")?;
    chk("maybe-null |? accepts-null")?;
    chk("conditional && execution")?;
    chk("alternative || execution")?;
    chk("echo \"Hello, World!\" @s.chat ")?;
    chk("tp @a 0 0 0")?;
    chk("tp @a @world.spawn")?;
    chk("tp @a 0 100 0 rel=@self")?;
    chk("for @a: tp [0 100 0]~$$")?;
    chk("test 0..10")?;
    chk("test (get1)..(get2 $$)")?;
    
    chk("alias FOO (BAR ARG)")?;
    chk("alias FOO: BAR ARG")?;
    
    chk("e tag=FOO|del")?;
    chk("e in=(box 0 0 0 8 8 8)|del")?;
    chk("e is=item|del")?;
    chk("$$|?le $.health 10|heal 10 5")?;
    
    chk("v fill (box -8 -8 -8 +8 +8 +8|offset $$) air")?;
    chk("v|raytrace $$ 10m|v set $ air")?;
    chk("v|raymarch $$ 10m|?is solid|v set $ glass")?;
    
    chk("raytrace $$ 10m elod=sphere")?;
    chk("raytrace $$ 10m elod=bounds")?;
    chk("raytrace $$ 10m elod=voxels")?;
    chk("raytrace $$ 10m elod=hitbox")?;
    chk("raytrace $$ 10m elod=phybox")?;
    
    chk("e|sphere $ 0.5m|raytrace $ max|?is marker|del")?;
    
    chk("tp 0 0 0 motion=0")?;
    chk("move forward for=1")?;
    chk("set $$.motion: * 0.5 $")?;
    
    chk("gamerules +foo -bar")?;
    
    Ok(())
}

#[test]
#[should_panic]
fn posarg_after_nomarg() {
    chk("test 1 a=2 3 b=4").expect("positional arguments cannot be written after nominal arguments");
}
