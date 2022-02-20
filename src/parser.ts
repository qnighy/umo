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
  | LetStatement
  | ParseErroredStatement;
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
/**
 * Placeholder node for when we encountered a parse error.
 */
export type ParseErroredStatement = {
  type: "ParseErroredStatement",
};

export type Expression =
  | VariableReference
  | IntegerLiteral
  | FloatingPointLiteral
  | AddExpression
  | ParseErroredExpression;
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
/**
 * Placeholder node for when we encountered a parse error.
 */
export type ParseErroredExpression = {
  type: "ParseErroredExpression",
};

export class ParseError extends Error {
  public subErrors: readonly SingleParseError[];
  constructor(subErrors: readonly SingleParseError[]) {
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

export class SingleParseError extends Error {
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
  const { statements, error } = parseStatementsWithErrors(text);
  if (error) throw error;
  return statements;
}

/**
 * Parses the text as a sequence of statements.
 * This function returns a partial AST even if there are parse errors.
 */
export function parseStatementsWithErrors(text: string): { statements: Statement[], error: ParseError | undefined } {
  const tokens = tokenize(text);
  const parser = new Parser(tokens);
  const statements = parser.parseFullStatements();
  const error = parser.getError();
  return { statements, error };
}

/**
 * Parses the text as a single expression.
 */
export function parseExpression(text: string): Expression {
  const { expression, error } = parseExpressionWithErrors(text);
  if (error) throw error;
  return expression;
}

/**
 * Parses the text as a single expression.
 * This function returns a partial AST even if there are parse errors.
 */
export function parseExpressionWithErrors(text: string): { expression: Expression, error: ParseError | undefined } {
  const tokens = tokenize(text);
  const parser = new Parser(tokens);
  const expression = parser.parseFullExpression();
  const error = parser.getError();
  return { expression, error };
}

const KEYWORDS = ["let"];

class Parser {
  private pos = 0;
  private errors: SingleParseError[] = [];
  constructor(public readonly tokens: readonly Token[]) {}
  private parsePrimaryExpression(): Expression {
    const token = this.tokens[this.pos];
    if (token.type === "IntegerLiteralToken") {
      this.pos++;
      return { type: "IntegerLiteral", value: token.value };
    } else if (token.type === "FloatingPointLiteralToken") {
      this.pos++;
      return { type: "FloatingPointLiteral", value: token.value };
    } else if (token.type === "IdentifierToken") {
      this.pos++;
      return { type: "VariableReference", name: token.name };
    } else {
      this.errors.push(new SingleParseError({
        start: token.start,
        end: token.end,
        message: `Unexpected token: ${tokenName(token)} (expected Expression)`,
      }));
      // TODO: better fallback strategy
      if (token.type !== "EOFToken" && !isSymbolicToken(token, [",", ";", ")", "}", "]"])) this.pos++;
      return { type: "ParseErroredExpression" };
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
    const initPos = this.pos;
    const expr = this.parseExpression();
    if (!isSymbolicToken(this.tokens[this.pos], [";"])) {
      this.errors.push(new SingleParseError({
        start: this.tokens[this.pos].start,
        end: this.tokens[this.pos].end,
        message: `Unexpected token: ${tokenName(this.tokens[this.pos])} (expected ;)`,
      }));
      if (initPos === this.pos && this.tokens[this.pos].type !== "EOFToken") this.pos++;
      return { type: "ExpressionStatement", expression: expr };
    }
    this.pos++;
    return { type: "ExpressionStatement", expression: expr };
  }
  private parseLetStatement(): LetStatement {
    let hasError = false;
    if (!isSymbolicToken(this.tokens[this.pos], ["let"])) {
      this.errors.push(new SingleParseError({
        start: this.tokens[this.pos].start,
        end: this.tokens[this.pos].end,
        message: `Unexpected token: ${tokenName(this.tokens[this.pos])} (expected let)`,
      }));
      if (this.tokens[this.pos].type !== "EOFToken") this.pos++;
      return { type: "LetStatement", lhs: "", rhs: { type: "ParseErroredExpression" } };
    }
    this.pos++;

    let lhs: string;
    if (this.tokens[this.pos].type === "IdentifierToken") {
      lhs = (this.tokens[this.pos] as IdentifierToken).name;
      this.pos++;
    } else {
      this.errors.push(new SingleParseError({
        start: this.tokens[this.pos].start,
        end: this.tokens[this.pos].end,
        message: `Unexpected token: ${tokenName(this.tokens[this.pos])} (expected identifier)`,
      }));
      lhs = "";
      hasError = true;
    }

    if (isSymbolicToken(this.tokens[this.pos], ["="])) {
      this.pos++;
    } else if (!hasError) {
      this.errors.push(new SingleParseError({
        start: this.tokens[this.pos].start,
        end: this.tokens[this.pos].end,
        message: `Unexpected token: ${tokenName(this.tokens[this.pos])} (expected =)`,
      }));
      hasError = true;
    }

    const rhs = this.parseExpression();

    if (isSymbolicToken(this.tokens[this.pos], [";"])) {
      this.pos++;
    } else {
      if (!hasError) {
        this.errors.push(new SingleParseError({
          start: this.tokens[this.pos].start,
          end: this.tokens[this.pos].end,
          message: `Unexpected token: ${tokenName(this.tokens[this.pos])} (expected ;)`,
        }));
        hasError = true;
      }
      // Recover towards a statement boundary or something of the sort.
      while (this.tokens[this.pos].type !== "EOFToken" && !isSymbolicToken(this.tokens[this.pos], [";", "}", ")", "]"])) {
        this.pos++;
      }
      if (this.tokens[this.pos].type !== "EOFToken") this.pos++;
    }

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
      this.errors.push(new SingleParseError({
        start: token.start,
        end: token.end,
        message: `Unexpected token: ${this.tokens[this.pos]} (expected EOF)`,
      }));
      this.pos++;
      return;
    }
  }
  public getError(): ParseError | undefined {
    return this.errors.length > 0 ? new ParseError(this.errors) : undefined;
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
