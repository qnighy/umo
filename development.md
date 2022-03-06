## Requirements

- Node.js https://nodejs.org/
- yarn https://yarnpkg.com/

Note: the current compiler, written in TypeScript, is just a prototype.
The development requirements may change in the future.

## Testing

```
yarn install
yarn test
```

## Directory structure

- src/parser.ts ... AST, tokenizer, and parser
- src/typeck.ts ... types and type checker
- src/compiler.ts ... emitter, currently targeting JS
