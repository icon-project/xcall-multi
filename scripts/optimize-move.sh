#!/bin/bash
set -e
# contracts
CONTRACTS=("contracts/sui/intent_v1" "contracts/sui/libs/sui_rlp" )

#cargo install --locked --git https://github.com/MystenLabs/sui.git --branch testnet sui
start_dir=$(pwd)

for file in "${CONTRACTS[@]}"; do
    cd $file
    sui move test
    sui move build
    cd $start_dir
done

