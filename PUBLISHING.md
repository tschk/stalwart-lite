# Publishing

`stalwart-lite` publishes release artifacts through GitHub releases and crates.io.

## GitHub Releases

Release workflows trigger on tags matching `v*.*.*`.

1. Confirm CI is green on `main`.
2. Create and push a version tag:

   ```bash
   git tag v0.15.6
   git push origin v0.15.6
   ```

3. Wait for the release workflow to finish.
4. Verify the release assets and install script still point at
   `tschk/stalwart-lite`.

## crates.io

### Package layout

`stalwart-lite` is published from the repository root. Internal Stalwart modules
remain under `crates/*/src`, but they are compiled as modules of the root
`stalwart-lite` package.

Rust proc macros must live in separate crates, so two helper crates are
published first:

- `stalwart-lite-event-macro`
- `stalwart-lite-proc-macros`

### Publish order

```bash
cargo publish --dry-run -p stalwart-lite-event-macro
cargo publish -p stalwart-lite-event-macro
cargo publish --dry-run -p stalwart-lite-proc-macros
cargo publish -p stalwart-lite-proc-macros
cargo publish --dry-run -p stalwart-lite
cargo publish -p stalwart-lite
```

The main crate depends on the helper crates by registry version, so crates.io
must finish indexing the helpers before publishing `stalwart-lite`.

### Validation

Before publishing, confirm:

```bash
cargo fmt --all --check
cargo check --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```
