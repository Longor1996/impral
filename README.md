# IMPRAL

IMPRAL is a command parsing and evaluation library for a LISP dialect, intended for reasonably ergonomic and specialized commandline input.

**DISCLAIMER:** Currently incomplete/still in development. *Do not use.*

## Introduction

### Features

A very quick overview:

- Basic language similar to LISP/Scheme.
- Mostly safe ~~and panic-free~~ parser.
- Conversion to an AST happens only once.
- A small-ish set of literal types...
  - `Null`
  - Booleans
  - Characters
  - Integers and floats (both 64-bit)
    - Integer radixes: `0b…`, `0o…`, `0d…`, `0x…`
  - Strings (compact at & below 24 bytes)
    - Including barewords!
  - ~~Byte-Arrays~~
- Data structures:
  - Lists (eg: `[ 1, 2, 3 ]`, commas optional)
  - Dicts (eg: `{ a=1, b=2, c=3 }`, commas optional)
  - Radixed number lists (`0x[C0 +FF -EE]`)
- References!
  - Local references: `$my-ref`
  - Global references: `@my-ref`
  - Result reference: `$`
  - Context reference: `$$`
- Pipes!
  - First command is the *source*
  - Commands separated by `|`
  - Filters: `… |? bar | …`
- Subcommands! (enclosed by `(…)`)
- Arbitrary member access (`foo.bar`)

## Syntax & Semantics

### Literals

A literal is a simple value, like a number, string, boolean, etc. etc.

Following is a list of possible literals:

- **Nothing**: The absence of a value; written as `null`.
- **Booleans**: There is `true` and `false`. That's it.
- **Numbers**: Numbers can be written in a variety of ways...
  - `1337`
  - `-1`
  - `42.69`
  - `1.0e-5`
  - `0b101010`
  - `0xC0FFEE`
- **Barewords**: A bareword is any sequence of characters that consists entirely of letters,
                 digits, `_` and `-`, always starting with at least one letter.
- **Strings**: Any text enclosed in double-quotes! `"Hello, World!"`
- **Lists**: A list can be created in two ways...
  - Trough syntax: `[item1, item2, … itemN]` (the commas are optional!)
  - By command: `list item1 item2 … itemN`
- **Maps**: A map, too, can be created in two ways...
  - Trough syntax: `{ key1: val1, key2: val2, …, keyN: valN}`
    > There *must* be one or more `,` between the key-value pairs;
    > there *may* be a `,` before the `}`.
  - By command: `mmap key1 val1 key2 val2 … keyN valN`

### References

There are several types of reference:

- **Global References**: Written as `@NAME`.
- **Local References**: Written as `$NAME` or `$NUMBER`.
- **Result Reference**: Written as `$`.
- **Context Reference**: Written as `$$`.

### Command Syntax

The language, like any Lisp does, consists of commands (function calls) stored as *lists*,
where the first item in the list is a *symbol*, representing the name of the specific command to be evaluated followed by any number of positional arguments. Where things deviate is that IMPRAL supports named parameters, written after the positional arguments, for sake of convenience.

So, a command consists of three (and a half) parts:

1. **The symbol identifying the command.**  
	A unique bareword or any of the built-in operators.
	Neither positional nor named arguments must come before the command identifier.

2. **The positional arguments.**  
	A whitespace separated list of values.

3. **The named arguments.**  
	A whitespace separated list of `key=value`-pairs; the keys are *always* barewords.  
	Named arguments are *required* to be written *after* the positional arguments.  
	The only exception to this are continuation commands in the last position.

4. **Continuation command.** (*optional*)  
	Another command that is an extra positional parameter in the last position, written after a `:`.

To sum this up:

- Basic command syntax: `symbol arg1 arg2 … argN`
- Named parameters:     `symbol … kvarg1=val kvarg2=val … kvargN=val`
- With continuation:    `symbol … …: command`

#### Subcommands

Commands can be enclosed in parentheses and be used as arguments for other commands:  `(symbol …)`

#### Logical Operators

By writing two commands separated by `&&`, the latter command will only be executed if the former *succeeds*, with the result being bound to `$`: `foo … && bar $ …`

By separating them with `||` instead, the latter command will only be executed if the former *fails*: `foo … || bar …`

#### Command Pipes

A sequence of commands can be written as a `pipe`, in which every command passes it's result (`$`) to the next command: `players | where $$.health less 50 | heal $`

If a command returns an iterator, the iterator's items will be passed thru the pipe, instead of the iterator itself.

### Indexing

By using the `.`/`.?`-syntax, members of values may be accessed.

### Ranges

By typing two consecutive dots (`..`), a range between/of two expressions can be created.

### Exists?

By using the `?` postfix-operator, one can test if the given value is `null`.

### Relation

> TODO: Specifiy how the relation/relative-to operator should work.

## TODO

- [ ] Pratt Parsing: https://lib.rs/crates/pratt
- [ ] Units?
- [ ] Iterators?
- [ ] Validation?
- [ ] Interpreter?
