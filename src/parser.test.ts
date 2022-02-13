import { describe, expect, it, xit } from "@jest/globals";
import { tokenize } from "./parser";

describe("tokenize", () => {
  it("tokenizes a text", () => {
    expect(tokenize("1 + 1")).toEqual(["1", "+", "1"]);
  });

  xit("tokenizes a number", () => {
    expect(tokenize("123 + 456")).toEqual(["123", "+", "456"]);
  });
});
