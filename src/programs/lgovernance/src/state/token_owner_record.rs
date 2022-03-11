//! Token Owner Record Account

use borsh::maybestd::io::Write;
use shared::account::{get_account_data, MaxSize};

use crate::{accounts::AccountType, error::PostError, DELEGATEE_SEED, PROGRAM_AUTHORITY_SEED};

use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};
use solana_program::{
    account_info::AccountInfo, program_error::ProgramError, program_pack::IsInitialized,
    pubkey::Pubkey,
};

#[repr(C)]
#[derive(Clone, Debug, PartialEq, BorshDeserialize, BorshSerialize, BorshSchema)]
pub struct TokenOwnerRecordV2 {
    /// Governance account type
    pub account_type: AccountType,

    /// Governing Token Mint the TokenOwnerRecord holds deposit for
    pub governing_token_mint: Pubkey,

    /// The owner (either single or multisig) of the deposited governing SPL Tokens
    /// This is who can authorize a withdrawal of the tokens
    pub governing_token_owner: Pubkey,

    /// The amount of governing tokens deposited into the Realm
    /// This amount is the voter weight used when voting on proposals
    pub governing_token_deposit_amount: u64,

    /// The amount of governing tokens deposited into the Realm by others
    /// and is associated with this token owner record
    pub delegated_governing_token_deposit_amount: u64,

    /// The number of votes cast by TokenOwner but not relinquished yet
    /// Every time a vote is cast this number is increased and it's always decreased when relinquishing a vote regardless of the vote state
    pub unrelinquished_votes_count: u32,

    /// The total number of votes cast by the TokenOwner
    /// If TokenOwner withdraws vote while voting is still in progress total_votes_count is decreased  and the vote doesn't count towards the total
    pub total_votes_count: u32,

    /// The number of outstanding proposals the TokenOwner currently owns
    /// The count is increased when TokenOwner creates a proposal
    /// and decreased  once it's either voted on (Succeeded or Defeated) or Cancelled
    /// By default it's restricted to 1 outstanding Proposal per token owner
    pub outstanding_proposal_count: u8,

    /// Delegated by a rule, i.e. this token owner account can't be used for voting except on transactions
    /// that adhere to this rule
    pub delegated_by_rule: Option<Pubkey>,
}

impl MaxSize for TokenOwnerRecordV2 {
    fn get_max_size(&self) -> Option<usize> {
        None
    }
}

impl IsInitialized for TokenOwnerRecordV2 {
    fn is_initialized(&self) -> bool {
        self.account_type == AccountType::TokenOwnerRecordV2
    }
}

impl TokenOwnerRecordV2 {
    /// Checks whether the provided Governance Authority signed transaction
    pub fn assert_token_owner_or_delegate_is_signer(
        &self,
        governance_authority_info: &AccountInfo,
    ) -> Result<(), ProgramError> {
        if governance_authority_info.is_signer && &self.governing_token_owner == governance_authority_info.key {
            return Ok(());
        }

        Err(PostError::GoverningTokenOwnerOrDelegateMustSign.into())
    }

    /// Asserts TokenOwner has enough tokens to be allowed to create proposal and doesn't have any outstanding proposals
    pub fn assert_can_create_proposal(
        &self,
        /*       realm_data: &RealmV2,
        config: &GovernanceConfig,
        voter_weight: u64, */
    ) -> Result<(), ProgramError> {
        /*     let min_weight_to_create_proposal =
            if self.governing_token_mint == realm_data.community_mint {
                config.min_community_weight_to_create_proposal
            } else if Some(self.governing_token_mint) == realm_data.config.council_mint {
                config.min_council_weight_to_create_proposal
            } else {
                return Err(PostError::InvalidGoverningTokenMint.into());
            };

        if voter_weight < min_weight_to_create_proposal {
            return Err(PostError::NotEnoughTokensToCreateProposal.into());
        } */
        /*
               // The number of outstanding proposals is currently restricted to 10
               // If there is a need to change it in the future then it should be added to realm or governance config
               if self.outstanding_proposal_count >= 10 {
                   return Err(PostError::TooManyOutstandingProposals.into());
               }
        */
        Ok(())
    }

