import assert from 'node:assert/strict';
import { test } from 'node:test';

import { findAll, findFirst, isMatch, replaceAll, rewrite } from '../src/index.js';

test('matches literals, choices, and repetition', () => {
  assert.equal(isMatch('hello', 'hello'), true);
  assert.equal(isMatch('h(i|ello)', 'hello'), true);
  assert.equal(isMatch('ab+c', 'abbbc'), true);
  assert.equal(isMatch('colou?r', 'color'), true);
  assert.equal(isMatch('colou?r', 'colour'), true);
  assert.equal(isMatch('ab+c', 'ac'), false);
});

test('captures named substrings for replacement', () => {
  assert.equal(
    replaceAll('{name:[A-Za-z]+}: {value:[0-9]+}', '$name=$value', 'width: 42 height: 7'),
    'width=42 height=7',
  );
});

test('finds unicode text at character boundaries', () => {
  const first = findFirst('{word:[A-Za-z]+}={value:.+}', 'alpha=βeta');

  assert.equal(first.text, 'alpha=βeta');
  assert.equal(first.captures.word, 'alpha');
  assert.equal(first.captures.value, 'βeta');
});

test('findAll advances after zero-width matches', () => {
  assert.deepEqual(
    findAll('a?', 'baa').map((found) => found.text),
    ['', 'a', 'a', ''],
  );
});

test('rewrite applies ordered rules until a terminal rule matches', () => {
  assert.deepEqual(
    rewrite(
      [
        { pattern: '{left:a}{right:b}', replacement: '$right$left' },
        { pattern: 'ba', replacement: 'done', terminal: true },
      ],
      'ab',
      { maxSteps: 10 },
    ),
    {
      output: 'done',
      steps: 2,
      terminated: true,
      maxStepsReached: false,
    },
  );
});

test('rewrite reports when the step limit stops applicable rules', () => {
  assert.deepEqual(rewrite([{ pattern: 'a', replacement: 'aa' }], 'a', { maxSteps: 3 }), {
    output: 'aaaa',
    steps: 3,
    terminated: false,
    maxStepsReached: true,
  });
});
