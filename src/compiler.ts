import { Expression, parseStatements, Statement } from "./parser";
import { typecheck } from "./typeck";

// Not implemented yet
export function compile(text: string): string {
  const ast = parseStatements(text);
  typecheck(ast);
  return toJSStatements(ast);
}

function toJSStatements(statements: Statement[]): string {
  return statements.map(toJSStatement).join("");
}

function toJSStatement(stmt: Statement): string {
  switch (stmt.type) {
    case "ExpressionStatement":
      return `${toJSExpression(stmt.expression)};\n`;
    case "LetStatement":
      return `const ${stmt.lhs} = ${toJSExpression(stmt.rhs)};\n`;
  }
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
