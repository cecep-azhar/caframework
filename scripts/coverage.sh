#!/usr/bin/env bash
set -e
# Run cargo tarpaulin and output HTML report
cargo tarpaulin -o Html
# Move generated report to evidence folder
mkdir -p ../Notes/evidence/fix
mv tarpaulin-report.html ../Notes/evidence/fix/coverage.html
