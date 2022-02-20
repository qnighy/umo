export type Position = {
  /**
   * 0-based line number.
   */
  line: number;
  /**
   * 0-based column number in UTF-16 code units.
   */
  column: number;
};

export type Statement =
  | ExpressionStatement
  | LetStatement;
/**
 * Expression statement `1 + 1;`.
 */
export type ExpressionStatement = {
  type: "ExpressionStatement",
  expression: Expression,
};
/**
 * Let statement `let x = 1 + 1;`.
 */
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
/**
 * `x` as in `x + 1`.
 */
export type VariableReference = {
  type: "VariableReference",
  name: string,
};
/**
 * Binary addition `1 + 1`.
 */
export type AddExpression = {
  type: "AddExpression",
  lhs: Expression,
  rhs: Expression,
};
/**
 * Integer literals like `123`. No negative numbers.
 * Currently decimals only.
 */
export type IntegerLiteral = {
  type: "IntegerLiteral",
  value: bigint,
};
/**
 * Floating point literals like `1.23`. No negative numbers.
 * Currently decimals only and no exponent notation.
 */
export type FloatingPointLiteral = {
  type: "FloatingPointLiteral",
  value: number,
};

export class ParseError extends Error {
  originalMessage: string;
  start: Position;
  end: Position;
  constructor(options: { start: Position, end: Position, message: string }) {
    const { start, end, message } = options;
    super(`${start.line + 1}:${start.column + 1}: ${message}`);
    this.originalMessage = message;
    this.start = start;
    this.end = end;

    if ((Error as any).captureStackTrace) {
      (Error as any).captureStackTrace(this, this.constructor);
    }

    this.name = this.constructor.name;
  }

  public toMessageWithCodeFrame(source: string): string {
    const lines = source.split(/\r\n?|\n/);
    const frameStart = Math.max(0, this.start.line - 2);
    const frameEnd = Math.min(lines.length, this.end.line + 2 + 1);
    const lineNumberWidth = Math.max(`${frameStart + 1}`.length, `${frameEnd}`.length);

    let message = `${this.message}\n\n`;
    for (let lineno = frameStart; lineno < frameEnd; lineno++) {
      const line = lines[lineno];
      message += `    ${`${lineno + 1}`.padStart(lineNumberWidth)} | ${line}\n`;
      if (this.start.line <= lineno && lineno <= this.end.line) {
        const startColumn = this.start.line === lineno ? this.start.column : 0;
        const endColumn = Math.max(this.end.line === lineno ? this.end.column : line.length, startColumn + 1);
        // TODO: take East Asian Width into account
        message += `    ${" ".repeat(lineNumberWidth)} | ${" ".repeat(startColumn)}${"^".repeat(endColumn - startColumn)}\n`;
      }
    }
    return message;
  }
}

/**
 * Parses the text as a sequence of statements.
 */
export function parseStatements(text: string): Statement[] {
  const tokens = tokenize(text);
  return new Parser(tokens).parseFullStatements();
}

/**
 * Parses the text as a single expression.
 */
export function parseExpression(text: string): Expression {
  const tokens = tokenize(text);
  return new Parser(tokens).parseFullExpression();
}

const KEYWORDS = ["let"];

