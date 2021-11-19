use solana_program::pubkey::Pubkey;
pub use solana_program;

solana_program::declare_id!("c39Hxxzh7Sh3GgkZM1QzMDyT5Q5cjK5397sbqeBrB1C");



pub fn get_channel_account_address_and_bump_seed(
    channel_name: &str,  // we should also send organization key,
    program_id: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            &channel_name.as_bytes()
        ],
        program_id,
    )
}

pub fn get_message_account_address_and_bump_seed(
    payer_account: &Pubkey,  // payer_account == from
    program_id: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            &payer_account.to_bytes()
        ],
        program_id,
    )
}