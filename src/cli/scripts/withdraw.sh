#!/usr/bin/env bash

# Script to withdraw stakes and SOL from a stake pool, given the stake pool public key
# and a path to a file containing a list of validator vote accounts

cd "$(dirname "$0")" || exit
validator_list=$1
withdraw_sol_amount=$2

create_keypair () {
  if test ! -f "$1"
  then
    solana-keygen new --no-passphrase -s -o "$1"
  fi
}

withdraw_stakes () {
  validator_list=$1
  pool_amount=$2
  while read -r validator
  do
    $westake withdraw-stake "$pool_amount" --vote-account "$validator"
  done < "$validator_list"
}

keys_dir=keys

westake=westake
# Uncomment to use a locally build CLI
#westake=../../../target/debug/westake

echo "Setting up keys directory $keys_dir"
mkdir -p $keys_dir
authority=$keys_dir/authority.json
echo "Setting up authority for withdrawn stake accounts at $authority"
create_keypair $authority

echo "Withdrawing stakes from stake pool"
withdraw_stakes "$stake_pool_pubkey" "$validator_list" "$withdraw_sol_amount"
echo "Withdrawing SOL from stake pool to authority"
$westake withdraw-sol "$stake_pool_pubkey" $authority "$withdraw_sol_amount"
