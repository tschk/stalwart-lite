# AGENTS.md

Guidance for coding agents working in `stalwart-lite`.

## Project Purpose

`stalwart-lite` is a small fork of `stalwartlabs/stalwart` for Unthought
deployments. Keep the fork close to upstream, but preserve the lite-specific
behavior:

- Do not restore bundled webadmin download logic.
- Do not restore bundled webadmin update routes.
- Do not restore static serving of bundled webadmin assets.
- Keep install and release links pointed at `tschk/stalwart-lite`.
- Keep fork notes in `STALWART-LITE.md` and project-facing docs in `README.md`.

## Upstream Sync Rules

When comparing or importing from upstream Stalwart:

1. Fetch upstream explicitly:

   ```bash
   git fetch https://github.com/stalwartlabs/stalwart.git main
   ```

2. Compare focused paths before applying:

   ```bash
   git diff --name-status HEAD..FETCH_HEAD -- <paths>
   ```

3. Import example, Docker, install, and test infrastructure updates when useful.
4. Review imported files for stale upstream release URLs or wrong local paths.
5. Do not delete fork-specific files unless the user explicitly asks.

## Current Fork-Specific Paths

- `README.md`: user-facing fork README.
- `STALWART-LITE.md`: maintainer notes and fork rationale.
- `install.sh`: downloads release artifacts from `tschk/stalwart-lite`.
- `Dockerfile`, `Dockerfile.build`, `Dockerfile.fdb`: Docker build/runtime
  entrypoints.
- `tests/docker/`: upstream-derived local integration service stack.

## Validation

For Rust code changes:

```bash
cargo fmt --check
cargo check --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

For shell and Docker/example updates:

```bash
git diff --check
bash -n install.sh tests/docker/scripts/*.sh tests/docker/powerdns/*.sh
docker compose -f tests/docker/docker-compose.yml config
```

If Cargo or GitHub commands fail because network is blocked, request network
approval instead of skipping the gate.

## Git Hygiene

- Do not revert user changes.
- Stage only files relevant to the task.
- `AGENTS.md` is part of the repo and should be committed when updated.
- Prefer concise Conventional Commit messages.
- Release publishing happens through tag pushes matching `v*.*.*`.

## Notes

The main branch currently keeps full upstream protocol support except for the
bundled webadmin behavior described above. Do not assume this branch is the old
IMAP/SMTP-only experiment unless the user explicitly switches to that branch.
