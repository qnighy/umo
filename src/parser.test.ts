import { describe, expect, it, xit } from "@jest/globals";
import { Expression, parse, tokenize } from "./parser";

describe("parse", () => {
  it("parses binary 1 + 1", () => {
    const expected: Expression = {
      type: "AddExpression",
      lhs: { type: "NumberExpression", value: 1 },
      rhs: { type: "NumberExpression", value: 1 },
    };
    expect(parse("1 + 1")).toEqual(expected);
  });

  it("parses ternary 1 + 2 + 3", () => {
    const expected: Expression = {
      type: "AddExpression",
      lhs: {
        type: "AddExpression",
        lhs: { type: "NumberExpression", value: 1 },
        rhs: { type: "NumberExpression", value: 2 },
      },
      rhs: { type: "NumberExpression", value: 3 },
    };
    expect(parse("1 + 2 + 3")).toEqual(expected);
  });

  xit("parses parentheses", () => {
    const expected: Expression = {
      type: "AddExpression",
      lhs: { type: "NumberExpression", value: 1 },
      rhs: {
        type: "AddExpression",
        lhs: { type: "NumberExpression", value: 2 },
        rhs: { type: "NumberExpression", value: 3 },
      },
    };
    expect(tokenize("1 + (2 + 3)")).toEqual(expected);
  });

  it("errors on early EOF", () => {
    expect(() => parse("")).toThrow(/Unexpected EOF/);
  });

  it("errors on initial-position unknown token", () => {
    expect(() => parse("\\")).toThrow(/Unexpected token/);
  });

  it("errors on mid-position unknown token", () => {
    expect(() => parse("1 \\")).toThrow(/Unexpected token/);
  });
});

describe("tokenize", () => {
  it("tokenizes a text", () => {
    expect(tokenize("1 + 1")).toEqual(["1", "+", "1"]);
  });

  xit("tokenizes a number", () => {
    expect(tokenize("123 + 456")).toEqual(["123", "+", "456"]);
  });
});
