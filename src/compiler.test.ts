import path from "path";
import fs from "fs";
import { describe, expect, it, it as realIt, xit } from "@jest/globals";
import { compile } from "./compiler";
import { ParseError } from "./parser";
import { TypeCheckerError } from "./typeck";

const updateSnapshot: "new" | "all" | "none" = (expect.getState() as any).snapshotState._updateSnapshot;

function readFileOrNull(path: string): string | null {
  if (fs.existsSync(path)) {
    return fs.readFileSync(path, "utf8");
  } else {
    return null;
  }
}

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
          const expected = readFileOrNull(path.resolve(testcaseDir, "error.txt"));
          let error: unknown = undefined;
          try {
            compile(input);
          } catch(e) {
            error = e;
          }
          expect(error).toBeInstanceOf(ParseError);

          const output = error instanceof ParseError ? error.message : "";
          if ((updateSnapshot === "new" && expected === null) || updateSnapshot === "all") {
            if (output !== expected) {
              fs.writeFileSync(path.resolve(testcaseDir, "error.txt"), output, "utf8");
              console.log(`Updated snapshot for ${testcaseName}`);
            }
          } else {
            if (expected === null) console.error(`Missing snapshot for ${testcaseName}`);
            expect(expected).not.toBeNull();
            expect(output).toBe(expected);
          }
        });
      } else if (config.typeCheckError) {
        it(testcaseName, () => {
          expect(() => compile(input)).toThrow(TypeCheckerError);
        });
      } else {
        const expected = readFileOrNull(path.resolve(testcaseDir, "output.js"));
        it(testcaseName, () => {
          const output = compile(input);
          if ((updateSnapshot === "new" && expected === null) || updateSnapshot === "all") {
            if (output !== expected) {
              fs.writeFileSync(path.resolve(testcaseDir, "output.js"), output, "utf8");
              console.log(`Updated snapshot for ${testcaseName}`);
            }
          } else {
            if (expected === null) console.error(`Missing snapshot for ${testcaseName}`);
            expect(expected).not.toBeNull();
            expect(output).toBe(expected);
          }
        });
      }
    }
  });
});
