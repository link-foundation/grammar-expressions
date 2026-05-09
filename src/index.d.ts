export interface Expression {
  kind: string;
}

export interface MatchResult {
  start: number;
  end: number;
  text: string;
  captures: Record<string, string>;
}

export class GrammarSyntaxError extends Error {
  readonly position: number;
}

export function parse(pattern: string): Expression;

export function isMatch(pattern: string, input: string): boolean;

export function matchFull(pattern: string, input: string): MatchResult | null;

export function findFirst(pattern: string, input: string): MatchResult | null;

export function findAll(pattern: string, input: string): MatchResult[];

export function replaceAll(pattern: string, replacement: string, input: string): string;

export function matchFullExpression(expression: Expression, input: string): MatchResult | null;

export function findAllExpression(expression: Expression, input: string): MatchResult[];

export function replaceAllExpression(
  expression: Expression,
  replacement: string,
  input: string,
): string;
