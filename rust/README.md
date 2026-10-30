# grammar-expressions Rust package

Rust library and command-line interface for the grammar-expressions engine.
This package is intended for crates.io publication.

```sh
cargo test -p grammar-expressions
cargo run -p grammar-expressions -- match "{word:[A-Za-z]+}" "grammar"
```

## Releases

The Rust workflow (`.github/workflows/rust.yml`) tests all three supported operating
systems and verifies the actual package with `cargo publish --dry-run` on pull
requests. Publishing runs only on `main`, after those checks pass. To release,
bump `version` in `rust/Cargo.toml`, update `Cargo.lock`, and merge through a PR.
Version 0.1.1 prepares the first automated release. JavaScript versions are independent.
A manual run of the Rust workflow on `main` can retry a failed release.

The publish job checks the crates.io sparse index and skips versions that already
exist, including yanked versions. Registry/network errors fail the job rather than
being treated as an unpublished version. Release jobs serialize to avoid duplicate
uploads. No registry credentials are used by PR validation.

Authentication tries trusted publishing first with `id-token: write` and the
commit-pinned `rust-lang/crates-io-auth-action`. If that step fails, publishing
falls back to `CARGO_REGISTRY_TOKEN`, then `CARGO_TOKEN`, as repository Actions
secrets. The auth action revokes its temporary OIDC token after the job. Missing
credentials fail publication with an explicit setup message.

### First publication and trusted publisher setup

The workflow does not register a trusted publisher. OIDC requires that
registration first; the fallback secret may publish the initial crate if valid
and sufficiently scoped. A crates.io owner must complete bootstrap after merge:

1. Use `package-registry-manager setup --repository . --registry crates-io --execute`
   from the repository checkout, as specified in issue #3. The tool performs the
   first publication with a short-lived crate-scoped token, revokes and verifies
   that token, and registers the trusted publisher. Review its reported result.
2. Verify the registered GitHub publisher matches owner `link-foundation`, repository
   `grammar-expressions`, workflow filename `rust.yml`, with no environment (this
   workflow does not declare one).
3. Run the Rust workflow on `main`. The bootstrap version is skipped if already
   published; the next version bump exercises OIDC publication with no secret.

If bootstrap is performed manually instead, use a short-lived token scoped to this
crate for `cargo publish --manifest-path rust/Cargo.toml --locked`, revoke it and
verify revocation immediately afterward, and register the same trusted publisher
in the crate's settings. For fallback CI publication, either documented Actions
secret is supported; remove bootstrap secrets after OIDC is working.

At implementation time, the sparse index returned HTTP 404 for this crate.
An organization-level `CARGO_TOKEN` is available to repository workflows, but no
local registry credential or authenticated crates.io browser session is available.
The fallback secret's validity and scopes have not been verified. **The first
publication and publisher registration remain unfinished**: the publishing job
requires this workflow on `main`, and the bootstrap tool requires an authenticated
crates.io owner session. The CI configuration and dry run do not claim that the
crate has been published.

See [issue #3](https://github.com/link-foundation/grammar-expressions/issues/3),
[crates.io trusted publishing](https://crates.io/docs/trusted-publishing), and
[the auth action](https://github.com/rust-lang/crates-io-auth-action).

Test release decisions without credentials or network access:

```sh
python3 -m unittest discover -s rust/scripts/tests -v
```
