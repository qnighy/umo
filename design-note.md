## Semicolon

- JavaScript, Go: supports ASI
  - Psychologically it is better
  - Complicates syntactic matters
- Rust: explicit semicolon
  - This is useful to distinguish two different BlockExpression semantics: `{ a; b }` and `{ a; b; }`.
  - Nonetheless it annoys many programmers.

How do they deal with implicit returns?

- JavaScript: explicit returns, `() => expr` works as a useful tool to eliminate extra `return`s.
- Go: explicit returns, period.
- Rust: implicit returns and `;` plays an important role here.

Conclusion: pending
