import { Expression, Range, renderCodeFrame, Statement } from "./parser";

export type Type = BuiltinType | AmbiguousType;
export type BuiltinType = {
  type: "BuiltinType";
  kind: "int" | "f64";
};
export type AmbiguousType = {
  type: "AmbiguousType";
};

export class TypeCheckerError extends Error {
  public subErrors: readonly SingleTypeCheckerError[];
  constructor(subErrors: readonly SingleTypeCheckerError[]) {
    super(subErrors[0].message);
    this.subErrors = subErrors;

    // TODO: copy stack traces from subErrors[0]
    if ((Error as any).captureStackTrace) {
      (Error as any).captureStackTrace(this, this.constructor);
    }

    this.name = this.constructor.name;
  }

  public toFullMessageWithCodeFrame(source: string): string {
    let messages = "";
    for (const subError of this.subErrors) {
      messages += subError.toMessageWithCodeFrame(source) + "\n";
    }
    return messages;
  }
}

export class SingleTypeCheckerError extends Error {
  originalMessage: string;
  range: Range;
  constructor(options: { range: Range, message: string }) {
    const { range, message } = options;
    super(`${range.start.line + 1}:${range.start.column + 1}: ${message}`);
    this.originalMessage = message;
    this.range = range;

    if ((Error as any).captureStackTrace) {
      (Error as any).captureStackTrace(this, this.constructor);
    }

    this.name = this.constructor.name;
  }

  public toMessageWithCodeFrame(source: string): string {
    return `${this.message}\n\n${renderCodeFrame(source, this.range)}`;
  }
}

export function typecheck(ast: Statement[]) {
  const errors: SingleTypeCheckerError[] = [];
  const variableTypes = new Map<string, Type>();
  for (const stmt of ast) {
    checkStatement(errors, variableTypes, stmt);
  }
  if (errors.length > 0) {
    throw new TypeCheckerError(errors);
  }
}

function checkStatement(errors: SingleTypeCheckerError[], variableTypes: Map<string, Type>, stmt: Statement) {
  switch (stmt.type) {
    case "ExpressionStatement":
      getType(errors, variableTypes, stmt.expression);
      break;
    case "LetStatement":{
      const rhsType = getType(errors, variableTypes, stmt.rhs);
      variableTypes.set(stmt.lhs, rhsType);
      break;
    }
    case "ParseErroredStatement":
      // Do nothing
      break;
  }
}

function getType(errors: SingleTypeCheckerError[], variableTypes: Map<string, Type>, ast: Expression): Type {
  switch (ast.type) {
    case "ParenthesizedExpression":
      return getType(errors, variableTypes, ast.expression);
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
      const lhsType = getType(errors, variableTypes, ast.lhs);
      const rhsType = getType(errors, variableTypes, ast.rhs);
      if (lhsType.type === "AmbiguousType" || rhsType.type === "AmbiguousType") {
        return { type: "AmbiguousType" };
      } else if (lhsType.type === "BuiltinType" && lhsType.kind === "int" && rhsType.type === "BuiltinType" && rhsType.kind === "int") {
        return { type: "BuiltinType", kind: "int" };
      } else if (lhsType.type === "BuiltinType" && lhsType.kind === "f64" && rhsType.type === "BuiltinType" && rhsType.kind === "f64") {
        return { type: "BuiltinType", kind: "f64" };
      } else {
        // TODO: more useful error message
        errors.push(new SingleTypeCheckerError({ message: "Invalid types in addition", range: ast.range }));
        return { type: "AmbiguousType" };
      }
    }
    case "ParseErroredExpression": {
      return { type: "AmbiguousType" };
    }
  }
}
