import { describe, expect, it, xit } from "@jest/globals";
import { compile } from "./compiler";

describe("compile", () => {
  it("returns a string", () => {
    expect(typeof compile("1 + 1")).toBe("string");
  });

  xit("Compiles a text to JS", () => {
    expect(compile("1 + 1")).toBe("1n + 1n");
  });
});
