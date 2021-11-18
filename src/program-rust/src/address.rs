use solana_program::pubkey::Pubkey;
pub use solana_program;

solana_program::declare_id!("c39Hxxzh7Sh3GgkZM1QzMDyT5Q5cjK5397sbqeBrB1C");


pub fn get_channel_address_and_bump_seed(
    channel_name: &str,  // we should also send organization key,
    program_id: &Pubkey,
) -> (Pubkey, u8) {
    get_channel_address_and_bump_seed_internal(
        channel_name,
        program_id,
    )
}

/// Derives the associated token account address for the given wallet address and token mint
pub fn get_channel_address(
    channel_name: &str,  // we should also send organization key,
    program_id: &Pubkey,

) -> Pubkey {
    get_channel_address_and_bump_seed(channel_name,  program_id).0
}

fn get_channel_address_and_bump_seed_internal(
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