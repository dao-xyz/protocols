#!/usr/bin/env bash

# Script to add a certain amount of SOL into a stake pool, given the stake pool
# keyfile and a path to a file containing a list of validator vote accounts

cd "$(dirname "$0")" || exit
validator_list=$1
sol_amount=$2

westake=../../../target/debug/westake

increase_stakes () {
  stake_pool_pubkey=$1
  validator_list=$2
  sol_amount=$3
  while read -r validator
  do
    $westake increase-validator-stake "$validator" "$sol_amount"
  done < "$validator_list"
}

echo "Increasing amount delegated to each validator in stake pool"
increase_stakes "$validator_list" "$sol_amount"
