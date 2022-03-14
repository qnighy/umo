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
    case "ParseErroredStatement":
      throw new Error("Cannot compile sources with parse error");
  }
}

function toJSExpression(node: Expression, outerPrec: number = 4): string {
  if (node.type === "ParenthesizedExpression") {
    return toJSExpression(node.expression, outerPrec);
  }
  const prec = precedence(node);
  if (prec > outerPrec) {
    return `(${toJSExpression(node)})`;
  }
  switch (node.type) {
    case "IntegerLiteral":
      return `${node.value}n`;
    case "FloatingPointLiteral":
      return `${node.value}`;
    case "VariableReference":
      return `${node.name}`;
    case "CallExpression":
      return `${toJSExpression(node.callee, prec)}(${node.arguments.map((a) => toJSExpression(a)).join(", ")})`;
    case "ClosureExpression":
      return `(${node.parameters.join(", ")}) => ${toJSExpression(node.body, prec)}`;
    case "AddExpression":
      return `${toJSExpression(node.lhs, prec)} + ${toJSExpression(node.rhs, prec - 1)}`;
    case "ParseErroredExpression":
      throw new Error("Cannot compile sources with parse error");
  }
}

function precedence(node: Expression): number {
  switch (node.type) {
    case "ParenthesizedExpression":
    case "IntegerLiteral":
    case "FloatingPointLiteral":
    case "VariableReference":
      return 0;
    case "CallExpression":
      return 1;
    case "AddExpression":
      return 2;
    case "ClosureExpression":
      return 3;
    case "ParseErroredExpression":
      throw new Error("Cannot compile sources with parse error");
  }
}
