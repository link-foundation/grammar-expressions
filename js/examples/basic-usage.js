import { findFirst, isMatch, replaceAll, rewrite } from '../src/index.js';

console.log(isMatch('{word:[A-Za-z]+}', 'grammar'));

const found = findFirst('{key:[A-Za-z]+}: {value:[0-9]+}', 'width: 42');
console.log(found.text, found.captures);

console.log(
  replaceAll('{key:[A-Za-z]+}: {value:[0-9]+}', '$key=$value', 'width: 42 height: 7'),
);

const rewritten = rewrite(
  [
    { pattern: '{left:a}{right:b}', replacement: '$right$left' },
    { pattern: 'ba', replacement: 'done', terminal: true },
  ],
  'ab',
  { maxSteps: 10 },
);
console.log(rewritten.output);
