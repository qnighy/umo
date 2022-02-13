import { Expression, parse } from "./parser";

// Not implemented yet
export function compile(text: string): string {
  const ast = parse(text);
  return toJSExpression(ast);
}

function toJSExpression(node: Expression): string {
  switch (node.type) {
    case "NumberExpression":
      return `${node.value}n`;
    case "AddExpression":
      return `(${toJSExpression(node.lhs)} + ${toJSExpression(node.rhs)})`;
  }
}
