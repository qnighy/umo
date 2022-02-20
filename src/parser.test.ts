import { describe, expect, it, xit } from "@jest/globals";
import { Expression, parseExpression, parseStatements, Statement, Token, tokenize } from "./parser";

describe("parseExpression", () => {
  it("parses binary 1 + 1", () => {
    const expected: Expression = {
      type: "AddExpression",
      lhs: { type: "IntegerLiteral", value: 1n },
      rhs: { type: "IntegerLiteral", value: 1n },
    };
    expect(parseExpression("1 + 1")).toEqual(expected);
  });

  it("parses ternary 1 + 2 + 3", () => {
    const expected: Expression = {
      type: "AddExpression",
      lhs: {
        type: "AddExpression",
        lhs: { type: "IntegerLiteral", value: 1n },
        rhs: { type: "IntegerLiteral", value: 2n },
      },
      rhs: { type: "IntegerLiteral", value: 3n },
    };
    expect(parseExpression("1 + 2 + 3")).toEqual(expected);
  });

  xit("parses parentheses", () => {
    const expected: Expression = {
      type: "AddExpression",
      lhs: { type: "IntegerLiteral", value: 1n },
      rhs: {
        type: "AddExpression",
        lhs: { type: "IntegerLiteral", value: 2n },
        rhs: { type: "IntegerLiteral", value: 3n },
      },
    };
    expect(parseExpression("1 + (2 + 3)")).toEqual(expected);
  });

  it("parses floating-point number", () => {
    const expected: Expression = {
      type: "AddExpression",
      lhs: { type: "FloatingPointLiteral", value: 1 },
      rhs: { type: "FloatingPointLiteral", value: 1.25 },
    };
    expect(parseExpression("1.0 + 1.25")).toEqual(expected);
  });

  it("parses identifiers", () => {
    const expected: Expression = {
      type: "AddExpression",
      lhs: {
        type: "VariableReference",
        name: "foo",
      },
      rhs: {
        type: "VariableReference",
        name: "bar",
      },
    };
    expect(parseExpression("foo + bar")).toEqual(expected);
  });

  it("errors on early EOF", () => {
    expect(() => parseExpression("")).toThrow(/Unexpected EOF/);
  });

  it("errors on initial-position unknown token", () => {
    expect(() => parseExpression("\\")).toThrow(/Unexpected token/);
  });

  it("errors on mid-position unknown token", () => {
    expect(() => parseExpression("1 \\")).toThrow(/Unexpected token/);
  });

  it("errors on keywords", () => {
    expect(() => parseExpression("let")).toThrow(/Unexpected token/);
    expect(() => parseExpression("1 + let")).toThrow(/Unexpected token/);
  });
});