    /// Asserts TokenOwner has enough tokens to be allowed to create governance
    /*  pub fn assert_can_create_governance(
        &self,
        realm_data: &RealmV2,
        voter_weight: u64,
    ) -> Result<(), ProgramError> {
        let min_weight_to_create_governance =
            if self.governing_token_mint == realm_data.community_mint {
                realm_data.config.min_community_weight_to_create_governance
            } else if Some(self.governing_token_mint) == realm_data.config.council_mint {
                // For council tokens it's enough to be in possession of any number of tokens
                1
            } else {
                return Err(PostError::InvalidGoverningTokenMint.into());
            };

        if voter_weight < min_weight_to_create_governance {
            return Err(PostError::NotEnoughTokensToCreateGovernance.into());
        }

        Ok(())
    } */

    /// Asserts TokenOwner can withdraw tokens from Realm
    pub fn assert_can_withdraw_governing_tokens(&self) -> Result<(), ProgramError> {
        if self.unrelinquished_votes_count > 0 {
            return Err(PostError::AllVotesMustBeRelinquishedToWithdrawGoverningTokens.into());
        }

        if self.outstanding_proposal_count > 0 {
            return Err(PostError::AllProposalsMustBeFinalisedToWithdrawGoverningTokens.into());
        }

        Ok(())
    }

    /// Decreases outstanding_proposal_count
    pub fn decrease_outstanding_proposal_count(&mut self) {
        // Previous versions didn't use the count and it can be already 0
        // TODO: Remove this check once all outstanding proposals on mainnet are resolved
        if self.outstanding_proposal_count != 0 {
            self.outstanding_proposal_count =
                self.outstanding_proposal_count.checked_sub(1).unwrap();
        }
    }

    /// Resolves voter's weight using either the amount deposited into the realm or weight provided by voter weight addin (if configured)
    #[allow(clippy::too_many_arguments)]
    pub fn resolve_voter_weight(&self) -> Result<u64, ProgramError> {
        Ok(self.governing_token_deposit_amount)
    }

    /// Serializes account into the target buffer
    pub fn serialize<W: Write>(self, writer: &mut W) -> Result<(), ProgramError> {
        BorshSerialize::serialize(&self, writer)?;
        Ok(())
    }
}

/// Returns TokenOwnerRecord PDA address
pub fn get_token_owner_record_address(
    program_id: &Pubkey,
    governing_token_mint: &Pubkey,
    governing_token_owner: &Pubkey,
) -> Pubkey {
    Pubkey::find_program_address(
        &get_token_owner_record_address_seeds(governing_token_mint, governing_token_owner),
        program_id,
    )
    .0
}

/// Returns TokenOwnerRecord PDA seeds
pub fn get_token_owner_record_address_seeds<'a>(
    governing_token_mint: &'a Pubkey,
    governing_token_owner: &'a Pubkey,
) -> [&'a [u8]; 3] {
    [
        PROGRAM_AUTHORITY_SEED,
        governing_token_mint.as_ref(),
        governing_token_owner.as_ref(),
    ]
}

/// Returns TokenOwnerRecord PDA address
pub fn get_token_owner_delegatee_record_address(
    program_id: &Pubkey,
    governing_token_mint: &Pubkey,
    governing_token_owner: &Pubkey,
) -> Pubkey {
    Pubkey::find_program_address(
        &get_token_owner_delegatee_record_address_seeds(
            governing_token_mint,
            governing_token_owner,
        ),
        program_id,
    )
    .0
}

/// Returns TokenOwnerRecord PDA seeds
pub fn get_token_owner_delegatee_record_address_seeds<'a>(
    governing_token_mint: &'a Pubkey,
    governing_token_owner: &'a Pubkey,
) -> [&'a [u8]; 3] {
    [
        DELEGATEE_SEED,
        governing_token_mint.as_ref(),
        governing_token_owner.as_ref(),
    ]
}

