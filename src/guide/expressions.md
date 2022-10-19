# Expressions

An [`Expression`] is a self-contained tree of both data and code.

## Subexpressions
Expressions can be enclosed in parentheses, to be used within other expressions and as arguments for other commands:  `(…)`

### Ranges
By typing two consecutive dots (`..` and `..=` for right-inclusive), a range between/of two expressions can be created.

### Fallibility
By using the `?` postfix-operator, one can convert the given value into a default value, if it's `null` or an error. Adding an exclamation mark (`?!`) makes the expression throw an error, forcefully ending evaluation.