describe("parseStatements", () => {
  it("parses an empty text", () => {
    expect(parseStatements("")).toEqual([]);
  });

  it("parses an almost-empty text", () => {
    expect(parseStatements("\n \n")).toEqual([]);
  });

  it("parses a single expression statement", () => {
    const expected: Statement[] = [
      {
        type: "ExpressionStatement",
        expression: {
          type: "AddExpression",
          lhs: {
            type: "IntegerLiteral",
            value: 1n,
          },
          rhs: {
            type: "IntegerLiteral",
            value: 1n,
          },
        }
      }
    ];
    expect(parseStatements("1 + 1;")).toEqual(expected);
  });

  // May change to auto-insert semicolons
  it("errors on missing semicolon", () => {
    expect(() => parseStatements("1 + 1")).toThrow(/Unexpected EOF/);
    expect(() => parseStatements("let x = 1 + 1")).toThrow(/Unexpected EOF/);
  });

  it("errors on an invalid token", () => {
    expect(() => parseStatements("1 + 1#")).toThrow(/Unexpected token: #/);
    expect(() => parseStatements("let x = 1 + 1#")).toThrow(/Unexpected token: #/);
    expect(() => parseStatements("let x 1 + 1")).toThrow(/Unexpected token: integer literal 1/);
    expect(() => parseStatements("let")).toThrow(/Unexpected EOF/);
    expect(() => parseStatements("let let = 1;")).toThrow(/Unexpected token: let/);
    expect(() => parseStatements("let [] = 1;")).toThrow(/Unexpected token: \[/);
  });

  it("parses let statements", () => {
    const expected: Statement[] = [
      {
        type: "LetStatement",
        lhs: "x",
        rhs: {
          type: "AddExpression",
          lhs: {
            type: "IntegerLiteral",
            value: 1n,
          },
          rhs: {
            type: "IntegerLiteral",
            value: 2n,
          },
        },
      },
      {
        type: "LetStatement",
        lhs: "y",
        rhs: {
          type: "AddExpression",
          lhs: {
            type: "VariableReference",
            name: "x",
          },
          rhs: {
            type: "VariableReference",
            name: "x",
          },
        },
      },
      {
        type: "ExpressionStatement",
        expression: {
          type: "AddExpression",
          lhs: {
            type: "VariableReference",
            name: "y",
          },
          rhs: {
            type: "VariableReference",
            name: "y",
          },
        },
      },
    ];
    expect(parseStatements(`
      let x = 1 + 2;
      let y = x + x;
      y + y;
    `)).toEqual(expected);
  });
});

describe("tokenize", () => {
  it("tokenizes a text", () => {
    const expected: Token[] = [
      { type: "IntegerLiteralToken", value: 1n, start: { line: 0, column: 0 }, end: { line: 0, column: 1 } },
      { type: "SymbolicToken", value: "+", start: { line: 0, column: 2 }, end: { line: 0, column: 3 } },
      { type: "IntegerLiteralToken", value: 1n, start: { line: 0, column: 4 }, end: { line: 0, column: 5 } },
    ];
    expect(tokenize("1 + 1")).toEqual(expected);
  });

  it("tokenizes a number", () => {
    const expected: Token[] = [
      { type: "IntegerLiteralToken", value: 123n, start: { line: 0, column: 0 }, end: { line: 0, column: 3 } },
      { type: "SymbolicToken", value: "+", start: { line: 0, column: 4 }, end: { line: 0, column: 5 } },
      { type: "IntegerLiteralToken", value: 456n, start: { line: 0, column: 6 }, end: { line: 0, column: 9 } },
    ];
    expect(tokenize("123 + 456")).toEqual(expected);
  });

  it("tokenizes a floating-point number", () => {
    const expected: Token[] = [
      { type: "FloatingPointLiteralToken", value: 123.04, start: { line: 0, column: 0 }, end: { line: 0, column: 7 } },
      { type: "SymbolicToken", value: "+", start: { line: 0, column: 8 }, end: { line: 0, column: 9 } },
      { type: "FloatingPointLiteralToken", value: 456.789, start: { line: 0, column: 10 }, end: { line: 0, column: 17 } },
    ];
    expect(tokenize("123.040 + 456.789")).toEqual(expected);
  });

  it("tokenizes a stray dot after integer", () => {
    const expected: Token[] = [
      { type: "IntegerLiteralToken", value: 123n, start: { line: 0, column: 0 }, end: { line: 0, column: 3 } },
      { type: "SymbolicToken", value: ".", start: { line: 0, column: 3 }, end: { line: 0, column: 4 } },
      { type: "IdentifierToken", name: "x", start: { line: 0, column: 4 }, end: { line: 0, column: 5 } },
    ];
    expect(tokenize("123.x")).toEqual(expected);
  });

  it("tokenizes an identifier", () => {
    const expected: Token[] = [
      { type: "IdentifierToken", name: "foo123", start: { line: 0, column: 0 }, end: { line: 0, column: 6 } },
      { type: "SymbolicToken", value: "+", start: { line: 0, column: 7 }, end: { line: 0, column: 8 } },
      { type: "IdentifierToken", name: "abc_def", start: { line: 0, column: 9 }, end: { line: 0, column: 16 } },
    ];
    expect(tokenize("foo123 + abc_def")).toEqual(expected);
  });

  it("disallows identifiers starting with digits", () => {
    const expected: Token[] = [
      { type: "IntegerLiteralToken", value: 123n, start: { line: 0, column: 0 }, end: { line: 0, column: 3 } },
      { type: "IdentifierToken", name: "foo", start: { line: 0, column: 3 }, end: { line: 0, column: 6 } },
    ];
    expect(tokenize("123foo")).toEqual(expected);
  });
});
