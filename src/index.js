export class GrammarSyntaxError extends Error {
  constructor(message, position) {
    super(`${message} at pattern position ${position}`);
    this.name = 'GrammarSyntaxError';
    this.position = position;
  }
}

export function parse(pattern) {
  return new Parser(pattern).parse();
}

export function isMatch(pattern, input) {
  return matchFullExpression(parse(pattern), input) !== null;
}

export function matchFull(pattern, input) {
  return matchFullExpression(parse(pattern), input);
}

export function findFirst(pattern, input) {
  return findFrom(parse(pattern), input, 0);
}

export function findAll(pattern, input) {
  return findAllExpression(parse(pattern), input);
}

export function replaceAll(pattern, replacement, input) {
  return replaceAllExpression(parse(pattern), replacement, input);
}

export function matchFullExpression(expression, input) {
  const states = matchExpression(expression, input, { pos: 0, captures: {} });
  const found = states.find((state) => state.pos === input.length);

  if (!found) {
    return null;
  }

  return {
    start: 0,
    end: input.length,
    text: input,
    captures: found.captures,
  };
}

export function findAllExpression(expression, input) {
  const results = [];
  let cursor = 0;

  while (cursor <= input.length) {
    const found = findFrom(expression, input, cursor);
    if (!found) {
      break;
    }

    const empty = found.start === found.end;
    results.push(found);

    if (empty) {
      if (found.end === input.length) {
        break;
      }
      cursor = nextBoundary(input, found.end);
    } else {
      cursor = found.end;
    }
  }

  return results;
}

export function replaceAllExpression(expression, replacement, input) {
  let output = '';
  let copiedUntil = 0;
  let searchStart = 0;

  while (searchStart <= input.length) {
    const found = findFrom(expression, input, searchStart);
    if (!found) {
      break;
    }

    output += input.slice(copiedUntil, found.start);
    output += renderReplacement(replacement, found);
    copiedUntil = found.end;

    if (found.start === found.end) {
      searchStart = found.end === input.length ? input.length + 1 : nextBoundary(input, found.end);
    } else {
      searchStart = found.end;
    }
  }

  return output + input.slice(copiedUntil);
}

function findFrom(expression, input, minStart) {
  for (const start of boundaryIndices(input)) {
    if (start < minStart) {
      continue;
    }

    const states = matchExpression(expression, input, { pos: start, captures: {} });
    const found = states[0];
    if (found) {
      return {
        start,
        end: found.pos,
        text: input.slice(start, found.pos),
        captures: found.captures,
      };
    }
  }

  return null;
}

function matchExpression(expression, input, state) {
  switch (expression.kind) {
    case 'empty':
      return [state];
    case 'literal': {
      const actual = nextChar(input, state.pos);
      return actual && actual.value === expression.value
        ? [{ ...state, pos: actual.next }]
        : [];
    }
    case 'any': {
      const actual = nextChar(input, state.pos);
      return actual ? [{ ...state, pos: actual.next }] : [];
    }
    case 'class': {
      const actual = nextChar(input, state.pos);
      return actual && classContains(expression, actual.value)
        ? [{ ...state, pos: actual.next }]
        : [];
    }
    case 'sequence': {
      let states = [state];

      for (const item of expression.expressions) {
        const nextStates = [];
        for (const current of states) {
          nextStates.push(...matchExpression(item, input, current));
        }

        if (nextStates.length === 0) {
          return [];
        }

        states = nextStates;
      }

      return states;
    }
    case 'choice': {
      for (const item of expression.expressions) {
        const states = matchExpression(item, input, cloneState(state));
        if (states.length > 0) {
          return states;
        }
      }

      return [];
    }
    case 'repeat': {
      const states = [];
      collectRepetition(expression.expression, input, state, 0, expression.min, expression.max, states);
      return states;
    }
    case 'capture': {
      const start = state.pos;
      return matchExpression(expression.expression, input, state).map((next) => ({
        ...next,
        captures: {
          ...next.captures,
          [expression.name]: input.slice(start, next.pos),
        },
      }));
    }
    default:
      throw new Error(`Unknown expression kind: ${expression.kind}`);
  }
}

function collectRepetition(expression, input, state, count, min, max, states) {
  if (max === null || count < max) {
    for (const next of matchExpression(expression, input, cloneState(state))) {
      if (next.pos !== state.pos) {
        collectRepetition(expression, input, next, count + 1, min, max, states);
      }
    }
  }

  if (count >= min) {
    states.push(state);
  }
}

function classContains(expression, value) {
  const codePoint = value.codePointAt(0);
  const matched = expression.ranges.some(
    ([start, end]) => start.codePointAt(0) <= codePoint && codePoint <= end.codePointAt(0),
  );

  return matched !== expression.inverted;
}

function renderReplacement(template, found) {
  let output = '';
  const chars = [...template];

  for (let index = 0; index < chars.length; index += 1) {
    const current = chars[index];
    if (current !== '$') {
      output += current;
      continue;
    }

    const next = chars[index + 1];
    if (next === '$') {
      output += '$';
      index += 1;
    } else if (next === '0') {
      output += found.text;
      index += 1;
    } else if (next === '{') {
      let name = '';
      index += 2;

      while (index < chars.length && chars[index] !== '}') {
        name += chars[index];
        index += 1;
      }

      if (index < chars.length) {
        output += found.captures[name] ?? '';
      } else {
        output += `\${${name}`;
      }
    } else if (isIdentifierStart(next)) {
      let name = '';
      index += 1;

      while (index < chars.length && isIdentifierContinue(chars[index])) {
        name += chars[index];
        index += 1;
      }

      index -= 1;
      output += found.captures[name] ?? '';
    } else {
      output += '$';
    }
  }

  return output;
}

