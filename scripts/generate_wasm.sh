#!/bin/bash
set -e

# install wasm-opt
BINARYEN_VERS=110
BINARYEN_DWN="https://github.com/WebAssembly/binaryen/releases/download/version_${BINARYEN_VERS}/binaryen-version_${BINARYEN_VERS}-x86_64-linux.tar.gz"

if ! which wasm-opt; then
  curl -OL $BINARYEN_DWN
  tar xf binaryen-version_${BINARYEN_VERS}-x86_64-linux.tar.gz
  export PATH=$PATH:$PWD/binaryen-version_${BINARYEN_VERS}/bin
fi

# Generate optimized wasm files and verify generated wasm with cosmwasm-check
mkdir -p artifacts
RUSTFLAGS='-C link-arg=-s' cargo wasm
for WASM in ./target/wasm32-unknown-unknown/release/*.wasm; do
  NAME=$(basename "$WASM" .wasm)${SUFFIX}.wasm
  echo "########Creating intermediate hash for $NAME ...########"
  sha256sum -- "$WASM" | tee -a artifacts/checksums_intermediate.txt
  echo "########Optimizing $NAME ...########"
  wasm-opt -Oz "$WASM" -o "artifacts/$NAME"
  echo "########Verifying $NAME file with cosmwasm-check ...########"
done

cosmwasm-check "artifacts/cw_hub_bnusd.wasm"
