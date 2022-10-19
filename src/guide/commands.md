# Commands


## Syntax
The language, like any Lisp does, consists of commands (function calls) stored as *lists*,
where the first item in the list is a *symbol*, representing the name of the specific command
to be evaluated followed by any number of positional arguments.

Where things deviate is that IMPRAL supports named parameters,
written after the positional arguments, for sake of convenience.

Thus the syntax consists of 4 parts: the symbol, positional arguments,
named arguments and an optional continuation.

1. **The symbol identifying the command.**<br>  
  A unique bareword ([`crate::lexer::Literal::Str`]) or any of the built-in operators ([`crate::lexer::Symbol`]).
  Neither positional nor named arguments may be placed before the command identifier.

2. **The positional arguments.**<br>  
  A whitespace separated list of [`crate::parser::Expression`]s.

3. **The named arguments.**<br>  
  A whitespace separated list of `key=value`-pairs; the keys are *always* barewords ([`crate::lexer::Literal::Str`]).  
  Named arguments are *required* to be written *after* the positional arguments.  
  One may also write bool-arguments, consisting of a `+` or `-` and a bareword as name.

4. **Continuation command.** (*optional*)<br>  
  Another command that is an extra positional parameter in the last position, written after a `:`.

**Tl;Dr:**
- Basic command syntax: `symbol arg1 arg2 … argN`
- Named parameters:     `symbol … kvarg1=val1 kvarg2=val2 … kvargN=valN`
- Flag parameters:      `symbol … +kvarg -kvarg …`
- With continuation:    `symbol … …: command`

### Logical Operators

By writing two commands separated by `&&`, the latter command will only be executed if the former *succeeds*, with the result being bound to `$`: `foo … && bar $ …`

By separating them with `||` instead, the latter command will only be executed if the former *fails*: `foo … || bar …`

Both of the logical operators may be chained; evaluation will occur from left to right.
