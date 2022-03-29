//! Governance Account
use borsh::maybestd::io::Write;
use shared::account::{get_account_data, MaxSize};

use crate::{accounts::AccountType, state::enums::VoteTipping};
use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};
use solana_program::{
    account_info::AccountInfo, borsh::try_from_slice_unchecked, program_error::ProgramError,
    program_pack::IsInitialized, pubkey::Pubkey,
};

/// Governance Account
#[repr(C)]
#[derive(Clone, Debug, PartialEq, BorshDeserialize, BorshSerialize, BorshSchema)]
pub struct GovernanceV2 {
    /// Account type. It can be Uninitialized, Governance, ProgramGovernance, TokenGovernance or MintGovernance
    pub account_type: AccountType,

    /// Goverannce associated channel
    pub channel: Pubkey,

    /// Account governed by this Governance and/or PDA identity seed
    /// It can be Program account, Mint account, Token account or any other account
    ///
    /// Note: The account doesn't have to exist. In that case the field is only a PDA seed
    ///
    /// Note: Setting governed_account doesn't give any authority over the governed account
    /// The relevant authorities for specific account types must still be transferred to the Governance PDA
    /// Ex: mint_authority/freeze_authority for a Mint account
    /// or upgrade_authority for a Program account should be transferred to the Governance PDA
    // pub governed_account: Pubkey,

    /// Running count of proposals
    pub proposals_count: u64,

    /// The number of proposals in voting state in the Governance
    pub voting_proposal_count: u32,
}

impl MaxSize for GovernanceV2 {}

impl IsInitialized for GovernanceV2 {
    fn is_initialized(&self) -> bool {
        self.account_type == AccountType::Governance
    }
}

/// Deserializes Governance account and checks owner program
pub fn get_governance_data(
    program_id: &Pubkey,
    governance_info: &AccountInfo,
) -> Result<GovernanceV2, ProgramError> {
    get_account_data::<GovernanceV2>(program_id, governance_info)
}
/// Returns Governance PDA seeds
pub fn get_governance_address_seeds<'a>(
    channel: &'a Pubkey,
    // governed_account: &'a Pubkey,
    bump_seed: &'a [u8],
) -> [&'a [u8]; 3] {
    [
        b"account-governance",
        channel.as_ref(),
        // governed_account.as_ref(),
        bump_seed,
    ]
}

/// Returns Governance PDA address
pub fn get_governance_address<'a>(
    program_id: &Pubkey,
    channel: &Pubkey,
    //  governed_account: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            b"account-governance",
            channel.as_ref(),
            // governed_account.as_ref(),
        ],
        program_id,
    )
}
