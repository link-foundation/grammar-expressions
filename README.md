# grammar-expressions

A small initial implementation of a PEG/regular-expression style grammar expression engine.

This repository currently contains matching engines for Rust and JavaScript with the same
public behavior:

- literal matching, grouping, ordered choice with `|`
- wildcard `.`
- character classes such as `[A-Za-z]` and `[^0-9]`
- `*`, `+`, and `?` repetition
- named captures with `{name:expression}`
- full matching, searching, and capture-aware replacement
- command-line entry points for Rust and JavaScript
- a small JSON HTTP microservice for JavaScript

This is the first executable slice for issue
[#1](https://github.com/link-foundation/grammar-expressions/issues/1). The broader language,
grammar inference, links-notation compatibility, WebAssembly, web app, and service roadmap are
documented in [docs/case-studies/issue-1](docs/case-studies/issue-1/README.md).

## Pattern syntax

```text
hello                         literal text
h(i|ello)                     ordered choice
ab+c                          repetition
colou?r                       optional item
[A-Za-z]+                     character class
[^0-9]+                       inverted character class
{name:[A-Za-z]+}: {value:.+}  named captures
```

Backslash escapes make metacharacters literal, for example `\(`, `\)`, `\+`, `\{`, and `\}`.
The escapes `\n`, `\r`, and `\t` produce newline, carriage return, and tab.

## Rust

```rust
use grammar_expressions::{is_match, replace_all};

assert!(is_match("{word:[A-Za-z]+}", "grammar")?);

let output = replace_all(
    "{key:[A-Za-z]+}: {value:[0-9]+}",
    "$key=$value",
    "width: 42 height: 7",
)?;

assert_eq!(output, "width=42 height=7");
# Ok::<(), Box<dyn std::error::Error>>(())
```

Run the Rust checks:

```sh
cargo test
```

Run the Rust CLI:

```sh
cargo run -- match "{word:[A-Za-z]+}" "grammar"
cargo run -- replace "{key:[A-Za-z]+}: {value:[0-9]+}" "$key=$value" "width: 42"
```

## JavaScript

```js
import { isMatch, replaceAll } from 'grammar-expressions';

console.log(isMatch('{word:[A-Za-z]+}', 'grammar'));
console.log(
  replaceAll('{key:[A-Za-z]+}: {value:[0-9]+}', '$key=$value', 'width: 42 height: 7'),
);
```

Run the JavaScript checks:

```sh
npm test
```

Run the JavaScript CLI:

```sh
node bin/grammar-expressions.js match "{word:[A-Za-z]+}" "grammar"
node bin/grammar-expressions.js replace "{key:[A-Za-z]+}: {value:[0-9]+}" '$key=$value' "width: 42"
```

Run the JavaScript microservice:

```sh
node bin/grammar-expressions-server.js 8787
curl -sS http://127.0.0.1:8787/replace \
  -H 'content-type: application/json' \
  -d '{"pattern":"{key:[A-Za-z]+}: {value:[0-9]+}","replacement":"$key=$value","input":"width: 42"}'
```

Service endpoints:

- `GET /health`
- `POST /match` with `{ "pattern": "...", "input": "..." }`
- `POST /find` with `{ "pattern": "...", "input": "..." }`
- `POST /replace` with `{ "pattern": "...", "replacement": "...", "input": "..." }`

## Design notes

The engine uses ordered choice, so `a|ab` matches `a` first. Repetition is greedy but can
backtrack inside the current expression. Matching uses UTF-8/Unicode character boundaries in Rust
and Unicode code point boundaries in JavaScript.

This first version intentionally has no third-party runtime dependencies. Packrat memoization,
grammar rule definitions, links-notation round-tripping, WebAssembly bindings, and inference from
examples are planned extensions rather than hidden promises in the current API.