function boundaryIndices(input) {
  const indices = [];
  for (let index = 0; index < input.length; ) {
    indices.push(index);
    index = nextBoundary(input, index);
  }
  indices.push(input.length);
  return indices;
}

function nextBoundary(input, position) {
  const current = nextChar(input, position);
  return current ? current.next : input.length;
}

function nextChar(input, position) {
  if (position >= input.length) {
    return null;
  }

  const codePoint = input.codePointAt(position);
  const value = String.fromCodePoint(codePoint);
  return {
    value,
    next: position + value.length,
  };
}

function cloneState(state) {
  return {
    pos: state.pos,
    captures: { ...state.captures },
  };
}

class Parser {
  constructor(pattern) {
    this.chars = [...pattern];
    this.pos = 0;
  }

  parse() {
    const expression = this.parseChoice();
    if (this.peek() !== undefined) {
      throw this.error('unexpected token');
    }
    return expression;
  }

  parseChoice() {
    const alternatives = [this.parseSequence()];

    while (this.peek() === '|') {
      this.next();
      alternatives.push(this.parseSequence());
    }

    return alternatives.length === 1
      ? alternatives[0]
      : { kind: 'choice', expressions: alternatives };
  }

  parseSequence() {
    const expressions = [];

    while (this.peek() !== undefined) {
      const value = this.peek();
      if (value === ')' || value === '}' || value === '|') {
        break;
      }
      expressions.push(this.parseQuantified());
    }

    if (expressions.length === 0) {
      return { kind: 'empty' };
    }

    return expressions.length === 1
      ? expressions[0]
      : { kind: 'sequence', expressions };
  }

  parseQuantified() {
    const expression = this.parseAtom();

    if (this.peek() === '*') {
      this.next();
      return { kind: 'repeat', expression, min: 0, max: null };
    }
    if (this.peek() === '+') {
      this.next();
      return { kind: 'repeat', expression, min: 1, max: null };
    }
    if (this.peek() === '?') {
      this.next();
      return { kind: 'repeat', expression, min: 0, max: 1 };
    }

    return expression;
  }

  parseAtom() {
    const value = this.next();

    if (value === '(') {
      const expression = this.parseChoice();
      this.expect(')');
      return expression;
    }
    if (value === '{') {
      return this.parseCapture();
    }
    if (value === '[') {
      return this.parseClass();
    }
    if (value === '.') {
      return { kind: 'any' };
    }
    if (value === '\\') {
      return { kind: 'literal', value: this.parseEscape() };
    }
    if (value === undefined) {
      throw this.error('expected expression');
    }
    if (['*', '+', '?', ')', '}', '|'].includes(value)) {
      throw this.errorAt(`unexpected token \`${value}\``, this.pos - 1);
    }

    return { kind: 'literal', value };
  }

  parseCapture() {
    const namePosition = this.pos;
    const name = this.parseIdentifier();
    if (!name) {
      throw this.errorAt('expected capture name', namePosition);
    }

    this.expect(':');
    const expression = this.parseChoice();
    this.expect('}');

    return {
      kind: 'capture',
      name,
      expression,
    };
  }

  parseClass() {
    const inverted = this.peek() === '^';
    if (inverted) {
      this.next();
    }

    const ranges = [];

    while (true) {
      if (this.peek() === ']') {
        this.next();
        break;
      }
      if (this.peek() === undefined) {
        throw this.error('unterminated character class');
      }

      const start = this.parseClassChar();
      if (this.peek() === '-' && this.peekOffset(1) !== ']') {
        this.next();
        const end = this.parseClassChar();
        ranges.push([start, end]);
      } else {
        ranges.push([start, start]);
      }
    }

    if (ranges.length === 0) {
      throw this.error('empty character class');
    }

    return {
      kind: 'class',
      ranges,
      inverted,
    };
  }

  parseClassChar() {
    const value = this.next();
    if (value === '\\') {
      return this.parseEscape();
    }
    if (value === undefined) {
      throw this.error('expected character class item');
    }
    return value;
  }

  parseEscape() {
    const value = this.next();
    if (value === 'n') {
      return '\n';
    }
    if (value === 'r') {
      return '\r';
    }
    if (value === 't') {
      return '\t';
    }
    if (value !== undefined) {
      return value;
    }
    throw this.error('unterminated escape sequence');
  }

  parseIdentifier() {
    let name = '';
    if (!isIdentifierStart(this.peek())) {
      return name;
    }

    name += this.next();
    while (isIdentifierContinue(this.peek())) {
      name += this.next();
    }

    return name;
  }

  expect(expected) {
    const actual = this.next();
    if (actual === expected) {
      return;
    }
    if (actual === undefined) {
      throw this.error(`expected \`${expected}\``);
    }
    throw this.errorAt(`expected \`${expected}\` but found \`${actual}\``, this.pos - 1);
  }

  peek() {
    return this.chars[this.pos];
  }

  peekOffset(offset) {
    return this.chars[this.pos + offset];
  }

  next() {
    const value = this.peek();
    if (value !== undefined) {
      this.pos += 1;
    }
    return value;
  }

  error(message) {
    return this.errorAt(message, this.pos);
  }

  errorAt(message, position) {
    return new GrammarSyntaxError(message, position);
  }
}

function isIdentifierStart(value) {
  return value === '_' || /^[A-Za-z]$/.test(value ?? '');
}

function isIdentifierContinue(value) {
  return isIdentifierStart(value) || /^[0-9]$/.test(value ?? '');
}
