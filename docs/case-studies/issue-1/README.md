# Case Study: Issue #1 - Initial Grammar Expressions Implementation

**Issue:** [#1](https://github.com/link-foundation/grammar-expressions/issues/1)
**Pull request:** [#2](https://github.com/link-foundation/grammar-expressions/pull/2)
**Date:** 2026-05-09
**Status:** Runtime split, CI split, and executable engine features implemented

## Executive Summary

Issue #1 asks for a complete grammar expression system spanning PEG-like parsing,
regular-expression-like text matching, Markov-style substitutions, Rust and JavaScript libraries,
CLI tooling, a microservice, a GitHub Pages web app, links-notation compatibility, WebAssembly
parity, and grammar inference from examples.

The repository started with only `README.md`, `LICENSE`, `.gitignore`, and a placeholder
`.gitkeep`. This PR establishes working Rust and JavaScript packages, keeps the broad roadmap
visible, and implements bounded rewrite semantics so substitution rules can execute as ordered
transforms instead of only single-pass replacements.

The implemented slice includes:

- Rust library and CLI.
- JavaScript package, CLI, and JSON HTTP microservice.
- Shared behavior for literals, grouping, ordered choice, wildcard, character classes, repetition,
  named captures, search, full-match, and capture-aware replacement.
- Bounded ordered rewrite rules with terminal rules and step-limit reporting for Markov-style
  substitution systems.
- Rust and JavaScript tests for matching, replacement, Unicode-boundary behavior, and zero-width
  scanning, plus rewrite-rule parity tests.
- Runtime-specific `rust/src` and `js/src` package layouts with examples for both languages.
- Separate Rust and JavaScript workflows with template-derived timeouts and file-size checks.
- Captured issue, PR, and template data under this case-study directory.

## Data Captured

| File | Purpose |
| --- | --- |
| `data/issue-1.json` | Full issue metadata captured with GitHub CLI. |
| `data/pr-2.json` | Pull request metadata before implementation. |
| `data/pr-2-review-comments.json` | Inline review comments for PR #2; empty at capture time. |
| `data/pr-2-conversation-comments.json` | PR conversation comments, including the follow-up requesting split Rust/JS layout and CI. |
| `data/current-repository-file-tree.txt` | Repository file tree after restructuring. |
| `data/js-template-file-tree.txt` | Full file tree from `js-ai-driven-development-pipeline-template`. |
| `data/rust-template-file-tree.txt` | Full file tree from `rust-ai-driven-development-pipeline-template`. |
| `data/js-template-release.yml` | JavaScript template workflow used for CI/CD comparison. |
| `data/rust-template-release.yml` | Rust template workflow used for CI/CD comparison. |

## Requirements and Resolution

| Requirement | Current resolution | Follow-up direction |
| --- | --- | --- |
| PEG/regular-expression-like grammar language | Implemented initial expression parser with ordered choice, grouping, wildcard, classes, captures, and repetition. | Add named grammar rules, lookahead, rule references, packrat memoization, and richer diagnostics. |
| Parsers for programming and natural languages | Not claimable in the first slice. | Evolve toward a real grammar definition format with rule graphs, left-recursion policy, AST construction, and conformance suites. |
| General substitution patterns | Implemented `replace_all` with `$name`, `${name}`, `$0`, and `$$`. | Add ordered rule sets, terminal rules, fixed-point execution, and explicit loop limits for Markov-style systems. |
| Turing-complete substitutions | Implemented bounded ordered rewrite rules with terminal rules and max-step reporting. | Add execution traces, rule labels, and persisted rule-set formats. |
| Rust library and CLI | Implemented in `rust/src/lib.rs` and `rust/src/main.rs`. | Add crates.io publication workflow once API stabilizes. |
| JavaScript library and CLI | Implemented in `js/src/index.js`, `js/src/index.d.ts`, and `js/bin/grammar-expressions.js`. | Add npm publication workflow once package metadata is finalized. |
| Microservice | Implemented JavaScript HTTP service with `/health`, `/match`, `/find`, `/replace`, and `/rewrite`. | Add Rust service parity and richer error/status mapping after result schemas stabilize. |
| GitHub Pages web app | Not included in this first PR. | Add a playground that can run JS and Rust/WASM engines side by side in web workers. |
| Rust + WebAssembly parity | Core Rust API is dependency-free and suitable for future WASM bindings. | Add `wasm-bindgen` package and parity tests against the JS engine. |
| links-notation compatibility | Not implemented yet. | Define an AST serialization that can round-trip through links-notation tokens and link-cli substitutions. |
| Grammar inference by examples | Researched and planned below. | Start with regular-language inference for small alphabets and PBE-style synthesis for substitutions. |
| Runtime package layout | Implemented explicit `rust/` and `js/` package roots with `rust/src` and `js/src`. | Keep generated WASM and web assets isolated under their own package folders. |
| CI/CD best practices from templates | Compared template file trees and added separate Rust and JavaScript workflows with timeouts, concurrency, matrix tests, build checks, and file-size scripts. | Add release automation after package naming, registry targets, and version policy are decided. |

## Research Notes

Bryan Ford's PEG work is the right theoretical base for the recognition side of this project:
PEGs use prioritized choice and avoid ambiguity by construction. The current engine follows that
direction with ordered `|` alternatives, but it is not yet packrat-memoized.

Grammar inference from examples should be split into two tracks:

1. Regular-language inference for recognizers. Angluin's L* result shows that regular languages can
   be learned efficiently when the learner has membership queries and equivalence/counterexample
   feedback. For this project, that maps well to interactive grammar construction where a user can
   confirm or reject candidate matches.
2. Programming-by-example for transformations. Flash Fill and PROSE-style systems synthesize ranked
   DSL programs from input-output examples. That is a better model for substitution inference than
   trying to infer arbitrary Turing-complete rewrite systems directly.

Practical implication: the first inference feature should synthesize small, ranked expression and
replacement candidates from positive/negative examples. Turing-complete rule inference should come
later and require explicit execution bounds, traces, and counterexample-driven refinement.

## Template Comparison

The JavaScript template includes npm metadata, Node tests, formatting/linting, changesets,
link-checking, examples, experiments, and extensive case-study practice.

The Rust template includes Cargo metadata, rustfmt/clippy/test workflows, release automation,
changelog fragments, pre-commit hooks, examples, experiments, and case studies.

This repository now adopts the low-risk subset that is useful immediately:

- Root `Cargo.toml` workspace with the Rust package in `rust/`.
- `rust/Cargo.toml` with library and binary targets.
- `js/package.json` with package exports, TypeScript declarations, binary mapping, and Node tests.
- JavaScript HTTP service as the first microservice implementation.
- `.github/workflows/rust.yml` and `.github/workflows/js.yml` with explicit job timeouts.
- `rust/scripts/check-file-size.sh` and `js/scripts/check-file-line-limits.sh`, adapted from the
  template file-size guardrails.
- `rust/examples/` and `js/examples/`.
- `docs/case-studies/issue-1/` with captured data.

Release automation is deliberately deferred. The package API and publication targets need review
before adding crates.io/npm publishing secrets or automated version bumping.

## Solution Plan

1. Stabilize the core expression AST and parser errors.
2. Add rule definitions and references so expressions can describe larger grammars.
3. Add packrat memoization and recursion checks before advertising programming-language parsing.
4. Add parity fixtures that run the same cases through Rust, JavaScript, and Rust/WASM.
5. Add Rust HTTP service parity once the library result types are stable.
6. Add a GitHub Pages playground that executes JS and WASM engines in separate workers and displays
   divergences.
7. Add links-notation import/export and link-cli substitution compatibility tests.
8. Add inference prototypes under `experiments/`, then promote useful cases into `examples/`.

## Verification

```sh
cargo fmt --all -- --check
cargo clippy -p grammar-expressions --all-targets --all-features
cargo test -p grammar-expressions --all-features --verbose
npm --prefix js run check
```

## References

- [Parsing Expression Grammars: A Recognition-Based Syntactic Foundation](https://pdos.csail.mit.edu/~baford/packrat/popl04/)
- [Learning Regular Sets from Queries and Counterexamples](https://homepages.math.uic.edu/~lreyzin/papers/angluin87.pdf)
- [Automating String Processing in Spreadsheets using Input-Output Examples](https://www.microsoft.com/en-us/research/publication/automating-string-processing-spreadsheets-using-input-output-examples/)
- [Microsoft PROSE Framework](https://www.microsoft.com/en-us/research/project/prose-framework/)
- [JavaScript AI-driven development pipeline template](https://github.com/link-foundation/js-ai-driven-development-pipeline-template)
- [Rust AI-driven development pipeline template](https://github.com/link-foundation/rust-ai-driven-development-pipeline-template)
