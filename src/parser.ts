export type Statement =
  | ExpressionStatement
  | LetStatement;
export type ExpressionStatement = {
  type: "ExpressionStatement",
  expression: Expression,
};
export type LetStatement = {
  type: "LetStatement",
  lhs: string,
  rhs: Expression,
};

export type Expression =
  | VariableReference
  | IntegerLiteral
  | FloatingPointLiteral
  | AddExpression;
export type VariableReference = {
  type: "VariableReference",
  name: string,
};
export type AddExpression = {
  type: "AddExpression",
  lhs: Expression,
  rhs: Expression,
};
export type IntegerLiteral = {
  type: "IntegerLiteral",
  value: bigint,
};
export type FloatingPointLiteral = {
  type: "FloatingPointLiteral",
  value: number,
};

export class ParseError extends Error {
  constructor(message: string) {
    super(message);

    if ((Error as any).captureStackTrace) {
      (Error as any).captureStackTrace(this, this.constructor);
    }

    this.name = this.constructor.name;
  }
}

export function parseStatements(text: string): Statement[] {
  const tokens = tokenize(text);
  return new Parser(tokens).parseFullStatements();
}

export function parseExpression(text: string): Expression {
  const tokens = tokenize(text);
  return new Parser(tokens).parseFullExpression();
}

const KEYWORDS = ["let"];

class Parser {
  private pos = 0;
  constructor(public readonly tokens: readonly Token[]) {}
  private parsePrimaryExpression(): Expression {
    if (this.pos >= this.tokens.length) {
      throw new ParseError("Unexpected EOF (expected Expression)");
    }
    const token = this.tokens[this.pos++];
    if (token.type === "IntegerLiteralToken") {
      return { type: "IntegerLiteral", value: token.value };
    } else if (token.type === "FloatingPointLiteralToken") {
      return { type: "FloatingPointLiteral", value: token.value };
    } else if (token.type === "IdentifierToken") {
      return { type: "VariableReference", name: token.name };
    } else {
      throw new ParseError(`Unexpected token: ${tokenName(token)} (expected Expression)`);
    }
  }
  private parseStatements(): Statement[] {
    const stmts: Statement[] = [];
    while(this.pos < this.tokens.length) {
      stmts.push(this.parseStatement());
    }
    return stmts;
  }
  private parseStatement(): Statement {
    if (this.pos >= this.tokens.length) {
      throw new ParseError("Unexpected EOF");
    }
    if (isSymbolicToken(this.tokens[this.pos], ["let"])) {
      return this.parseLetStatement();
    }
    return this.parseExpressionStatement();
  }
  private parseExpressionStatement(): ExpressionStatement {
    const expr = this.parseExpression();
    if (this.pos >= this.tokens.length) {
      throw new ParseError("Unexpected EOF");
    } else if (!isSymbolicToken(this.tokens[this.pos], [";"])) {
      throw new ParseError(`Unexpected token: ${tokenName(this.tokens[this.pos])} (expected ;)`);
    }
    this.pos++;
    return { type: "ExpressionStatement", expression: expr };
  }
  private parseLetStatement(): LetStatement {
    if (this.pos >= this.tokens.length) {
      throw new ParseError("Unexpected EOF (expected let)");
    } else if (!isSymbolicToken(this.tokens[this.pos], ["let"])) {
      throw new ParseError(`Unexpected token: ${tokenName(this.tokens[this.pos])} (expected let)`);
    }
    this.pos++;

    if (this.pos >= this.tokens.length) {
      throw new ParseError("Unexpected EOF (expected identifier)");
    } else if (this.tokens[this.pos].type !== "IdentifierToken") {
      throw new ParseError(`Unexpected token: ${tokenName(this.tokens[this.pos])} (expected identifier)`);
    }
    const lhs = (this.tokens[this.pos] as IdentifierToken).name;
    this.pos++;

    if (this.pos >= this.tokens.length) {
      throw new ParseError("Unexpected EOF (expected =)");
    } else if (!isSymbolicToken(this.tokens[this.pos], ["="])) {
      throw new ParseError(`Unexpected token: ${tokenName(this.tokens[this.pos])} (expected =)`);
    }
    this.pos++;

    const rhs = this.parseExpression();

    if (this.pos >= this.tokens.length) {
      throw new ParseError("Unexpected EOF");
    } else if (!isSymbolicToken(this.tokens[this.pos], [";"])) {
      throw new ParseError(`Unexpected token: ${tokenName(this.tokens[this.pos])} (expected ;)`);
    }
    this.pos++;

    return { type: "LetStatement", lhs, rhs };
  }
  private parseExpression(): Expression {
    if (this.pos >= this.tokens.length) {
      throw new ParseError("Unexpected EOF");
    }
    let expr = this.parsePrimaryExpression();
    while (this.pos < this.tokens.length) {
      const token = this.tokens[this.pos];
      if (isSymbolicToken(token, ["+"])) {
        this.pos++;
        expr = { type: "AddExpression", lhs: expr, rhs: this.parsePrimaryExpression() };
      } else {
        break;
      }
    }
    return expr;
  }
  private parseEOF() {
    if (this.pos < this.tokens.length) {
      throw new ParseError(`Unexpected token: ${this.tokens[this.pos]} (expected EOF)`);
    }
  }
  public parseFullStatements(): Statement[] {
    const stmts = this.parseStatements();
    this.parseEOF();
    return stmts;
  }
  public parseFullExpression(): Expression {
    const expr = this.parseExpression();
    this.parseEOF();
    return expr;
  }
}

