# stalwart-lite

Fork of [stalwartlabs/stalwart](https://github.com/stalwartlabs/stalwart) maintained for **Unthought**: same mail server, **bundled webadmin disabled** (no download, no update route, no static asset serving from the webadmin bundle).

- Default branch: `main` (upstream `main` plus small lite-only commits on top; see `git log`)
- Rebase onto upstream periodically and resolve conflicts as needed.
- Unthought builds this repo in `cloudflare/container/stalwart/Dockerfile` via `STALWART_REPO_URL`.
- GitHub releases are the supported publishing path. crates.io publication needs the registry migration described in [PUBLISHING.md](./PUBLISHING.md).

See commit history on `main` for the exact diff.

A minimal public library API is exposed in `crates/main/src/lib.rs` (`start_server()`).
