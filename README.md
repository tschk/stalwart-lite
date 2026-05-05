# stalwart-lite

`stalwart-lite` is a maintained fork of
[Stalwart](https://github.com/stalwartlabs/stalwart) for Unthought deployments.
It keeps the Stalwart mail and collaboration server, but removes the bundled
webadmin asset path from this repository: no webadmin bundle download, no
webadmin update route, and no static serving of bundled webadmin files.

The server, management APIs, protocol support, storage backends, and release
workflow otherwise track upstream Stalwart as closely as possible.

[![CI](https://img.shields.io/github/actions/workflow/status/tschk/stalwart-lite/ci.yml?style=flat-square)](https://github.com/tschk/stalwart-lite/actions/workflows/ci.yml)
[![License: AGPL v3](https://img.shields.io/badge/License-AGPL_v3-blue.svg?label=license&style=flat-square)](https://www.gnu.org/licenses/agpl-3.0)

## Why This Fork Exists

Unthought builds Stalwart inside its own deployment pipeline and does not want
the upstream bundled webadmin behavior in the server binary. This fork is kept
small on purpose so upstream changes can be reviewed and merged without turning
the project into a separate product.

Main differences from upstream:

- Bundled webadmin download/update/static-serving code is disabled.
- Release and install assets point at `tschk/stalwart-lite`.
- Docker and test example files are kept current with upstream where useful.
- Local project notes live in [STALWART-LITE.md](./STALWART-LITE.md).

## Protocols And Features

`stalwart-lite` inherits Stalwart's Rust mail server stack, including:

- SMTP inbound, outbound, submission, queueing, DKIM, DMARC, SPF, ARC, MTA-STS,
  DANE, throttling, filtering, and reporting.
- IMAP, POP3, ManageSieve, JMAP, CalDAV, CardDAV, and WebDAV support on the
  main branch.
- Storage backends such as RocksDB, FoundationDB, PostgreSQL, MySQL, SQLite,
  S3-compatible storage, Azure, Redis, Kafka, and NATS.
- Spam and phishing filtering, full-text search, OpenID Connect, OAuth,
  LDAP/OIDC/SQL/internal directories, roles, ACLs, telemetry, metrics, and
  webhooks.

The bundled webadmin UI is the intentional exception in this fork. Use external
deployment/admin tooling or the server management APIs instead.

## Build

Debug check:

```bash
cargo check --workspace
```

Release binary:

```bash
cargo build --release -p stalwart-lite --no-default-features \
  --features "sqlite postgres mysql rocks s3 redis azure nats enterprise"
```

CLI:

```bash
cargo build --release -p stalwart-lite-cli
```

## Docker

Build the runtime image from this repo:

```bash
docker build -t stalwart-lite .
```

Run with explicit config and data volumes:

```bash
docker run --rm \
  -p 25:25 -p 587:587 -p 993:993 -p 8080:8080 \
  -v "$PWD/config:/etc/stalwart" \
  -v "$PWD/data:/var/lib/stalwart" \
  stalwart-lite
```

Current Docker images run as the `stalwart` user, store config under
`/etc/stalwart`, store data under `/var/lib/stalwart`, and expect
`/etc/stalwart/config.json` by default.

## Install Script

The install script downloads release assets from this repository:

```bash
curl -fsSL https://raw.githubusercontent.com/tschk/stalwart-lite/main/install.sh | sudo sh
```

Custom prefix:

```bash
sudo sh install.sh /opt/stalwart-lite
```

FoundationDB build:

```bash
sudo sh install.sh --fdb
```

## Publishing Status

GitHub releases are the supported distribution path for this fork. Release
publishing is driven by pushed tags that match `v*.*.*`, and generated assets
are consumed by [install.sh](./install.sh).

This repository is not currently ready for crates.io publication. The server
package depends on local workspace crates through path-only dependencies, and
those internal crates would need registry-safe package names and published
versions before `cargo publish` can work. See [PUBLISHING.md](./PUBLISHING.md)
for the exact crates.io migration checklist.

## Test Infrastructure

Upstream Docker Compose examples for external services are kept under
[tests/docker](./tests/docker):

```bash
cd tests/docker
docker compose up -d
```

That stack provides PostgreSQL, MySQL, FoundationDB, Redis, OpenSearch,
Meilisearch, MinIO, Keycloak, OpenLDAP, Pebble ACME, PowerDNS, and NATS for
local integration testing.

## Tracking Upstream

This repo is meant to stay close to upstream Stalwart. When updating from
upstream, review changes carefully and preserve the lite-specific behavior:

- Do not restore bundled webadmin download/update/static-serving code.
- Keep installer and release links pointed at `tschk/stalwart-lite`.
- Keep fork-specific notes in [STALWART-LITE.md](./STALWART-LITE.md).

## Documentation And Support

For protocol behavior and general Stalwart configuration, use upstream Stalwart
documentation at [stalw.art/docs](https://stalw.art/docs/install/get-started).

For issues specific to this fork, use this repository's issue tracker. For
general Stalwart questions, upstream community channels remain the best source
of broad operational knowledge.

## License

This project follows upstream Stalwart licensing and is dual-licensed under:

- [GNU Affero General Public License v3.0](./LICENSES/AGPL-3.0-only.txt)
- [Stalwart Enterprise License v1](./LICENSES/LicenseRef-SEL.txt)

Each source file carries its applicable license notice. See the
[LICENSES](./LICENSES/) directory for full license text.

## Copyright

Copyright (C) 2020, Stalwart Labs LLC
