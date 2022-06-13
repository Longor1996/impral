//! Tests to ensure stuff works.

use super::*;

fn chk(input: &str) -> Result<(), ParseError> {
    let mut stream = tokenize(input);
    let mut stream = groupenize(&mut stream, None);
    
    let output = parse_expression(
        &mut stream,
        true,
        true
    );
    
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
    
    eprintln!("SizeOf µSTR  = {}", size_of::<CompactString>());
    eprintln!("SizeOf [Byt] = {}", size_of::<Vec<Byt>>());
    eprintln!("SizeOf ->Byt = {}", size_of::<Box<Byt>>());
    eprintln!("SizeOf LEX.L = {}", size_of::<Literal>());
    eprintln!("SizeOf LEX.S = {}", size_of::<Symbol>());
    eprintln!("SizeOf AST = {}", size_of::<Expression>());
    eprintln!("- SizeOf AST.I = {}", size_of::<Invoke>());
    eprintln!("- SizeOf AST.P = {}", size_of::<Pipe>());
    eprintln!("SizeOf [AST;1] = {}", size_of::<ExpressionVec>());
    //eprintln!("SizeOf EXE.V = {}", size_of::<crate::values::ValContainer>());
    
    //assert!(size_of::<crate::values::ValContainer>() == 8, "The size of a ValContainer-struct should be exactly 8 bytes.");
    assert!(dbg!(size_of::<Invoke>() <= 128), "The size of an Invoke-struct should be below 128 bytes.");
}

#[test]
fn should_succeed() -> Result<(), ParseError> {
    println!(": Constants");
    chk("null")?;
    chk("true")?;
    chk("false")?;
    chk("NaN")?;
    chk("inf")?;
    chk("infinity")?;
    chk("+infinity")?;
    chk("-infinity")?;
    chk("PI")?;
    chk("TAU")?;
    chk("EULER")?;
    chk("SQRT2")?;
    
    println!();
    println!(": Numbers");
    chk("12345")?;
    chk("3.141")?;
    chk("180°")?;
    chk("180.foobar")?;
    
    println!();
    println!(": Quoted Strings");
    chk("\"Hello, World!\"")?;
    chk("\"Oooops...")?;
    
    println!();
    println!(": References");
    chk("$")?;
    chk("$$")?;
    chk("$_")?;
    chk("$n")?;
    chk("$abcdef")?;
    chk("$_0123456789_")?;
    
    println!();
    println!(": Object References");
    chk("@_")?;
    chk("@abcdef")?;
    chk("@'quoted \" string'")?;
    chk("@\"quoted ' string\"")?;
    chk("@67e55044-10b1-426f-9247-bb680e5fe0c8")?;
    //chk("$ $$ $$ $$")?;
    
    println!();
    println!(": Numeric Arrays");
    chk("0x[FF 01 02 03 04]")?;
    chk("0d[-1 +1 -1 +1 -1]")?;
    
    println!();
    println!(": List Structures");
    chk("[1, 2, 3, 4, 5,]")?;
    chk("[1  2  3  4  5 ]")?;
    chk("[foo bar baz]")?;
    chk("[foo bar [foo bar baz]]")?;
    chk("[foo bar [foo bar [foo bar baz]]]")?;
    
    println!();
    println!(": Dict Structures");
    chk("{a=1, b=2, c=-3,}")?;
    chk("{a=1  b=2  c=-3 }")?;
    chk("{a=null b={a=1, b=2, c=-3,} c={a=1 b=2 c=-3}}")?;
    
    println!();
    println!(": Operators");
    chk("= 1 2 3")?;
    chk("+ 1 2 3")?;
    chk("- 1 2 3")?;
    chk("* 1 2 3")?;
    chk("/ 1 2 3")?;
    chk("++ 1 2 3")?;
    chk("-- 1 2 3")?;
    chk("<= 1 2 3")?;
    chk(">= 1 2 3")?;
    chk("chk $$")?;
    chk("ß ßß")?;
    chk("anything-can-be-an-operator 42")?;
    
    println!();
    println!(": Parameters");
    chk("test 1.234 2.345 1.99999 0.000001")?;
    chk("test 1 2 3")?;
    chk("test 1 2 3 a=4")?;
    chk("mul 2 (+ 1 2 3)")?;
    chk("test foo: bar baz")?;
    chk("test foo; bar baz")?;
    chk("test [1 2 3 4 5]")?;
    chk("test {a = 1, b=2, c=-3}")?;
    
    println!();
    println!(": Pipes");
    chk("testA 1 2 3 | testB 4 5 6 | testC 7 8 9")?;
    chk("maybe-null |? accepts-null")?;
    chk("outer | v1 | (inner v2 | v3) | v4")?;
    
    println!();
    println!(": Execution Modifiers");
    chk("conditional && execution")?;
    chk("alternative || execution")?;
    
    println!();
    println!(": Examples");
    chk("echo \"Hello, World!\" @s.chat ")?;
    chk("tp @a 0 0 0")?;
    chk("tp @a @world.spawn")?;
    chk("tp @a 0 100 0 rel=@self")?;
    chk("for @a: tp [0 100 0]~$$")?;
    chk("test 0..10")?;
    chk("test (get1)..(get2 $$)")?;
    
    println!();
    chk("alias FOO (BAR ARG)")?;
    chk("alias FOO: BAR ARG")?;
    
    println!();
    chk("e tag=FOO|del")?;
    chk("e in=(box 0 0 0 8 8 8)|del")?;
    chk("e is=item|del")?;
    chk("$$|?le $.health 10|heal 10 5")?;
    
    println!();
    chk("v fill (box -8 -8 -8 +8 +8 +8|offset $$) air")?;
    chk("v|raytrace $$ 10m|v set $ air")?;
    chk("v|raymarch $$ 10m|?is solid|v set $ glass")?;
    
    println!();
    chk("raytrace $$ 10m elod=sphere")?;
    chk("raytrace $$ 10m elod=bounds")?;
    chk("raytrace $$ 10m elod=voxels")?;
    chk("raytrace $$ 10m elod=hitbox")?;
    chk("raytrace $$ 10m elod=phybox")?;
    
    println!();
    chk("e|sphere $ 0.5m|raytrace $ max|?is marker|del")?;
    chk("tp 0 0 0 motion=0")?;
    chk("move forward for=1")?;
    chk("set $$.motion: * 0.5 $")?;
    chk("gamerules +foo -bar")?;
    chk("e get U67e55044-10b1-426f-9247-bb680e5fe0c8")?;
    chk("e get @67e55044-10b1-426f-9247-bb680e5fe0c8")?;
    
    Ok(())
}

#[test]
#[should_panic]
fn posarg_after_nomarg() {
    chk("test 1 a=2 3 b=4").expect("positional arguments cannot be written after nominal arguments");
}
