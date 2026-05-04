#!/bin/bash
set -e

fdbcli --exec "configure new single memory"
echo "FoundationDB configured."
exit 0
