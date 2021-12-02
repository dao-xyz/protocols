use solana_program::{account_info::AccountInfo, pubkey::Pubkey};

use crate::account::create_and_serialize_account_signed;

async fn initialize_mint<'a> (key: &'a Pubkey, programId: &Pubkey, payer_account:&AccountInfo<'a> ) {
    let owner_key = programId;
    let mint = key;
    let decimals = 9;

    /* let rent = banks_client.get_rent().await.unwrap();
    let mint_rent = rent.minimum_balance(Mint::LEN);
    let transaction = Transaction::new_signed_with_payer(
        &[system_instruction::create_account(
            &payer.pubkey(),
            &mint.pubkey(),
            mint_rent,
            Mint::LEN as u64,
            &id(),
        )],
        Some(&payer.pubkey()),
        &[&payer, &mint],
        recent_blockhash,
    );
    banks_client.process_transaction(transaction).await.unwrap(); */

    create_and_serialize_account_signed()

    let transaction = Transaction::new_signed_with_payer(
        &[
            instruction::initialize_mint(&id(), &mint.pubkey(), &owner_key, None, decimals)
                .unwrap(),
        ],
        Some(&payer.pubkey()),
        &[&payer],
        recent_blockhash,
    );
    banks_client.process_transaction(transaction).await.unwrap();
}
