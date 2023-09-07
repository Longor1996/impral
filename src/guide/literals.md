# Literals

A [`Literal`] is a simple value, like a number, string, boolean, etc. etc.

Following is a list of possible literals:

## Nothing
The absence of a value; written as `null`.

## Booleans
The usual `true` and `false`. That's it.

## Numbers
Numbers can be written in a variety of ways...

- As simple integers, like `123456789`
- With a decimal point: `0.5`
- In scientific notation: `1.0e+9`
- Binary:  `0b101010`
- Octal:   `0o755`
- Decimal: `0d10`
- Hexadecimal: `0xC0FFEE`

All ways of writing numbers allow a dash (`-`) in front,
to make the number in question negative;
i.e.: `-1337`, `-0.5`, `-1.0e+5`.

Number literals are stored either as [`i64`] or as [`f64`].

## Strings
Any text enclosed in double-quotes! i.e.: `"Hello, World!"`

Thanks to some memory shenanigans (see: [`smartstring::alias::CompactString`]), strings smaller than 24 bytes don't have to be heap-allocated.

### Barewords
A bareword is a *unquoted string* that consists entirely of letters,
digits, `_` and `-`, always starting with at least one letter.

## UUIDs
See [`uuid::Uuid`].

## Bytes
**TODO:** Not exposed via the parser... yet.

## Lists
A list can be created in two ways...

- Trough syntax: `[item1, item2, … itemN]` (the commas are optional!)
- Via [`commands`]: `list item1 item2 … itemN`

## Dicts
A dictionary, too, can be created in two ways...

- Trough syntax: `{ key1: val1, key2: val2, …, keyN: valN}`
  > There *must* be one or more `,` between the key-value pairs;
  > there *may* be a `,` before the `}`.
- Via [`commands`]: `dict key1 val1 key2 val2 … keyN valN`

## References
see [`references`].

References are literals that have no actual value, instead pointing elsewhere for data.
