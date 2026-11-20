#!/usr/bin/env bash

set -eEuo pipefail

SCRIPT=$(readlink -f "$0")
SCRIPT_DIR=$(dirname "$SCRIPT")
cd "$SCRIPT_DIR/.."

# Pin to a specific commit to ensure compatibility with PocketIC server version.
# The asset canister Wasm must not import system APIs unsupported by the PocketIC version
# used in integration tests (see scripts/run-integration-tests.sh for the server version).
# It must also match the `ic-asset` rev in Cargo.toml, whose asset evidence it has to agree with.
SDK_COMMIT=${1:-271392a3b7746df539925a6333b4cd7ddc9f1972}

echo "Downloading asset canister at commit $SDK_COMMIT"

mkdir -p wasms
cd wasms

HTTP_CODE=$(curl -L -so assetstorage.wasm.gz "https://github.com/dfinity/sdk/raw/${SDK_COMMIT}/src/distributed/assetstorage.wasm.gz" --write-out "%{http_code}")

if [[ ${HTTP_CODE} -ne 200 ]] ; then
    echo "Failed to download wasm. Response code: ${HTTP_CODE}"
    exit 1
fi

echo "Asset canister wasm downloaded"
