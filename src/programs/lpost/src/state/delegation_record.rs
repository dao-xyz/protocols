use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};
use shared::account::MaxSize;
use solana_program::pubkey::Pubkey;

use crate::accounts::AccountType;

#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, BorshSchema, PartialEq)]
pub struct DelegationRcord {
    pub from: Pubkey,
    pub mint: Pubkey,
    pub amount: u64,
}

impl MaxSize for DelegationRcord {
    fn get_max_size(&self) -> Option<usize> {
        None
    }
}

pub fn find_delegation_record_program_address(
    program_id: &Pubkey,
    from: &Pubkey,
    mint: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[from.as_ref(), mint.as_ref()], program_id)
}
pub fn create_delegation_record_program_address_seeds<'a>(
    program_id: &Pubkey,
    from: &'a Pubkey,
    mint: &'a Pubkey,
    bump_seed: &'a [u8],
) -> [&'a [u8]; 3] {
    return [from.as_ref(), mint.as_ref(), bump_seed];
}
