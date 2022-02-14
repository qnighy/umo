import { Expression, parse } from "./parser";
import { typecheck } from "./typeck";

// Not implemented yet
export function compile(text: string): string {
  const ast = parse(text);
  typecheck(ast);
  return toJSExpression(ast);
}

function toJSExpression(node: Expression): string {
  switch (node.type) {
    case "IntegerLiteral":
      return `${node.value}n`;
    case "FloatingPointLiteral":
      return `${node.value}`;
    case "VariableReference":
      return `${node.name}`;
    case "AddExpression":
      return `(${toJSExpression(node.lhs)} + ${toJSExpression(node.rhs)})`;
  }
}
