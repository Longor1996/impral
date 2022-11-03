//! Tests to ensure stuff works.

use super::*;

#[test]
fn sizes() {
    use std::mem::size_of;
    
    // eprintln!("SizeOf Vec<Expression>  = {}", size_of::<Vec<Expression>>());
    // eprintln!("SizeOf Vec<Range<...>>  = {}", size_of::<Vec<std::ops::Range<usize>>>());
    // eprintln!("SizeOf Vec<Literal>     = {}", size_of::<Vec<Literal>>());
    
    eprintln!("SizeOf µSTR  = {}", size_of::<CompactString>());
    eprintln!("SizeOf [Byt] = {}", size_of::<Vec<Byt>>());
    eprintln!("SizeOf ->Byt = {}", size_of::<Box<Byt>>());
    eprintln!("SizeOf LEX.L = {}", size_of::<Literal>());
    eprintln!("SizeOf LEX.S = {}", size_of::<Symbol>());
    eprintln!("SizeOf AST = {}", size_of::<Expression>());
    eprintln!("- SizeOf AST.I = {}", size_of::<FnCall>());
    eprintln!("- SizeOf AST.P = {}", size_of::<Pipe>());
    eprintln!("SizeOf [AST;1] = {}", size_of::<ExpressionVec>());
    //eprintln!("SizeOf EXE.V = {}", size_of::<crate::values::ValContainer>());
    
    //assert!(size_of::<crate::values::ValContainer>() == 8, "The size of a ValContainer-struct should be exactly 8 bytes.");
    assert!(dbg!(size_of::<FnCall>() <= 128), "The size of an FnCall-struct should be below 128 bytes.");
}

const SRC_CONSTANTS: &str = include_str!("./tests/constants.ifn");
const SRC_DICTS: &str = include_str!("./tests/dicts.ifn");
const SRC_EXAMPLES: &str = include_str!("./tests/examples.ifn");
const SRC_FIELD: &str = include_str!("./tests/field.ifn");
const SRC_IFS: &str = include_str!("./tests/ifs.ifn");
const SRC_INFIX: &str = include_str!("./tests/infix.ifn");
const SRC_INDEX: &str = include_str!("./tests/index.ifn");
const SRC_LISTS: &str = include_str!("./tests/lists.ifn");
const SRC_NUMARS: &str = include_str!("./tests/numars.ifn");
const SRC_NUMBERS: &str = include_str!("./tests/numbers.ifn");
const SRC_OBJ_REFS: &str = include_str!("./tests/obj-refs.ifn");
const SRC_OPERATORS: &str = include_str!("./tests/operators.ifn");
const SRC_PARAMS: &str = include_str!("./tests/params.ifn");
const SRC_PIPES: &str = include_str!("./tests/pipes.ifn");
const SRC_RANGE: &str = include_str!("./tests/range.ifn");
const SRC_REFS: &str = include_str!("./tests/refs.ifn");
const SRC_STRINGS: &str = include_str!("./tests/strings.ifn");
const SRC_TRY: &str = include_str!("./tests/try.ifn");

const SRC: &[&str] = &[
    SRC_CONSTANTS,
    SRC_NUMBERS,
    SRC_STRINGS,
    SRC_REFS,
    SRC_OBJ_REFS,
    SRC_NUMARS,
    SRC_LISTS,
    SRC_DICTS,
    SRC_OPERATORS,
    SRC_FIELD,
    SRC_INDEX,
    SRC_RANGE,
    SRC_TRY,
    SRC_PARAMS,
    SRC_PIPES,
    SRC_IFS,
    SRC_INFIX,
    SRC_EXAMPLES,
];

#[test]
fn parse_constants() -> Result<(), ParseError> {
    chks(SRC_CONSTANTS.lines())?;
    Ok(())
}

#[test]
fn parse_numbers() -> Result<(), ParseError> {
    chks(SRC_NUMBERS.lines())?;
    Ok(())
}

#[test]
fn parse_strings() -> Result<(), ParseError> {
    chks(SRC_STRINGS.lines())?;
    Ok(())
}

#[test]
fn parse_references() -> Result<(), ParseError> {
    chks(SRC_REFS.lines())?;
    Ok(())
}

#[test]
fn parse_obj_references() -> Result<(), ParseError> {
    chks(SRC_OBJ_REFS.lines())?;
    Ok(())
}

#[test]
fn parse_numeric_arrays() -> Result<(), ParseError> {
    chks(SRC_NUMARS.lines())?;
    Ok(())
}

#[test]
fn parse_lists() -> Result<(), ParseError> {
    chks(SRC_LISTS.lines())?;
    Ok(())
}

#[test]
fn parse_dicts() -> Result<(), ParseError> {
    chks(SRC_DICTS.lines())?;
    Ok(())
}

#[test]
fn parse_operators() -> Result<(), ParseError> {
    chks(SRC_OPERATORS.lines())?;
    Ok(())
}

#[test]
fn parse_field() -> Result<(), ParseError> {
    chks(SRC_FIELD.lines())?;
    Ok(())
}

