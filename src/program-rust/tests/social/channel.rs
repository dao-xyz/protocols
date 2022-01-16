/*
async fn create_and_verify_channel(
    banks_client: &mut BanksClient,
    payer: &Keypair,
    recent_blockhash: &Hash,
    program_id: &Pubkey,
    user_account: &Pubkey,
) -> Pubkey {
    let channel_name = "My channel";
    let (channel_address_pda, _bump) = find_channel_program_address(program_id, channel_name);

    let mut transaction_create = Transaction::new_with_payer(
        &[create_channel_transaction(
            &id(),
            channel_name,
            &payer.pubkey(),
            user_account,
        )],
        Some(&payer.pubkey()),
    );
    transaction_create.sign(&[payer], *recent_blockhash);
    banks_client
        .process_transaction(transaction_create)
        .await
        .unwrap();

    // Verify channel name
    let channel_account_info = banks_client
        .get_account(channel_address_pda)
        .await
        .expect("get_account")
        .expect("channel_account not found");
    let channel_account = deserialize_channel_account(&channel_account_info.data);

    assert_eq!(channel_account.name.as_str(), channel_name);
    channel_address_pda
} */
