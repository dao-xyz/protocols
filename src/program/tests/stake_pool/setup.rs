use s2g::{
    stake_pool::find_stake_pool_program_address,
    tokens::spl_utils::find_utility_mint_program_address,
};
use solana_program::borsh::get_packed_len;
use solana_program_test::*;
use solana_sdk::{signer::Signer, transaction::Transaction};

use crate::utils::program_test;

use super::helpers::get_account;

#[tokio::test]
async fn success() {
    let program = program_test();
    let (mut banks_client, payer, recent_blockhash) = program.start().await;
    let transaction = s2g::stake_pool::instruction::setup(
        &s2g::id(),
        &payer.pubkey(),
        get_packed_len::<s2g::stake_pool::state::StakePool>() as u64,
    );
    let mut transaction_create = Transaction::new_with_payer(&[transaction], Some(&payer.pubkey()));
    transaction_create.sign(&[&payer], recent_blockhash);
    banks_client
        .process_transaction(transaction_create)
        .await
        .unwrap();

    // Assert expected accounts exists
    let stake_pool_address = find_stake_pool_program_address(&s2g::id()).0;
    let stake_pool = get_account(&mut banks_client, &stake_pool_address).await;
    assert_eq!(stake_pool.owner, s2g::id());

    let mint = get_account(
        &mut banks_client,
        &find_utility_mint_program_address(&s2g::id()).0,
    )
    .await;
    assert_eq!(mint.owner, spl_token::id());
}
