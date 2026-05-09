import assert from 'node:assert/strict';
import { test } from 'node:test';

import { createGrammarExpressionsServer } from '../src/server.js';

test('microservice replaces captures through JSON API', async (t) => {
  const server = createGrammarExpressionsServer();
  await new Promise((resolve) => server.listen(0, '127.0.0.1', resolve));
  t.after(() => server.close());

  const { port } = server.address();
  const response = await fetch(`http://127.0.0.1:${port}/replace`, {
    method: 'POST',
    headers: {
      'content-type': 'application/json',
    },
    body: JSON.stringify({
      pattern: '{key:[A-Za-z]+}: {value:[0-9]+}',
      replacement: '$key=$value',
      input: 'width: 42 height: 7',
    }),
  });

  assert.equal(response.status, 200);
  assert.deepEqual(await response.json(), {
    output: 'width=42 height=7',
  });
});

test('microservice runs bounded rewrite rules through JSON API', async (t) => {
  const server = createGrammarExpressionsServer();
  await new Promise((resolve) => server.listen(0, '127.0.0.1', resolve));
  t.after(() => server.close());

  const { port } = server.address();
  const response = await fetch(`http://127.0.0.1:${port}/rewrite`, {
    method: 'POST',
    headers: {
      'content-type': 'application/json',
    },
    body: JSON.stringify({
      rules: [
        { pattern: '{left:a}{right:b}', replacement: '$right$left' },
        { pattern: 'ba', replacement: 'done', terminal: true },
      ],
      input: 'ab',
      maxSteps: 10,
    }),
  });

  assert.equal(response.status, 200);
  assert.deepEqual(await response.json(), {
    result: {
      output: 'done',
      steps: 2,
      terminated: true,
      maxStepsReached: false,
    },
  });
});
