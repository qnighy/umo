import path from "path";
import fs from "fs";
import { describe, expect, it, it as realIt, xit } from "@jest/globals";
import { compile } from "./compiler";

describe("compile", () => {
  it("returns a string", () => {
    expect(typeof compile("1 + 1")).toBe("string");
  });

  xit("Compiles a text to JS", () => {
    expect(compile("1 + 1")).toBe("1n + 1n");
  });

  describe("testcases", () => {
    const testcasesDir = path.resolve(__dirname, "__testcases__");
    const testcaseNames = fs.readdirSync(testcasesDir);
    for (const testcaseName of testcaseNames) {
      const testcaseDir = path.resolve(testcasesDir, testcaseName);
      const config = JSON.parse(fs.readFileSync(path.resolve(testcaseDir, "config.json"), "utf8"));
      const input = fs.readFileSync(path.resolve(testcaseDir, "input.umo"), "utf8");
      const expected = fs.readFileSync(path.resolve(testcaseDir, "output.js"), "utf8");
      const it = config.pending ? xit : realIt;
      it(testcaseName, () => {
        expect(compile(input)).toBe(expected);
      });
    }
  });
});
