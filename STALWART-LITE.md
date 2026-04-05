# stalwart-lite

Fork of [stalwartlabs/stalwart](https://github.com/stalwartlabs/stalwart) maintained for **Unthought**: same mail server, **bundled webadmin disabled** (no download, no update route, no static asset serving from the webadmin bundle).

- Default branch: `main` (tracks upstream `main` + one commit with lite changes)
- Rebase onto upstream periodically and resolve conflicts as needed
- Unthought builds this repo in `cloudflare/container/stalwart/Dockerfile` via `STALWART_REPO_URL`

See commit history on `main` for the exact diff.
