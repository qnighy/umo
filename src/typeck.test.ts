import { describe, expect, it, xit } from "@jest/globals";
import { parseExpression } from "./parser";
import { typecheck } from "./typeck";

describe("typecheck", () => {
  it("accepts a literal", () => {
    expect(() => typecheck(parseExpression("1"))).not.toThrow();
  });

  it("accepts int-int addition", () => {
    expect(() => typecheck(parseExpression("1 + 2 + 3 + 4"))).not.toThrow();
  });

  it("accepts f64-f64 addition", () => {
    expect(() => typecheck(parseExpression("1.2 + 3.4 + 5.6 + 7.8"))).not.toThrow();
  });

  it("rejects int-f64 addition", () => {
    expect(() => typecheck(parseExpression("1 + 2 + 3.4"))).toThrow(/Invalid types in addition/);
  });

  it("rejects f64-int addition", () => {
    expect(() => typecheck(parseExpression("1.2 + 3.4 + 5"))).toThrow(/Invalid types in addition/);
  });

  it("accepts identifier", () => {
    expect(() => typecheck(parseExpression("foo + 123"))).not.toThrow();
    expect(() => typecheck(parseExpression("123 + bar"))).not.toThrow();
    expect(() => typecheck(parseExpression("foo + bar"))).not.toThrow();
  })
});
