# Publishing

`stalwart-lite` currently publishes release artifacts through GitHub releases.
Release workflows are triggered by tags that match `v*.*.*`.

## GitHub Releases

Use GitHub releases as the primary distribution channel:

1. Confirm CI is green on `main`.
2. Create and push a version tag:

   ```bash
   git tag v0.15.5-lite.1
   git push origin v0.15.5-lite.1
   ```

3. Wait for the release workflow to finish.
4. Verify the release assets and install script still point at
   `tschk/stalwart-lite`.

## crates.io Status

The crate name `stalwart-lite` appears to be unused as of 2026-05-04, but this
workspace is not ready for `cargo publish`.

The server package depends on many local workspace crates with path-only
dependencies:

```text
common, directory, email, http, imap, migration, services, smtp, store, trc,
types, utils, and related protocol crates
```

Cargo does not allow publishing a crate that depends on unpublished local path
dependencies. Adding `version = "0.15.5"` to those dependency declarations only
moves resolution to crates.io for published consumers; each internal dependency
would also need to exist on crates.io under an owned package name.

Current packaging check:

```bash
cargo package -p stalwart --allow-dirty --no-verify
```

Expected blocker:

```text
all dependencies must have a version requirement specified when packaging.
dependency `common` does not specify a version
```

## Required crates.io Plan

To publish this fork to crates.io, do the registry migration deliberately:

1. Reserve/own `stalwart-lite` on crates.io.
2. Choose names for every internal crate, preferably fork-prefixed names such as
   `stalwart-lite-common`, `stalwart-lite-store`, and
   `stalwart-lite-jmap-proto`.
3. Rename internal package names or add dependency aliases so source imports can
   keep using existing crate names while registry package names are unique.
4. Add registry `version` requirements to every internal path dependency.
5. Publish internal crates in dependency order.
6. Run `cargo publish --dry-run -p stalwart-lite`.
7. Publish the final server crate only after the dry run resolves all registry
   dependencies and ownership checks.

Do not run `cargo publish` from this repository until these steps are complete.
