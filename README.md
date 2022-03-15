# Umo programming language

## Concept

- Effect system (not implemented)
- Subtyping (not implemented)
- Opt-in shared mutability (not implemented)
- Gradual typing (not implemented)

## Expressions

### Comment (not an expression though)

Currently `// comment` is implemented

### Primary

- Parentheses: `1 + (2 + 3)`
- Identifiers
- Numbers

### Primary -- Identifiers

Currently `/[a-zA-Z_][a-zA-Z_0-9]*/`

### Primary -- Numbers

- Decimal integer: `/\d+/` (octal-like numbers like `018` may be forbidden later)
- Hexadecimal integer (not implemented)
- Octal integer (not implemented)
- Binary integer (not implemented)
- Decimal floating-point number `/\d+\.\d+/`

### Call/Member expressions

- Call expressions: `foo(bar, baz)`
- Member expressions: `foo.bar` (not implemented)

### Call/Member expressions -- Call expressions

- Empty arguments: `foo()`
- Non-empty arguments: `foo(bar, baz)`
- Trailing comma: `foo(bar, baz,)`

### Operators

- Additive: `+`, `-` (Only `+` is implemented now)
- Multiplicative: `*`, `/` (not implemented)
- Relational: `<`, `<=`, `>`, `>=` (not implemented)
- Equality: `==`, `!=` (not implemented)
- Logical: `&&`, `||` (not implemented)
- Bitwise: `&`, `|`, `^` (not implemented)
- Shift: `<<`, `>>` (not implemented)
- Unary: `-`, `!` (not implemented)

Note on types:

- Arithmetic operators: different types cannot be mixed together.

### Closures

- Closure: `|x| x * 2` (not implemented, subject to change)

## Statements

### Expression statements

Expression + `;` is an expression statement.

(`;` may be auto-inserted in the future)

### Let statements

```
let <ident> = <expr>;
```

The identifiers are in scope after the statement.

#### Scoping

The expression in the `let` statement can reference variables declared before the statement.
Especially, it cannot reference variables declared in the statement itself.

```
// error
let f = |x| f(x);
```

This rule is not implemented yet and subject to change.

## Types

- Primitive types: `int`, `f64`, ... (only `int` and `f64` are implemented now)
- Placeholder type called `Ambiguous` (`Ambiguous<Lo, Up>`) ... corresponds with `any` in TS.
- TBD

## Development

Concept:

- Document-first
- Test-first

Prototype will be written in TypeScript