export type Token = IdentifierToken | IntegerLiteralToken | FloatingPointLiteralToken | SymbolicToken;
export type IdentifierToken = {
  type: "IdentifierToken";
  name: string;
};
export type IntegerLiteralToken = {
  type: "IntegerLiteralToken";
  value: bigint;
};
export type FloatingPointLiteralToken = {
  type: "FloatingPointLiteralToken";
  value: number;
};
export type SymbolicToken = {
  type: "SymbolicToken";
  value: string;
};

export function tokenize(text: string): Token[] {
  const tokens: Token[] = [];
  let i = 0;
  while (i < text.length) {
    const start = i;
    const c = text[i++];
    if (/\s/.test(c)) continue;
    if (/\d/.test(c)) {
      while (i < text.length && /\d/.test(text[i])) i++;
      if (i + 1 < text.length && text[i] === "." && /\d/.test(text[i + 1])) {
        i++;
        while (i < text.length && /\d/.test(text[i])) i++;
        tokens.push({ type: "FloatingPointLiteralToken", value: Number(text.substring(start, i)) });
        continue;
      }
      tokens.push({ type: "IntegerLiteralToken", value: BigInt(text.substring(start, i)) });
      continue;
    } else if (/[a-zA-Z_]/.test(c)) {
      while (i < text.length && /[a-zA-Z_0-9]/.test(text[i])) i++;
      const name = text.substring(start, i);
      if (KEYWORDS.includes(name)) {
        tokens.push({ type: "SymbolicToken", value: name });
      } else {
        tokens.push({ type: "IdentifierToken", name });
      }
      continue;
    }
    tokens.push({ type: "SymbolicToken", value: c });
  }
  return tokens;
}

export function isSymbolicToken(token: Token, symbols: readonly string[]): token is SymbolicToken {
  return token.type === "SymbolicToken" && symbols.includes(token.value);
}

export function tokenName(token: Token): string {
  switch (token.type) {
    case "IdentifierToken":
      return `identifier ${token.name}`;
    case "IntegerLiteralToken":
      return `integer literal ${token.value.toString()}`;
    case "FloatingPointLiteralToken":
      return `float literal ${token.value.toString()}`;
    case "SymbolicToken":
      return token.value;
  }
}
