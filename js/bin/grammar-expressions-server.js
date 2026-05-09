#!/usr/bin/env node

import { createGrammarExpressionsServer } from '../src/server.js';

const port = Number(process.env.PORT ?? process.argv[2] ?? 8787);

if (!Number.isInteger(port) || port <= 0) {
  console.error('PORT must be a positive integer');
  process.exitCode = 1;
} else {
  const server = createGrammarExpressionsServer();
  server.listen(port, () => {
    console.log(`grammar-expressions server listening on http://127.0.0.1:${port}`);
  });
}
