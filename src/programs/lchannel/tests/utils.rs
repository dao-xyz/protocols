use solana_program::pubkey::Pubkey;
use solana_program_test::*;

use lchannel::processor::Processor;
use solana_program::hash::Hash;
use solana_sdk::{signature::Keypair, signer::Signer, transaction::Transaction};

pub fn program_test() -> ProgramTest {
    let mut program = ProgramTest::new("lchannel", lchannel::id(), processor!(Processor::process));
    /*   program.add_program(
        "luser",
        luser::id(),
        processor!(luser::processor::Processor::process),
    ); */
    program
}
/*
pub async fn create_and_verify_user(
    banks_client: &mut BanksClient,
    payer: &Keypair,
    recent_blockhash: &Hash,
    username: &str,
    profile: &str,
) -> Pubkey {
    // Create user
    banks_client
        .process_transaction(Transaction::new_signed_with_payer(
            &[create_user_transaction(
                &luser::id(),
                username,
                Some(profile.into()),
                &payer.pubkey(),
            )],
            Some(&payer.pubkey()),
            &[payer],
            *recent_blockhash,
        ))
        .await
        .unwrap();

    // Verify username name
    let user_account_address = find_user_account_program_address(&luser::id(), username).0;
    let user_account_info = banks_client
        .get_account(user_account_address)
        .await
        .expect("get_user")
        .expect("user not found");
    let user = deserialize_user_account(&user_account_info.data).unwrap();
    assert_eq!(user.name, username);
    user_account_address
}
 */
