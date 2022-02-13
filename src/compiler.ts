import { parse } from "./parser";

// Not implemented yet
export function compile(text: string): string {
  const ast = parse(text);
  return text;
}
