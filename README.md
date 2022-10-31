# IMPRAL

## Introduction
IMPRAL is a command parsing and evaluation library for a LISP-ish dialect, intended for reasonably ergonomic and specialized command-line input *within* larger applications, games and frameworks.

A work-in-progress guide to the language itself, is included in the crate documentation; see [`crate::guide`]/<https://docs.rs/impral/latest/impral/guide/index.html>.

> **NOTICE:**  
> Currently incomplete/still in development. Expect breaking changes.  
> *Do not use in production code.*

## Features

A very basic overview:

- Basic language similar to LISP/Scheme.
- Mostly safe ~~and panic-free~~ parser.
- Conversion to an AST happens only once.
- AST is linearized; no tree visitor here!
- A small-ish set of literal types...
  - `_`
  - `null`
  - Booleans (`true` / `false`)
  - Characters
  - Integers and floats (both 64-bit)
  - Strings (compact at & below 24 bytes)
    - Quoteless strings (barewords)!
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
- Operators!
  - Arithmetic (`+ - * / **`)
  - Equality (`== != < > <= >=`)
  - Misc. (`? ! ~ ^ ++ --`)
- Fields and indices!
  - Named fields: `_.name`
  - Indexed fields: `_.[index]`
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
- Conditional operations!
  - `A && B`: `B` only runs if `A` succeeds.
  - `A || B`: `B` only runs if `A` fails.
- Positional parameters: `foo _ _`
- Named parameters: `foo bar=_ baz=_`
- Subexpressions, enclosed by `(…)`
- ...and other small things!

---

## TODO

- [ ] Validation
- [ ] Interpreter
