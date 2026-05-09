import { findFirst, isMatch, replaceAll } from '../src/index.js';

console.log(isMatch('{word:[A-Za-z]+}', 'grammar'));

const found = findFirst('{key:[A-Za-z]+}: {value:[0-9]+}', 'width: 42');
console.log(found.text, found.captures);

console.log(
  replaceAll('{key:[A-Za-z]+}: {value:[0-9]+}', '$key=$value', 'width: 42 height: 7'),
);
