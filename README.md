# IMPRAL

## Introduction

> Originally called *TaleCraft Engine Command Processor System* and developed for the **Talecraft Game Engine** (*TCGE*) by **Longor1996**, ***IMPRAL*** is a simple command processing language, intended for use in a commandline/*REPL*.

Currently incomplete/still in development. Do not use.

## Syntax & Semantics

### Command Syntax

The language, like any Lisp does, consists of commands (function calls) stored as *lists*,
where the first item in the list is a *symbol*, representing the name of the specific command to be evaluated.

A command consists of three (and a half) parts and may contain line breaks:

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

- Basic Command Syntax: `symbol arg1 arg2 … argN kvarg1=val kvarg2=val … kvargN=val`
- With continuation: `symbol … …: command`

#### Subcommands

Commands can be enclosed in parentheses and be used as arguments for other commands:  `(symbol …)`

#### Falliable Commands

One may write two commands in succession, separated by an ampersand/`&`, in which case the latter command will only be executed if the former succeeds, with the result being bound to `$$`: `foo … & bar $$ …`

#### Command Pipes

A sequence of commands can be written as a `pipe`, in which every command passes it's result (`$$`) to the next command: `players | where $$.health less 50 | heal $$`

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

### Variables

There are several types of variable:

- **Global Variables**: Written as `@NAME`.
- **Local Variables**: Written as `$NAME` or `$NUMBER`.
- **Result Variable**: Written as `$$`.

### Indexing

By using either the `.`/`.?`-syntax or the `idx`/`idxn`-commands, values may have sub-values.

### Exists?

By using the `?` postfix-operator, one can test if the given value is `null`.

### Relation

> TODO: Specifiy how the relation/relative-to operator should work.

## TODO

- [ ] Ranges
- [ ] Units
- [ ] Interpreter
