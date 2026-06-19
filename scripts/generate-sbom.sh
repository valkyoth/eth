#!/usr/bin/env sh
set -eu

mkdir -p sbom
cargo sbom --output-format spdx_json_2_3 > sbom/eth.spdx.json
test -s sbom/eth.spdx.json
