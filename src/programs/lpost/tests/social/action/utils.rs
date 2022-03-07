use std::time::{SystemTime, UNIX_EPOCH};

use lpost::{
    instruction::create_post_execution_transaction,
    state::post::{deserialize_post_account, ActionStatus, PostType},
};

use solana_program::{hash::Hash, program_pack::Pack, pubkey::Pubkey, system_instruction};
use solana_program_test::BanksClient;
use solana_sdk::{
    signature::Keypair, signer::Signer, transaction::Transaction, transport::TransportError,
};

use crate::social::utils::{TestChannel, TestPost};

pub fn time_since_epoch() -> u64 {
    let start = SystemTime::now();
    let since_the_epoch = start.duration_since(UNIX_EPOCH).unwrap();
    since_the_epoch.as_secs()
}

pub async fn execute_post(
    banks_client: &mut BanksClient,
    payer: &Keypair,
    recent_blockhash: &Hash,
    channel: &TestChannel,
    post: &TestPost,
) -> Result<(), TransportError> {
    let mut execute_post = Transaction::new_with_payer(
        &[create_post_execution_transaction(
            &lpost::id(),
            &payer.pubkey(),
            &post.post,
            &post.get_post_account(banks_client).await,
            &channel.mint,
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
