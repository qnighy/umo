import { Expression, Statement } from "./parser";

export type Type = BuiltinType | AmbiguousType;
export type BuiltinType = {
  type: "BuiltinType";
  kind: "int" | "f64";
};
export type AmbiguousType = {
  type: "AmbiguousType";
};

export class TypeCheckerError extends Error {
  constructor(message: string) {
    super(message);

    if (Error.captureStackTrace) {
      Error.captureStackTrace(this, this.constructor);
    }

    this.name = this.constructor.name;
  }
}

export function typecheck(ast: Statement[]) {
  const variableTypes = new Map<string, Type>();
  for (const stmt of ast) {
    checkStatement(variableTypes, stmt);
  }
}

function checkStatement(variableTypes: Map<string, Type>, stmt: Statement) {
  switch (stmt.type) {
    case "ExpressionStatement":
      getType(variableTypes, stmt.expression);
      break;
    case "LetStatement":{
      const rhsType = getType(variableTypes, stmt.rhs);
      variableTypes.set(stmt.lhs, rhsType);
      break;
    }
  }
}

function getType(variableTypes: Map<string, Type>, ast: Expression): Type {
  switch (ast.type) {
    case "IntegerLiteral":
      return { type: "BuiltinType", kind: "int" };
    case "FloatingPointLiteral":
      return { type: "BuiltinType", kind: "f64" };
    case "VariableReference": {
      const type = variableTypes.get(ast.name);
      if (type) {
        return type;
      } else {
        return { type: "AmbiguousType" }; // TODO
      }
    }
    case "AddExpression": {
      const lhsType = getType(variableTypes, ast.lhs);
      const rhsType = getType(variableTypes, ast.rhs);
      if (lhsType.type === "AmbiguousType" || rhsType.type === "AmbiguousType") {
        return { type: "AmbiguousType" };
      } else if (lhsType.type === "BuiltinType" && lhsType.kind === "int" && rhsType.type === "BuiltinType" && rhsType.kind === "int") {
        return { type: "BuiltinType", kind: "int" };
      } else if (lhsType.type === "BuiltinType" && lhsType.kind === "f64" && rhsType.type === "BuiltinType" && rhsType.kind === "f64") {
        return { type: "BuiltinType", kind: "f64" };
      } else {
        // TODO: more useful error message
        throw new TypeCheckerError("Invalid types in addition");
      }
    }
  }
}
