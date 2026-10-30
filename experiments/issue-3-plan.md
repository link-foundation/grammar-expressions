# Issue #3 work plan

- [x] Read issue body, all issue comments, PR conversation, inline comments, reviews, and sub-issues.
- [x] Inspect runtime layout, manifests, CI, and merged PR style; confirm prepared branch.
- [x] Choose publication: reusable Rust library/CLI, documented independent runtime releases.
- [x] Reproduce missing release automation with an automated workflow contract test.
- [x] Add pinned OIDC authentication, continue-on-error, and token fallback per policy comment.
- [x] Validate packages on PRs without credentials; publish only from main after successful checks.
- [x] Document bootstrap/trusted-publisher registration and any external blockers.
- [x] Run full local CI sequentially with CARGO_BUILD_JOBS=1; save verbose logs.
- [ ] Review diff, merge latest main if necessary, commit and push only prepared branch.
- [ ] Update PR #4 title/body, include Fixes #3 and verification; inspect fresh CI/logs.
- [ ] Mark PR ready only after verification; report exact unfinished external criteria.

Acceptance mapping:
- Issue publication path: release workflow plus first crates.io publication and trusted publisher registration.
- Policy comment: pinned auth action, id-token: write, continue-on-error: true,
  token selection OIDC output || CARGO_REGISTRY_TOKEN || CARGO_TOKEN.
- Later versions publish from CI: main-only release following Rust verification, version-aware idempotence.

No required sub-issues. package-registry-manager#62 and pipeline-template#205 are
context/alternative tooling, not required sub-issues.

Evidence so far:
- Nine unit tests cover first/new/existing/yanked releases, failed registry probes,
  malformed index data, and GitHub outputs from the package manifest.
- Tests before implementation fail because release automation is missing; log:
  ci-logs/release-tests-before.log. Passing log: ci-logs/release-tests-after.log.
- Local formatting, Clippy (-Dwarnings), Rust tests/doctests/release build, file
  checks, package dry run, JavaScript checks, and actionlint all pass.
- Registry probe: https://index.crates.io/gr/am/grammar-expressions returns 404.
- Repository secret list is empty; organization secret metadata lists CARGO_TOKEN.
  No secret values were read. No local registry credential or browser profile.
- External criteria unfinished: first registry publication and trusted publisher
  registration. Bootstrap requires authenticated crates.io owner session, and
  publishing workflow is intentionally restricted to main after merge.
