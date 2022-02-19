import { describe, expect, it, xit } from "@jest/globals";
import { parseStatements } from "./parser";
import { typecheck } from "./typeck";

describe("typecheck", () => {
  it("accepts a literal", () => {
    expect(() => typecheck(parseStatements("1;"))).not.toThrow();
  });

  it("accepts int-int addition", () => {
    expect(() => typecheck(parseStatements("1 + 2 + 3 + 4;"))).not.toThrow();
  });

  it("accepts f64-f64 addition", () => {
    expect(() => typecheck(parseStatements("1.2 + 3.4 + 5.6 + 7.8;"))).not.toThrow();
  });

  it("rejects int-f64 addition", () => {
    expect(() => typecheck(parseStatements("1 + 2 + 3.4;"))).toThrow(/Invalid types in addition/);
  });

  it("rejects f64-int addition", () => {
    expect(() => typecheck(parseStatements("1.2 + 3.4 + 5;"))).toThrow(/Invalid types in addition/);
  });

  it("accepts identifier", () => {
    expect(() => typecheck(parseStatements("foo + 123;"))).not.toThrow();
    expect(() => typecheck(parseStatements("123 + bar;"))).not.toThrow();
    expect(() => typecheck(parseStatements("foo + bar;"))).not.toThrow();
  })

  it("checks types from declared identifiers", () => {
    expect(() => typecheck(parseStatements(`
      let foo = 1;
      let bar = 2;
      foo + bar;
    `))).not.toThrow();
    expect(() => typecheck(parseStatements(`
      let foo = 1;
      let bar = 2.3;
      foo + bar;
    `))).toThrow(/Invalid types in addition/);
    expect(() => typecheck(parseStatements(`
      let foo = 1;
      let bar = foo + 1;
      foo + bar;
    `))).not.toThrow();
    expect(() => typecheck(parseStatements(`
      let foo = 1;
      let bar = foo + 1;
      let baz = 2.3;
      foo + baz;
    `))).toThrow(/Invalid types in addition/);
  });

  it("allows shadowing the existing identifier", () => {
    expect(() => typecheck(parseStatements(`
      let foo = 1;
      let bar = foo + 1;
      let foo = 1.23;
      let baz = foo + 2.0;
      bar + 10;
      baz + 10.0;
    `))).not.toThrow();
    expect(() => typecheck(parseStatements(`
      let foo = 1;
      let bar = foo + 1;
      let foo = 1.23;
      let baz = foo + 2.0;
      bar + baz;
    `))).toThrow(/Invalid types in addition/);
  });
});