#[test]
fn parse_index() -> Result<(), ParseError> {
    chks(SRC_INDEX.lines())?;
    Ok(())
}

#[test]
fn parse_range() -> Result<(), ParseError> {
    chks(SRC_RANGE.lines())?;
    Ok(())
}

#[test]
fn parse_try() -> Result<(), ParseError> {
    chks(SRC_TRY.lines())?;
    Ok(())
}

#[test]
fn parse_params() -> Result<(), ParseError> {
    chks(SRC_PARAMS.lines())?;
    Ok(())
}

#[test]
fn parse_pipes() -> Result<(), ParseError> {
    chks(SRC_PIPES.lines())?;
    Ok(())
}

#[test]
fn parse_ifs() -> Result<(), ParseError> {
    chks(SRC_IFS.lines())?;
    Ok(())
}

#[test]
fn parse_infix() -> Result<(), ParseError> {
    chks(SRC_INFIX.lines())?;
    Ok(())
}

#[test]
fn parse_examples() -> Result<(), ParseError> {
    chks(SRC_EXAMPLES.lines())?;
    Ok(())
}

#[test]
#[ignore = "tests all example files at once"]
fn parse_all() -> Result<(), ParseError> {
    for src in SRC {
        chks(src.lines())?;
    }
    Ok(())
}

#[test]
#[ignore = "outputs html file for visual inspection"]
fn parse_into_html() -> Result<(), ParseError> {
    use std::fmt::*;
    use crate::parser::fmt_html::BlockHtmlPrinter;
    let mut out = String::from(include_str!("./ast/head.html"));
    
    for src in SRC {
        for line in src.lines()
            .filter(|l| !l.is_empty())
            .filter(|l| !l.starts_with("//"))
        {
            match chk(line) {
                Ok(expr) => match BlockHtmlPrinter::from(&expr).to_string() {
                    Ok(html) => writeln!(&mut out, "<div><code>{}</code><br>{}</div>", line, html).unwrap(),
                    Err(err) => writeln!(&mut out, "<div><code>{}</code><br><pre>{:?}</pre></div>", line, err).unwrap(),
                },
                Err(err) => writeln!(&mut out, "<div><code>{}</code><br><pre>{:?}</pre></div>", line, err).unwrap(),
            }
        }
        
        writeln!(&mut out, "<hr>").unwrap();
    }
    
    use strum::IntoEnumIterator;
    writeln!(&mut out, "<h2>Symbols</h2>").unwrap();
    writeln!(&mut out, "<table>").unwrap();
    writeln!(&mut out, "<tr><th>Name</th><th>Symbol</th><th>Attributes</th></tr>").unwrap();
    for sym in Symbol::iter() {
        let symtxt: SymbolName = sym.into();
        writeln!(&mut out, "<tr>").unwrap();
        writeln!(&mut out, "<td>{symtxt}&nbsp;</td>").unwrap();
        writeln!(&mut out, "<td><code>{sym}</code></td>").unwrap();
        write!(&mut out, "<td><code>").unwrap();
        if sym.is_operator() { write!(&mut out, "op ").unwrap() }
        if sym.is_arrow() { write!(&mut out, "arrow ").unwrap() }
        if sym.is_postop().is_some() { write!(&mut out, "postop ").unwrap() }
        if sym.is_delimiter() { write!(&mut out, "delim ").unwrap() }
        writeln!(&mut out, "</code></td>").unwrap();
        writeln!(&mut out, "</tr>").unwrap();
    }
    writeln!(&mut out, "</table>").unwrap();
    
    std::fs::write("parser-tests.html", out.as_bytes()).unwrap();
    Ok(())
}

#[test]
#[should_panic]
fn posarg_after_nomarg() {
    chk("test 1 a=2 3 b=4").expect("positional arguments cannot be written after nominal arguments");
}

fn chk(input: &str) -> Result<Block, ParseError> {
    use peekmore::PeekMore;
    let mut stream = tokenize(input).peekmore();
    let mut stream = groupenize(&mut stream, None);
    
    let mut parser: Parser = Parser::default();
    let output = parse_expression(
        &mut parser,
        &mut stream,
        true,
        true
    );
    
    //parser.block.entry = output;
    
    match output {
        Ok(o) => o,
        Err(err) => {
            println!("Failed to parse: {input}");
            println!("Because: {err}");
            println!("Tokens: {:?}", groupenize(&mut tokenize(input).peekmore(), None).collect::<Vec<_>>());
            
            if let Some(token) = stream.peek() {
                println!("Next token: {token}");
            }
            
            return Err(err);
        },
    };
    
    eprintln!("INPUT:  {},\t PARSED:  {:?}", input, parser.block);
    //eprintln!("<div><code>{}</code><br>{}</div>", input, output.to_html());
    
    Ok(parser.block)
}

fn chks(input: impl Iterator<Item=&'static str>) -> Result<(), ParseError> {
    for line in input
        .filter(|l| !l.is_empty())
        .filter(|l| !l.starts_with("//"))
    {
        chk(line)?;
    }
    Ok(())
}
