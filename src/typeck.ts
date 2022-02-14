import { Expression } from "./parser";

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

export function typecheck(ast: Expression) {
  getType(ast);
}

function getType(ast: Expression): Type {
  switch (ast.type) {
    case "IntegerLiteral":
      return { type: "BuiltinType", kind: "int" };
    case "FloatingPointLiteral":
      return { type: "BuiltinType", kind: "f64" };
    case "VariableReference":
      return { type: "AmbiguousType" }; // TODO
    case "AddExpression": {
      const lhsType = getType(ast.lhs);
      const rhsType = getType(ast.rhs);
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
