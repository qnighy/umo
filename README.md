# Umo programming language

## Concept

- Effect system (not implemented)
- Subtyping (not implemented)
- Opt-in shared mutability (not implemented)
- Gradual typing (not implemented)

## Expressions

### Primary

- Parentheses: `1 + (2 + 3)` (not implemented)

### Numbers

- Decimal integer: `/\d+/` (octal-like numbers like `018` may be forbidden later)
- Hexadecimal integer (not implemented)
- Octal integer (not implemented)
- Binary integer (not implemented)
- Decimal floating-point number `/\d+\.\d+/`

### Operators

- Additive: `+`, `-` (Only `+` is implemented now)
- Multiplicative: `*`, `/` (not implemented)
- Relational: `<`, `<=`, `>`, `>=` (not implemented)
- Equality: `==`, `!=` (not implemented)
- Logical: `&&`, `||` (not implemented)
- Bitwise: `&`, `|`, `^` (not implemented)
- Shift: `<<`, `>>` (not implemented)
- Unary: `-`, `!` (not implemented)

## Development

Concept:

- Document-first
- Test-first

Prototype will be written in TypeScript