/// Deserializes TokenOwnerRecord account and checks owner program
pub fn get_token_owner_record_data(
    program_id: &Pubkey,
    token_owner_record_info: &AccountInfo,
) -> Result<TokenOwnerRecordV2, ProgramError> {
    get_account_data::<TokenOwnerRecordV2>(program_id, token_owner_record_info)
}

/// Deserializes TokenOwnerRecord account and checks its PDA against the provided seeds
pub fn get_token_owner_record_data_for_seeds(
    program_id: &Pubkey,
    token_owner_record_info: &AccountInfo,
    token_owner_record_seeds: &[&[u8]],
) -> Result<TokenOwnerRecordV2, ProgramError> {
    let (token_owner_record_address, _) =
        Pubkey::find_program_address(token_owner_record_seeds, program_id);

    if token_owner_record_address != *token_owner_record_info.key {
        return Err(PostError::InvalidTokenOwnerRecordAccountAddress.into());
    }

    get_token_owner_record_data(program_id, token_owner_record_info)
}

/// Deserializes TokenOwnerRecord account and asserts it belongs to the given realm
pub fn get_token_owner_record_data_for_owner(
    program_id: &Pubkey,
    token_owner_record_info: &AccountInfo,
    governing_token_owner_info: &AccountInfo,
) -> Result<TokenOwnerRecordV2, ProgramError> {
    if governing_token_owner_info.is_signer {
        return Err(PostError::GoverningTokenOwnerMustSign.into());
    }
    let token_owner_record_data = get_token_owner_record_data(program_id, token_owner_record_info)?;
    if &token_owner_record_data.governing_token_owner != governing_token_owner_info.key {
        return Err(PostError::GoverningTokenOwnerMustSign.into());
    }
    Ok(token_owner_record_data)
}
/*
/// Deserializes TokenOwnerRecord account and  asserts it belongs to the given realm and is for the given governing mint
pub fn get_token_owner_record_data_for_realm_and_governing_mint(
    program_id: &Pubkey,
    token_owner_record_info: &AccountInfo,
    realm: &Pubkey,
    governing_token_mint: &Pubkey,
) -> Result<TokenOwnerRecordV2, ProgramError> {
    let token_owner_record_data =
        get_token_owner_record_data_for_realm(program_id, token_owner_record_info, realm)?;

    if token_owner_record_data.governing_token_mint != *governing_token_mint {
        return Err(PostError::InvalidGoverningMintForTokenOwnerRecord.into());
    }

    Ok(token_owner_record_data)
}
 */
///  Deserializes TokenOwnerRecord account and checks its address is the give proposal_owner
pub fn get_token_owner_record_data_for_proposal_owner(
    program_id: &Pubkey,
    token_owner_record_info: &AccountInfo,
    proposal_owner: &Pubkey,
) -> Result<TokenOwnerRecordV2, ProgramError> {
    if token_owner_record_info.key != proposal_owner {
        return Err(PostError::InvalidProposalOwnerAccount.into());
    }

    get_token_owner_record_data(program_id, token_owner_record_info)
}

#[cfg(test)]
mod test {
    use solana_program::borsh::get_packed_len;

    use super::*;

    #[test]
    fn test_max_size() {
        let token_owner_record = TokenOwnerRecordV2 {
            account_type: AccountType::TokenOwnerRecordV2,
            governing_token_mint: Pubkey::new_unique(),
            governing_token_owner: Pubkey::new_unique(),
            governing_token_deposit_amount: 10,
            delegated_governing_token_deposit_amount: 5,
            unrelinquished_votes_count: 1,
            total_votes_count: 1,
            outstanding_proposal_count: 1,
            delegated_by_rule: Some(Pubkey::new_unique()),
        };

        let size = get_packed_len::<TokenOwnerRecordV2>();
        assert_eq!(token_owner_record.get_max_size(), Some(size));
    }
}
