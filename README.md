# IMPRAL

IMPRAL is a command parsing and evaluation library for a LISP-ish dialect, intended for reasonably ergonomic and specialized command-line input *within* larger applications, games and frameworks.

> **NOTICE:**  
> Currently incomplete/still in development. Expect breaking changes.  
> *Do not use in production code.*

## Introduction

### Features

A very quick overview:

- Basic language similar to LISP/Scheme.
- Mostly safe ~~and panic-free~~ parser.
- Conversion to an AST happens only once.
- A small-ish set of literal types...
  - `_`
  - `null`
  - Booleans (`true` / `false`)
  - Characters
  - Integers and floats (both 64-bit)
    - Integer radixes: `0b…`, `0o…`, `0d…`, `0x…`
  - Strings (compact at & below 24 bytes)
    - Including barewords!
  - ~~Byte-Arrays~~
  - [UUID's](https://lib.rs/crates/uuid), prefixed with `U`.
- Data structures:
  - Lists (eg: `[ 1, 2, 3 ]`, commas optional)
  - Dicts (eg: `{ a=1, b=2, c=3 }`, commas optional)
  - Radix number lists (`0x[C0 +FF -EE]`)
- References!
  - Result reference: `$`
  - Context reference: `$$`
  - Local references: `$my-ref`
  - Global references: `@my-ref`
  - `A && B`: `B` only runs if `A` succeeds.
  - `A || B`: `B` only runs if `A` fails.
- Operators!
  - Arithmetic (`+ - * / **`)
  - Equality (`== != < > <= >=`)
  - Misc. (`? ! ~ ^ ++ --`)
- Fields and indices!
  - Any item followed by a dot and a name is a field access: `_.name`
  - Any item followed by a bracketed expression is a index access: `_[index]`
- Ranges!
  - Any two expressions, that are not themselves ranges, separated by two dots.
  - Optional 'last-inclusive' flag can be set by adding a `=` after the dots.
  - i.e.: `_ .. _` and `_ ..= _`
- Pipes!
  - First expression is the *source*
  - Expressions separated by `|`
  - Filters: `… |? bar | …`
  - Folding: `… |! 0 min $ $acc | …`
  - Collecting: `… |!`
  - Chaining! `… | … |? … | …`
- Fallible operations!
  - Any expression followed by `?` gets unwrapped to the default value.
  - Any expression followed by `?!` will throw an error.
- Positional parameters: `foo _ _`
- Named parameters: `foo bar=_ baz=_`
- Subexpressions, enclosed by `(…)`
- ...and other small things!

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
- **Dicts**: A dictionary, too, can be created in two ways...
  - Trough syntax: `{ key1: val1, key2: val2, …, keyN: valN}`
    > There *must* be one or more `,` between the key-value pairs;
    > there *may* be a `,` before the `}`.
  - By command: `dict key1 val1 key2 val2 … keyN valN`

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
	Neither positional nor named arguments may be placed before the command identifier.

2. **The positional arguments.**  
	A whitespace separated list of values.

3. **The named arguments.**  
	A whitespace separated list of `key=value`-pairs; the keys are *always* barewords.  
	Named arguments are *required* to be written *after* the positional arguments.  
	The only exception to this are continuation commands in the last position.  
  One may also write bool-arguments, consisting of a `+` or `-` and a bareword as name.

4. **Continuation command.** (*optional*)  
	Another command that is an extra positional parameter in the last position, written after a `:`.

5. **Command Delimiter.** (*optional*)  
	If the parser encounters a `;`, it will stop parsing the current command,
  regardless of what comes after the semicolon; useful for sequences?

To sum this up:

- Basic command syntax: `symbol arg1 arg2 … argN`
- Named parameters:     `symbol … kvarg1=val1 kvarg2=val2 … kvargN=valN`
- Flag parameters:      `symbol … +kvarg -kvarg …`
- With continuation:    `symbol … …: command`

### Subexpressions

Expressions can be enclosed in parentheses, to be used within other expressions and as arguments for other commands:  `(…)`

### Logical Operators

By writing two commands separated by `&&`, the latter command will only be executed if the former *succeeds*, with the result being bound to `$`: `foo … && bar $ …`

By separating them with `||` instead, the latter command will only be executed if the former *fails*: `foo … || bar …`

Both of the logical operators may be chained; evaluation will occur from left to right.

### Command Pipes

A sequence of expressions can be written as a `pipe`, in which each stage passes it's result (`$`) to the next one: `players |? < $.health 50 | heal $`

If a stage returns something iterable, that iterator will be evaluated
and it's items be passed thru the pipe, instead of the iterator itself.

### Field- and Index-Access

By using the `_.FIELD`- and `_[INDEX]`-syntax, subitems (properties/fields/elements/etc.) may be accessed.

### Ranges

By typing two consecutive dots (`..` and `..=` for right-inclusive), a range between/of two expressions can be created.

### Fallibility

By using the `?` postfix-operator, one can convert the given value into a default value, if it's `null` or an error. Adding an exclamation mark (`?!`) makes the expression throw an error.

---

## TODO

- [ ] Pratt Parsing with `= …`: https://lib.rs/crates/pratt
- [ ] Numbers with Units
- [ ] Validation
- [ ] Interpreter
