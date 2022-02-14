
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

    if (Error.captureStackTrace) {
      Error.captureStackTrace(this, this.constructor);
    }

    this.name = this.constructor.name;
  }
}

export function parse(text: string): Expression {
  const tokens = tokenize(text);
  return new Parser(tokens).parseFullExpression();
}

class Parser {
  private pos = 0;
  constructor(public readonly tokens: readonly string[]) {}
  private parsePrimaryExpression(): Expression {
    if (this.pos >= this.tokens.length) {
      throw new ParseError("Unexpected EOF (expected Expression)");
    }
    if (/^\d+$/.test(this.tokens[this.pos])) {
      return { type: "IntegerLiteral", value: BigInt(this.tokens[this.pos++]) };
    } else if (/^\d+\.\d+$/.test(this.tokens[this.pos])) {
      return { type: "FloatingPointLiteral", value: Number(this.tokens[this.pos++]) };
    } else if (/^[a-zA-Z_][a-zA-Z0-9_]*$/.test(this.tokens[this.pos])) {
      return { type: "VariableReference", name: this.tokens[this.pos++] };
    } else {
      throw new ParseError(`Unexpected token: ${this.tokens[this.pos]} (expected Expression)`);
    }
  }
  private parseExpression(): Expression {
    if (this.pos >= this.tokens.length) {
      throw new ParseError("Unexpected EOF");
    }
    let expr = this.parsePrimaryExpression();
    while (this.pos < this.tokens.length) {
      const token = this.tokens[this.pos];
      if (token === "+") {
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
  public parseFullExpression(): Expression {
    const expr = this.parseExpression();
    this.parseEOF();
    return expr;
  }
}

export function tokenize(text: string): string[] {
  const tokens: string[] = [];
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
      }
      tokens.push(text.slice(start, i));
      continue;
    } else if (/[a-zA-Z_]/.test(c)) {
      while (i < text.length && /[a-zA-Z_0-9]/.test(text[i])) i++;
      tokens.push(text.slice(start, i));
      continue;
    }
    tokens.push(c);
  }
  return tokens;
}
