import http from 'node:http';

import { findFirst, isMatch, replaceAll } from './index.js';

export function createGrammarExpressionsServer() {
  return http.createServer(async (request, response) => {
    try {
      const url = new URL(request.url ?? '/', 'http://localhost');

      if (request.method === 'GET' && url.pathname === '/health') {
        sendJson(response, 200, { ok: true });
        return;
      }

      if (request.method !== 'POST') {
        sendJson(response, 405, { error: 'method not allowed' });
        return;
      }

      const body = await readJson(request);

      if (url.pathname === '/match') {
        sendJson(response, 200, {
          matched: isMatch(required(body.pattern, 'pattern'), required(body.input, 'input')),
        });
      } else if (url.pathname === '/find') {
        sendJson(response, 200, {
          match: findFirst(required(body.pattern, 'pattern'), required(body.input, 'input')),
        });
      } else if (url.pathname === '/replace') {
        sendJson(response, 200, {
          output: replaceAll(
            required(body.pattern, 'pattern'),
            required(body.replacement, 'replacement'),
            required(body.input, 'input'),
          ),
        });
      } else {
        sendJson(response, 404, { error: 'not found' });
      }
    } catch (error) {
      sendJson(response, 400, { error: error.message });
    }
  });
}

function required(value, name) {
  if (typeof value !== 'string') {
    throw new Error(`${name} must be a string`);
  }
  return value;
}

function readJson(request) {
  return new Promise((resolve, reject) => {
    let text = '';

    request.setEncoding('utf8');
    request.on('data', (chunk) => {
      text += chunk;
      if (text.length > 1_000_000) {
        reject(new Error('request body is too large'));
        request.destroy();
      }
    });
    request.on('end', () => {
      try {
        resolve(text ? JSON.parse(text) : {});
      } catch {
        reject(new Error('request body must be valid JSON'));
      }
    });
    request.on('error', reject);
  });
}

function sendJson(response, statusCode, payload) {
  response.writeHead(statusCode, {
    'content-type': 'application/json; charset=utf-8',
  });
  response.end(`${JSON.stringify(payload)}\n`);
}
