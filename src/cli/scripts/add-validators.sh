#!/usr/bin/env bash

# Script to add new validators to a stake pool, given the stake pool keyfile and
# a file listing validator vote account pubkeys

cd "$(dirname "$0")" || exit
stake_pool_keyfile=$1
validator_list=$2  # File containing validator vote account addresses, each will be added to the stake pool after creation

add_validator_stakes () {
  stake_pool=$1
  validator_list=$2
  while read -r validator
  do
    $westake add-validator "$stake_pool" "$validator"
  done < "$validator_list"
}

# Uncomment to use a local build
westake=../../../target/debug/westake

stake_pool_pubkey=$(solana-keygen pubkey "$stake_pool_keyfile")
echo "Adding validator stake accounts to the pool"
add_validator_stakes "$stake_pool_pubkey" "$validator_list"