class Parser {
  private pos = 0;
  constructor(public readonly tokens: readonly Token[]) {}
  private parsePrimaryExpression(): Expression {
    const token = this.tokens[this.pos++];
    if (token.type === "IntegerLiteralToken") {
      return { type: "IntegerLiteral", value: token.value };
    } else if (token.type === "FloatingPointLiteralToken") {
      return { type: "FloatingPointLiteral", value: token.value };
    } else if (token.type === "IdentifierToken") {
      return { type: "VariableReference", name: token.name };
    } else {
      throw new ParseError({
        start: token.start,
        end: token.end,
        message: `Unexpected token: ${tokenName(token)} (expected Expression)`,
      });
    }
  }
  private parseStatements(): Statement[] {
    const stmts: Statement[] = [];
    while(this.tokens[this.pos].type !== "EOFToken") {
      stmts.push(this.parseStatement());
    }
    return stmts;
  }
  private parseStatement(): Statement {
    if (isSymbolicToken(this.tokens[this.pos], ["let"])) {
      return this.parseLetStatement();
    }
    return this.parseExpressionStatement();
  }
  private parseExpressionStatement(): ExpressionStatement {
    const expr = this.parseExpression();
    if (!isSymbolicToken(this.tokens[this.pos], [";"])) {
      throw new ParseError({
        start: this.tokens[this.pos].start,
        end: this.tokens[this.pos].end,
        message: `Unexpected token: ${tokenName(this.tokens[this.pos])} (expected ;)`,
      });
    }
    this.pos++;
    return { type: "ExpressionStatement", expression: expr };
  }
  private parseLetStatement(): LetStatement {
    if (!isSymbolicToken(this.tokens[this.pos], ["let"])) {
      throw new ParseError({
        start: this.tokens[this.pos].start,
        end: this.tokens[this.pos].end,
        message: `Unexpected token: ${tokenName(this.tokens[this.pos])} (expected let)`,
      });
    }
    this.pos++;

    if (this.tokens[this.pos].type !== "IdentifierToken") {
      throw new ParseError({
        start: this.tokens[this.pos].start,
        end: this.tokens[this.pos].end,
        message: `Unexpected token: ${tokenName(this.tokens[this.pos])} (expected identifier)`,
      });
    }
    const lhs = (this.tokens[this.pos] as IdentifierToken).name;
    this.pos++;

    if (!isSymbolicToken(this.tokens[this.pos], ["="])) {
      throw new ParseError({
        start: this.tokens[this.pos].start,
        end: this.tokens[this.pos].end,
        message: `Unexpected token: ${tokenName(this.tokens[this.pos])} (expected =)`,
      });
    }
    this.pos++;

    const rhs = this.parseExpression();

    if (!isSymbolicToken(this.tokens[this.pos], [";"])) {
      throw new ParseError({
        start: this.tokens[this.pos].start,
        end: this.tokens[this.pos].end,
        message: `Unexpected token: ${tokenName(this.tokens[this.pos])} (expected ;)`,
      });
    }
    this.pos++;

    return { type: "LetStatement", lhs, rhs };
  }
  private parseExpression(): Expression {
    let expr = this.parsePrimaryExpression();
    while (true) {
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
    const token = this.tokens[this.pos];
    if (token.type !== "EOFToken") {
      throw new ParseError({
        start: token.start,
        end: token.end,
        message: `Unexpected token: ${this.tokens[this.pos]} (expected EOF)`,
      });
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

export type Token = IdentifierToken | IntegerLiteralToken | FloatingPointLiteralToken | SymbolicToken | EOFToken;
export type IdentifierToken = {
  type: "IdentifierToken";
  name: string;
  start: Position;
  end: Position;
};
export type IntegerLiteralToken = {
  type: "IntegerLiteralToken";
  value: bigint;
  start: Position;
  end: Position;
};
export type FloatingPointLiteralToken = {
  type: "FloatingPointLiteralToken";
  value: number;
  start: Position;
  end: Position;
};
/**
 * All tokens that don't carry additional data, like `+`, `;`, and `let`.
 */
export type SymbolicToken = {
  type: "SymbolicToken";
  value: string;
  start: Position;
  end: Position;
};
/**
 * Virtual sentinel token to simplify the parser.
 */
export type EOFToken = {
  type: "EOFToken";
  start: Position;
  end: Position;
};

export function tokenize(text: string): Token[] {
  const tokens: Token[] = [];
  let i = 0;
  let line = 0;
  let lineStart = 0;
  while (i < text.length) {
    const start = i;
    const c = text[i++];
    if (c === "\n" || c === "\r") {
      if (c === "\r" && i < text.length && text[i] === "\n") {
        i++;
      }
      line++;
      lineStart = i;
      continue;
    }
    if (/\s/.test(c)) continue;
    const startLoc = { line, column: start - lineStart };
    if (/\d/.test(c)) {
      while (i < text.length && /\d/.test(text[i])) i++;
      if (i + 1 < text.length && text[i] === "." && /\d/.test(text[i + 1])) {
        i++;
        while (i < text.length && /\d/.test(text[i])) i++;
        tokens.push({
          type: "FloatingPointLiteralToken",
          value: Number(text.substring(start, i)),
          start: startLoc,
          end: { line, column: i - lineStart },
        });
        continue;
      }
      tokens.push({
        type: "IntegerLiteralToken",
        value: BigInt(text.substring(start, i)),
        start: startLoc,
        end: { line, column: i - lineStart },
      });
      continue;
    } else if (/[a-zA-Z_]/.test(c)) {
      while (i < text.length && /[a-zA-Z_0-9]/.test(text[i])) i++;
      const name = text.substring(start, i);
      if (KEYWORDS.includes(name)) {
        tokens.push({
          type: "SymbolicToken",
          value: name,
          start: startLoc,
          end: { line, column: i - lineStart },
        });
      } else {
        tokens.push({
          type: "IdentifierToken",
          name,
          start: startLoc,
          end: { line, column: i - lineStart },
        });
      }
      continue;
    }
    tokens.push({
      type: "SymbolicToken",
      value: c,
      start: startLoc,
      end: { line, column: i - lineStart },
    });
  }
  tokens.push({
    type: "EOFToken",
    start: { line, column: i - lineStart },
    end: { line, column: i - lineStart },
  });
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
    case "EOFToken":
      return "EOF";
  }
}
