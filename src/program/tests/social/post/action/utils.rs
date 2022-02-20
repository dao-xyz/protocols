use std::time::{SystemTime, UNIX_EPOCH};

use s2g::socials::post::{
    instruction::create_post_execution_transaction,
    state::{deserialize_post_account, ActionStatus, ActionType, PostType},
};
use solana_program::{hash::Hash, pubkey::Pubkey, system_instruction};
use solana_program_test::BanksClient;
use solana_sdk::{
    signature::Keypair, signer::Signer, transaction::Transaction, transport::TransportError,
};
use spl_associated_token_account::{create_associated_token_account, get_associated_token_address};

use crate::{
    social::post::utils::{SocialAccounts, TestPost},
    stake_pool::helpers::create_mint_from_keypair,
};
pub fn time_since_epoch() -> u64 {
    let start = SystemTime::now();
    let since_the_epoch = start.duration_since(UNIX_EPOCH).unwrap();
    return since_the_epoch.as_secs();
}

pub async fn create_governence_token_and_supply_payer(
    banks_client: &mut BanksClient,
    payer: &Keypair,
    recent_blockhash: &Hash,
    amount: u64,
) -> Keypair {
    let mint = Keypair::new();
    let mint_pubkey = mint.pubkey();
    create_mint_from_keypair(
        banks_client,
        &payer,
        recent_blockhash,
        &mint,
        &payer.pubkey(),
    )
    .await
    .unwrap();

    let mut create_associated_token_account_transaction = Transaction::new_with_payer(
        &[create_associated_token_account(
            &payer.pubkey(),
            &payer.pubkey(),
            &mint_pubkey,
        )],
        Some(&payer.pubkey()),
    );

    create_associated_token_account_transaction.sign(&[payer], *recent_blockhash);
    banks_client
        .process_transaction(create_associated_token_account_transaction)
        .await
        .unwrap();

    let mut token_mint_transaction = Transaction::new_with_payer(
        &[spl_token::instruction::mint_to(
            &spl_token::id(),
            &mint_pubkey,
            &get_associated_token_address(&payer.pubkey(), &mint_pubkey),
            &payer.pubkey(),
            &[&payer.pubkey()],
            amount,
        )
        .unwrap()],
        Some(&payer.pubkey()),
    );

    token_mint_transaction.sign(&[payer], *recent_blockhash);
    banks_client
        .process_transaction(token_mint_transaction)
        .await
        .unwrap();

    mint
}

pub async fn execute_post(
    banks_client: &mut BanksClient,
    payer: &Keypair,
    recent_blockhash: &Hash,
    socials: &SocialAccounts,
    post: &TestPost,
) -> Result<(), TransportError> {
    let mut execute_post = Transaction::new_with_payer(
        &[create_post_execution_transaction(
            &s2g::id(),
            &payer.pubkey(),
            &post.post,
            &post.get_post_account(banks_client).await,
            &socials.governence_mint,
        )],
        Some(&payer.pubkey()),
    );
    execute_post.sign(&[payer], *recent_blockhash);
    banks_client.process_transaction(execute_post).await
}

pub async fn assert_action_status(
    banks_client: &mut BanksClient,
    post: &Pubkey,
    status: &ActionStatus,
) {
    let account =
        deserialize_post_account(&*banks_client.get_account(*post).await.unwrap().unwrap().data)
            .unwrap();
    if let PostType::ActionPost(post) = account.post_type {
        assert_eq!(&post.status, status);
    } else {
        panic!("Unexpected");
    }
}
