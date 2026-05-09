#!/usr/bin/env node

import { findFirst, isMatch, replaceAll, rewrite } from '../src/index.js';

const [, , command, pattern, first, second, third] = process.argv;

try {
  if (command === 'match') {
    requireArgs(pattern, first, 'match requires PATTERN INPUT');
    console.log(String(isMatch(pattern, first)));
  } else if (command === 'find') {
    requireArgs(pattern, first, 'find requires PATTERN INPUT');
    const found = findFirst(pattern, first);
    if (found) {
      console.log(`${found.start}..${found.end} ${found.text}`);
      for (const [name, value] of Object.entries(found.captures)) {
        console.log(`${name}=${value}`);
      }
    }
  } else if (command === 'replace') {
    requireArgs(pattern, first, 'replace requires PATTERN REPLACEMENT INPUT');
    if (second === undefined) {
      throw new Error('replace requires PATTERN REPLACEMENT INPUT');
    }
    console.log(replaceAll(pattern, first, second));
  } else if (command === 'rewrite') {
    requireArgs(pattern, first, 'rewrite requires PATTERN REPLACEMENT INPUT [MAX_STEPS]');
    if (second === undefined) {
      throw new Error('rewrite requires PATTERN REPLACEMENT INPUT [MAX_STEPS]');
    }
    const result = rewrite([{ pattern, replacement: first }], second, {
      maxSteps: parseMaxSteps(third),
    });
    console.log(result.output);
  } else {
    throw new Error('expected command: match, find, replace, or rewrite');
  }
} catch (error) {
  console.error(`${error.message}\n\n${usage()}`);
  process.exitCode = 1;
}

function requireArgs(patternValue, inputValue, message) {
  if (patternValue === undefined || inputValue === undefined) {
    throw new Error(message);
  }
}

function parseMaxSteps(value) {
  if (value === undefined) {
    return undefined;
  }

  const maxSteps = Number(value);
  if (!Number.isInteger(maxSteps) || maxSteps < 0) {
    throw new Error('MAX_STEPS must be a non-negative integer');
  }
  return maxSteps;
}

function usage() {
  return [
    'Usage:',
    '  grammar-expressions match PATTERN INPUT',
    '  grammar-expressions find PATTERN INPUT',
    '  grammar-expressions replace PATTERN REPLACEMENT INPUT',
    '  grammar-expressions rewrite PATTERN REPLACEMENT INPUT [MAX_STEPS]',
  ].join('\n');
}
