use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};
use solana_program::pubkey::Pubkey;

// Vote weights for a particular mint
#[derive(Clone, Debug, PartialEq, BorshDeserialize, BorshSerialize, BorshSchema)]
pub struct _VoteWeight {
    pub vote_mint: Pubkey,
    pub vote_weight: u64,
}
