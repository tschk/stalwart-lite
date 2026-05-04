#!/bin/bash
set -e

# Wait for the SQLite database to be ready
sleep 2

# Create the zone
pdnsutil create-zone stalwart.test ns1.stalwart.test
pdnsutil set-kind stalwart.test native

# Add basic records
pdnsutil add-record stalwart.test '' SOA 'ns1.stalwart.test. admin.stalwart.test. 2024010101 3600 900 604800 86400'
pdnsutil add-record stalwart.test '' NS 'ns1.stalwart.test.'
pdnsutil add-record stalwart.test 'ns1' A '127.0.0.1'
pdnsutil add-record stalwart.test '' A '127.0.0.1'
pdnsutil add-record stalwart.test '' MX '10 mail.stalwart.test.'
pdnsutil add-record stalwart.test 'mail' A '127.0.0.1'

# Add a sample TLSA record
# Usage=3 (DANE-EE), Selector=1 (SubjectPublicKeyInfo), Matching=1 (SHA-256)
pdnsutil add-record stalwart.test '_25._tcp.mail' TLSA '3 1 1 0000000000000000000000000000000000000000000000000000000000000000'

# Import static TSIG key for RFC2136 dynamic updates
# Key: stalwart-update-key / HMAC-SHA256
# Base64 secret: c3RhbHdhcnQtdGVzdC10c2lnLXNlY3JldC1rZXkxMjM0NTY3ODkw
pdnsutil import-tsig-key stalwart-update-key hmac-sha256 'c3RhbHdhcnQtdGVzdC10c2lnLXNlY3JldC1rZXkxMjM0NTY3ODkw'
pdnsutil activate-tsig-key stalwart.test stalwart-update-key master
pdnsutil set-meta stalwart.test TSIG-ALLOW-DNSUPDATE stalwart-update-key
pdnsutil set-meta stalwart.test ALLOW-DNSUPDATE-FROM '0.0.0.0/0'

echo "PowerDNS zone setup complete."
echo "TSIG key name:      stalwart-update-key"
echo "TSIG algorithm:     hmac-sha256"
echo "TSIG secret (b64):  c3RhbHdhcnQtdGVzdC10c2lnLXNlY3JldC1rZXkxMjM0NTY3ODkw"
