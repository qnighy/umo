import path from "path";
import fs from "fs";
import { describe, expect, it, it as realIt, xit } from "@jest/globals";
import { compile } from "./compiler";
import { ParseError, parseStatements } from "./parser";
import { typecheck, TypeCheckerError } from "./typeck";

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
    expect(() => compile("1 +")).toThrow(/Unexpected token: EOF/);
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

          const output = error instanceof ParseError ? error.toFullMessageWithCodeFrame(input) : "";
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
          const ast = parseStatements(input);
          const expected = readFileOrNull(path.resolve(testcaseDir, "error.txt"));
          let error: unknown = undefined;
          try {
            typecheck(ast);
          } catch(e) {
            error = e;
          }
          expect(error).toBeInstanceOf(TypeCheckerError);

          const output = error instanceof TypeCheckerError ? error.toFullMessageWithCodeFrame(input) : "";
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
      } else {
        const expected = readFileOrNull(path.resolve(testcaseDir, "output.js"));
        it(testcaseName, () => {
          const output = compile(input);
          if (!config.compileOnly) {
            evalWith(output, {
              assert_eq(a: any, b: any) {
                if (a !== b) throw new Error(`${a} !== ${b}`);
              }
            });
          }
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

function evalWith(code: string, locals: Record<string, any>): any {
  const localKeys = Object.keys(locals);
  const f = new Function(...localKeys, code);
  return f.apply(null, localKeys.map(k => locals[k]));
}
