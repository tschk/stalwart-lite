# stalwart-lite

Fork of [stalwartlabs/stalwart](https://github.com/stalwartlabs/stalwart) maintained for **Unthought**: same mail server, **bundled webadmin disabled** (no download, no update route, no static asset serving from the webadmin bundle).

- Default branch: `main` (upstream `main` plus small lite-only commits on top; see `git log`)
- Rebase onto upstream periodically and resolve conflicts as needed
- Unthought builds this repo in `cloudflare/container/stalwart/Dockerfile` via `STALWART_REPO_URL`

See commit history on `main` for the exact diff.

## Branch: `claude/simplify-mailserver-jmap-pXoEi`

Strips the server down to **IMAP+SMTP** — standard email clients (Thunderbird, Apple Mail, phones) connect via IMAP; delivery and submission via SMTP. Protocols removed from this build:

| Removed | Kept |
|---------|------|
| JMAP (`jmap`, `jmap-proto`) | IMAP (`imap`, `imap-proto`) |
| POP3 (`pop3`) | SMTP (inbound, outbound, submission) (`smtp`) |
| ManageSieve (`managesieve`) | Management/OAuth HTTP API (`http`, `http-proto`) |
| CalDAV / CardDAV / WebDAV (`dav`, `dav-proto`) | Spam filtering (`spam-filter`, `nlp`) |
| Calendar / contacts / file storage (`groupware`) | Storage backends, directory, services, migration, cli, tests |

Standard email clients connect on IMAP port (993/143) and SMTP submission (587/465). The HTTP port (80/443) still serves the management API and OAuth.

A minimal public library API is exposed in `crates/main/src/lib.rs` (`start_server()`).
