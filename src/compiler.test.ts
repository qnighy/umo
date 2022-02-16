import path from "path";
import fs from "fs";
import { describe, expect, it, it as realIt, xit } from "@jest/globals";
import { compile } from "./compiler";
import { ParseError } from "./parser";
import { TypeCheckerError } from "./typeck";

describe("compile", () => {
  it("returns a string", () => {
    expect(typeof compile("1 + 1;")).toBe("string");
  });

  it("Compiles a text to JS", () => {
    expect(compile("1 + 1;")).toBe("(1n + 1n);\n");
  });

  it("errors on parse error", () => {
    expect(() => compile("1 +")).toThrow(/Unexpected EOF/);
  });

  describe("testcases", () => {
    const testcasesDir = path.resolve(__dirname, "__testcases__");
    const testcaseNames = fs.readdirSync(testcasesDir);
    for (const testcaseName of testcaseNames) {
      const testcaseDir = path.resolve(testcasesDir, testcaseName);
      const config = JSON.parse(fs.readFileSync(path.resolve(testcaseDir, "config.json"), "utf8"));
      const input = fs.readFileSync(path.resolve(testcaseDir, "input.umo"), "utf8");
      const it = config.pending ? xit : realIt;
      if (config.parseError) {
        it(testcaseName, () => {
          expect(() => compile(input)).toThrow(ParseError);
        });
      } else if (config.typeCheckError) {
        it(testcaseName, () => {
          expect(() => compile(input)).toThrow(TypeCheckerError);
        });
      } else {
        const expected = fs.readFileSync(path.resolve(testcaseDir, "output.js"), "utf8");
        it(testcaseName, () => {
          expect(compile(input)).toBe(expected);
        });
      }
    }
  });
});
