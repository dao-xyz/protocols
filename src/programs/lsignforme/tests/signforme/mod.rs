use lsignforme::get_sign_for_me_program_address;
use lsignforme::state::SignForMeAccount;
use solana_program::borsh::try_from_slice_unchecked;
use solana_program::hash::Hash;
use solana_program_test::*;
use solana_sdk::signature::Keypair;
use solana_sdk::{pubkey::Pubkey, signer::Signer, transaction::Transaction};

use crate::utils::program_test;

pub async fn create_sign_for_me(
    banks_client: &mut BanksClient,
    payer: &Keypair,
    recent_blockhash: &Hash,
    owner: &Keypair,
    signer: &Pubkey,
    scope: &Pubkey,
) -> Pubkey {
    // Create tag record
    banks_client
        .process_transaction(Transaction::new_signed_with_payer(
            &[lsignforme::instruction::create_sign_for_me(
                &lsignforme::id(),
                &owner.pubkey(),
                signer,
                scope,
                &payer.pubkey(),
            )],
            Some(&payer.pubkey()),
            &[payer, owner],
            *recent_blockhash,
        ))
        .await
        .unwrap();

    let sign_for_me_address =
        get_sign_for_me_program_address(&lsignforme::id(), &owner.pubkey(), signer, scope).0;
    let sign_for_me_address_account_info = banks_client
        .get_account(sign_for_me_address)
        .await
        .expect("get_sign_for_me")
        .expect("tag not found");
    let sign_for_me_account =
        try_from_slice_unchecked::<SignForMeAccount>(&sign_for_me_address_account_info.data)
            .unwrap();
    assert_eq!(&sign_for_me_account.owner, &owner.pubkey());
    assert_eq!(&sign_for_me_account.signer, signer);
    assert_eq!(&sign_for_me_account.scope, scope);
    sign_for_me_address
}

pub async fn delete_sign_for_me_as_owner(
    banks_client: &mut BanksClient,
    payer: &Keypair,
    recent_blockhash: &Hash,
    sign_for_me: &Pubkey,
    owner: &Keypair,
    withdraw_destination: &Pubkey,
) {
    // Delete tag record
    let balance_pre = banks_client
        .get_balance(*withdraw_destination)
        .await
        .unwrap();
    banks_client
        .process_transaction(Transaction::new_signed_with_payer(
            &[lsignforme::instruction::delete_sign_for_me_as_owner(
                &lsignforme::id(),
                sign_for_me,
                &owner.pubkey(),
                withdraw_destination,
            )],
            Some(&payer.pubkey()),
            &[payer, owner],
            *recent_blockhash,
        ))
        .await
        .unwrap();

    assert_eq!(
        banks_client
            .get_account(*sign_for_me)
            .await
            .expect("get_sign_for_me")
            .is_none(),
        true
    );

    let balance_post = banks_client
        .get_balance(*withdraw_destination)
        .await
        .unwrap();
    assert!(balance_pre < balance_post); // Redeemed some lamports
}

#[tokio::test]
async fn success() {
    let (mut banks_client, payer, recent_blockhash) = program_test().start().await;

    let owner = Keypair::new();
    let signer = Pubkey::new_unique();
    let scope = Pubkey::new_unique();

    let sign_for_me = create_sign_for_me(
        &mut banks_client,
        &payer,
        &recent_blockhash,
        &owner,
        &signer,
        &scope,
    )
    .await;

    delete_sign_for_me_as_owner(
        &mut banks_client,
        &payer,
        &recent_blockhash,
        &sign_for_me,
        &owner,
        &payer.pubkey(),
    )
    .await;
}
// TODO add negative tests
