#!/bin/bash
build_directory="target/wasm32-unknown-unknown/release"
artifacts_directory="artifacts/stellar"

mkdir -p "$artifacts_directory"

cd contracts/soroban
cargo clean
cargo build --target wasm32-unknown-unknown --release

for WASM in $build_directory/*.wasm; do
  NAME=$(basename "$WASM" .wasm)${SUFFIX}.wasm
  echo "Optimizing $NAME ... $WASM"
  stellar contract optimize --wasm "$WASM"
done

cd -
for WASM in contracts/soroban/$build_directory/*.optimized.wasm; do
  NAME=$(basename "$WASM" .wasm)${SUFFIX}.wasm

  mv "$WASM" "$artifacts_directory/$NAME"
done



