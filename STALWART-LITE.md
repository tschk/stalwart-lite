# stalwart-lite

Fork of [stalwartlabs/stalwart](https://github.com/stalwartlabs/stalwart) maintained for **Unthought**: same mail server, **bundled webadmin disabled** (no download, no update route, no static asset serving from the webadmin bundle).

- Default branch: `main` (upstream `main` plus small lite-only commits on top; see `git log`)
- Rebase onto upstream periodically and resolve conflicts as needed
- Unthought builds this repo in `cloudflare/container/stalwart/Dockerfile` via `STALWART_REPO_URL`

See commit history on `main` for the exact diff.

## Branch: `claude/simplify-mailserver-jmap-pXoEi`

Strips the server down to **JMAP-only** — email store, retrieval, and submission via JMAP over HTTP. Protocols removed from this build:

| Removed | Kept |
|---------|------|
| IMAP (`imap`, `imap-proto`) | JMAP over HTTP (`jmap`, `jmap-proto`, `http`) |
| POP3 (`pop3`) | Outbound SMTP queue for `EmailSubmission` (`smtp`) |
| ManageSieve (`managesieve`) | Spam filtering (`spam-filter`, `nlp`) |
| CalDAV / CardDAV / WebDAV (`dav`, `dav-proto`) | All email JMAP methods: Email, Mailbox, Thread, Identity, Blob, Submission, Push, Sieve, Vacation, Quota |
| Calendar / contacts / file storage (`groupware`) | Storage backends, directory, services, migration, cli, tests |

A minimal public library API is exposed in `crates/main/src/lib.rs` (`start_server()`).
