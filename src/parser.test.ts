import { describe, expect, it, xit } from "@jest/globals";
import { Expression, parseExpression, parseStatements, Statement, tokenize } from "./parser";

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
  });

  it("errors on an invalid token", () => {
    expect(() => parseStatements("1 + 1#")).toThrow(/Unexpected token: #/);
  });
});

describe("tokenize", () => {
  it("tokenizes a text", () => {
    expect(tokenize("1 + 1")).toEqual(["1", "+", "1"]);
  });

  it("tokenizes a number", () => {
    expect(tokenize("123 + 456")).toEqual(["123", "+", "456"]);
  });

  it("tokenizes a floating-point number", () => {
    expect(tokenize("123.040 + 456.789")).toEqual(["123.040", "+", "456.789"]);
  });

  it("tokenizes a stray dot after integer", () => {
    expect(tokenize("123.x")).toEqual(["123", ".", "x"]);
  });

  it("tokenizes an identifier", () => {
    expect(tokenize("foo123 + abc_def")).toEqual(["foo123", "+", "abc_def"]);
  });

  it("disallows identifiers starting with digits", () => {
    expect(tokenize("123foo")).toEqual(["123", "foo"]);
  });
});
