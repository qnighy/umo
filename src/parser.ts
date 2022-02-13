
export type Expression = AddExpression | NumberExpression;
export type AddExpression = {
  type: "AddExpression",
  lhs: Expression,
  rhs: Expression,
}
export type NumberExpression = {
  type: "NumberExpression",
  value: number,
}

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
    if (/\d+/.test(this.tokens[this.pos])) {
      return { type: "NumberExpression", value: parseInt(this.tokens[this.pos++]) };
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
    const c = text[i++];
    if (/\s/.test(c)) continue;
    tokens.push(c);
  }
  return tokens;
}
