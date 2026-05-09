# grammar-expressions

A PEG/regular-expression style grammar expression engine implemented as a Rust package and a
JavaScript package in one repository.

The repository is split by runtime so each engine can evolve, test, publish, and release through
runtime-specific tooling:

```text
rust/   Rust library, CLI, tests, examples, and package checks
js/     JavaScript library, CLI, HTTP microservice, tests, examples, and package checks
```

Both engines currently expose the same public behavior:

- literal matching, grouping, ordered choice with `|`
- wildcard `.`
- character classes such as `[A-Za-z]` and `[^0-9]`
- `*`, `+`, and `?` repetition
- named captures with `{name:expression}`
- full matching, searching, and capture-aware replacement
- bounded ordered rewrite rules for Markov-style substitutions
- command-line entry points for Rust and JavaScript
- a JSON HTTP microservice for JavaScript

The broader language, grammar inference, links-notation compatibility, WebAssembly, web app, and
service roadmap from issue [#1](https://github.com/link-foundation/grammar-expressions/issues/1)
is tracked in [docs/case-studies/issue-1](docs/case-studies/issue-1/README.md).

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
cargo fmt --all -- --check
cargo clippy -p grammar-expressions --all-targets --all-features
cargo test -p grammar-expressions --all-features
```

Run the Rust CLI:

```sh
cargo run -p grammar-expressions -- match "{word:[A-Za-z]+}" "grammar"
cargo run -p grammar-expressions -- replace "{key:[A-Za-z]+}: {value:[0-9]+}" "$key=$value" "width: 42"
cargo run -p grammar-expressions -- rewrite "{left:a}{right:b}" "$right$left" "ab" 10
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
npm --prefix js run check
```

Run the JavaScript CLI:

```sh
node js/bin/grammar-expressions.js match "{word:[A-Za-z]+}" "grammar"
node js/bin/grammar-expressions.js replace "{key:[A-Za-z]+}: {value:[0-9]+}" '$key=$value' "width: 42"
node js/bin/grammar-expressions.js rewrite "{left:a}{right:b}" '$right$left' "ab" 10
```

Run the JavaScript microservice:

```sh
node js/bin/grammar-expressions-server.js 8787
curl -sS http://127.0.0.1:8787/replace \
  -H 'content-type: application/json' \
  -d '{"pattern":"{key:[A-Za-z]+}: {value:[0-9]+}","replacement":"$key=$value","input":"width: 42"}'
```

Service endpoints:

- `GET /health`
- `POST /match` with `{ "pattern": "...", "input": "..." }`
- `POST /find` with `{ "pattern": "...", "input": "..." }`
- `POST /replace` with `{ "pattern": "...", "replacement": "...", "input": "..." }`
- `POST /rewrite` with `{ "rules": [{ "pattern": "...", "replacement": "...", "terminal": false }], "input": "...", "maxSteps": 1000 }`

## Rewrite rules

Rewrite rules are ordered. On each step, the engine finds the first rule whose pattern matches the
current text, replaces the first match, and restarts from the first rule. A terminal rule stops the
run after its replacement. `maxSteps` bounds execution so intentionally powerful rewrite systems
cannot loop forever by accident.

```js
import { rewrite } from 'grammar-expressions';

const result = rewrite(
  [
    { pattern: '{left:a}{right:b}', replacement: '$right$left' },
    { pattern: 'ba', replacement: 'done', terminal: true },
  ],
  'ab',
  { maxSteps: 10 },
);

console.log(result.output); // done
```

## Design notes

The engine uses ordered choice, so `a|ab` matches `a` first. Repetition is greedy but can
backtrack inside the current expression. Matching uses UTF-8/Unicode character boundaries in Rust
and Unicode code point boundaries in JavaScript.

The current core intentionally has no third-party runtime dependencies. Packrat memoization,
grammar rule definitions, links-notation round-tripping, WebAssembly bindings, and inference from
examples are tracked as explicit next implementation areas in the case study.
