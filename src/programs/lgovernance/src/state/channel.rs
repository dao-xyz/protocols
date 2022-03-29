use solana_program::pubkey::Pubkey;

pub struct ChannelSigner {
    pub authority: Pubkey,
    pub channel_path: Vec<Pubkey>,
}
