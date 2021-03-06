//! Token Owner Record Account
use borsh::maybestd::io::Write;
use shared::account::{create_and_serialize_account_verify_with_bump, get_account_data, MaxSize};

use crate::{
    accounts::AccountType, error::GovernanceError, state::scopes::scope::VotePowerUnit,
    DELEGATEE_SEED,
};

use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};
use solana_program::{
    account_info::AccountInfo, msg, program_error::ProgramError, program_pack::IsInitialized,
    pubkey::Pubkey, rent::Rent,
};

#[repr(C)]
#[derive(Clone, Debug, PartialEq, BorshDeserialize, BorshSerialize, BorshSchema)]
pub struct VotePowerOwnerRecord {
    /// Governance account type
    pub account_type: AccountType,

    pub source: VotePowerUnit,

    pub amount: u64,

    /// The owner (either single or multisig) of the deposited governing SPL Tokens
    /// This is who can authorize a withdrawal of the tokens
    pub governing_owner: Pubkey,

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

    /// Delegated by a scope, i.e. this token owner account can't be used for voting except on transactions
    /// that adhere to this scope
    pub delegated_by_scope: Pubkey,

    /// Latest vote using the token owner record
    pub first_vote: Option<Pubkey>,

    /// Latest vote using the token owner record
    pub latest_vote: Option<Pubkey>,
}

impl MaxSize for VotePowerOwnerRecord {
    fn get_max_size(&self) -> Option<usize> {
        Some(1 + 32 + 32 + 8 + 8 + 4 + 4 + 1 + 1 + 32 + 1 + 32 + 1 + 32)
    }
}

impl IsInitialized for VotePowerOwnerRecord {
    fn is_initialized(&self) -> bool {
        self.account_type == AccountType::VotePowerOwnerRecord
    }
}

impl VotePowerOwnerRecord {
    pub fn create_empty_delegate<'a>(
        program_id: &Pubkey,
        delegated_by_scope: &Pubkey,
        rent: &Rent,
        vote_power_owner_record_info: &AccountInfo<'a>,
        token_owner_record_bump_seed: u8,
        governing_owner_info: &AccountInfo<'a>,
        source: &VotePowerUnit,
        payer_info: &AccountInfo<'a>,
        system_info: &AccountInfo<'a>,
    ) -> Result<(), ProgramError> {
        let bump_seeds = [token_owner_record_bump_seed];

        let vote_power_owner_record_address_seeds = get_vote_power_owner_record_address_seeds(
            source,
            governing_owner_info.key,
            delegated_by_scope,
            &bump_seeds,
        );

        if vote_power_owner_record_info.data_is_empty() {
            if !(governing_owner_info.is_signer) {
                return Err(GovernanceError::GoverningTokenOwnerMustSign.into());
            }

            msg!(
                "CREATE DELEGATE WITH ID: {}",
                vote_power_owner_record_info.key
            );
            let token_owner_record_data = VotePowerOwnerRecord {
                account_type: AccountType::VotePowerOwnerRecord,
                governing_owner: *governing_owner_info.key,
                /*     governing_token_deposit_amount: 0,
                governing_token_mint: *governing_token_mint, */
                source: source.clone(),
                unrelinquished_votes_count: 0,
                amount: 0,
                total_votes_count: 0,
                outstanding_proposal_count: 0,
                delegated_by_scope: *delegated_by_scope, // this is not a delegation
                first_vote: None,
                latest_vote: None,
            };

            create_and_serialize_account_verify_with_bump(
                payer_info,
                vote_power_owner_record_info,
                &token_owner_record_data,
                &vote_power_owner_record_address_seeds,
                program_id,
                system_info,
                rent,
            )?;
        } else {
            return Err(GovernanceError::TokenOwnerRecordAlreadyExists.into());
        }
        Ok(())
    }
    /*
    pub fn add_amount<'a>(
        program_id: &Pubkey,
        vote_power_owner_record_info: &AccountInfo<'a>,
        governing_owner: &Pubkey,
        governing_token_mint: &Pubkey,
        delegated_by_scope: Option<&Pubkey>,
        add_amount: u64,
    ) -> Result<(), ProgramError> {
        let mut token_owner_record_data = get_vote_power_owner_record_data_for_delegation_activity(
            program_id,
            vote_power_owner_record_info,
            governing_owner,
            governing_token_mint,
            delegated_by_scope,
        )?;

        token_owner_record_data.governing_token_deposit_amount = token_owner_record_data
            .governing_token_deposit_amount
            .checked_add(add_amount)
            .unwrap();

        token_owner_record_data.serialize(&mut *vote_power_owner_record_info.data.borrow_mut())?;
        Ok(())
    }

    pub fn subtract_amount<'a>(
        program_id: &Pubkey,
        vote_power_owner_record_info: &AccountInfo<'a>,
        governing_owner: &Pubkey,
        governing_token_mint: &Pubkey,
        delegated_by_scope: Option<&Pubkey>,
        add_amount: u64,
    ) -> Result<(), ProgramError> {
        let mut token_owner_record_data = get_vote_power_owner_record_data_for_delegation_activity(
            program_id,
            vote_power_owner_record_info,
            governing_owner,
            governing_token_mint,
            delegated_by_scope,
        )?;

        token_owner_record_data.governing_token_deposit_amount = token_owner_record_data
            .governing_token_deposit_amount
            .checked_sub(add_amount)
            .unwrap();

        token_owner_record_data.serialize(&mut *vote_power_owner_record_info.data.borrow_mut())?;
        Ok(())
    } */

    /// Checks whether the provided Governance Authority signed transaction
    pub fn assert_token_owner_or_delegate_is_signer(
        &self,
        governance_authority_info: &AccountInfo,
    ) -> Result<(), ProgramError> {
        if governance_authority_info.is_signer
            && &self.governing_owner == governance_authority_info.key
        {
            return Ok(());
        }

        Err(GovernanceError::GoverningTokenOwnerOrDelegateMustSign.into())
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
            return Err(
                GovernanceError::AllVotesMustBeRelinquishedToWithdrawGoverningTokens.into(),
            );
        }

        if self.outstanding_proposal_count > 0 {
            return Err(
                GovernanceError::AllProposalsMustBeFinalisedToWithdrawGoverningTokens.into(),
            );
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
    /*     #[allow(clippy::too_many_arguments)]
    pub fn resolve_voter_weight(&self) -> Result<u64, ProgramError> {
        Ok(self.governing_token_deposit_amount)
    } */

    /// Serializes account into the target buffer
    pub fn serialize<W: Write>(self, writer: &mut W) -> Result<(), ProgramError> {
        BorshSerialize::serialize(&self, writer)?;
        Ok(())
    }
}

/// Returns TokenOwnerRecord PDA address
pub fn get_vote_power_owner_record_address(
    program_id: &Pubkey,
    /*  governing_token_mint: &Pubkey, */
    source: &VotePowerUnit,
    governing_owner: &Pubkey,
    scope: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &match source {
            VotePowerUnit::Mint(governing_token_mint) => [
                DELEGATEE_SEED,
                scope.as_ref(),
                governing_token_mint.as_ref(),
                governing_owner.as_ref(),
            ],
            VotePowerUnit::Tag { record_factory } => [
                DELEGATEE_SEED,
                scope.as_ref(),
                record_factory.as_ref(),
                governing_owner.as_ref(),
            ],
        },
        program_id,
    )
}

/// Returns TokenOwnerRecord PDA seeds
pub fn get_vote_power_owner_record_address_seeds<'a>(
    source: &'a VotePowerUnit,
    governing_owner: &'a Pubkey,
    scope: &'a Pubkey,
    bump_seed: &'a [u8],
) -> [&'a [u8]; 5] {
    match source {
        VotePowerUnit::Mint(governing_token_mint) => [
            DELEGATEE_SEED,
            scope.as_ref(),
            governing_token_mint.as_ref(),
            governing_owner.as_ref(),
            bump_seed,
        ],
        VotePowerUnit::Tag { record_factory, .. } => [
            DELEGATEE_SEED,
            scope.as_ref(),
            record_factory.as_ref(),
            governing_owner.as_ref(),
            bump_seed,
        ],
    }
}

/// Deserializes TokenOwnerRecord account and checks owner program
pub fn get_vote_power_owner_record_data(
    program_id: &Pubkey,
    vote_power_owner_record_info: &AccountInfo,
) -> Result<VotePowerOwnerRecord, ProgramError> {
    get_account_data::<VotePowerOwnerRecord>(program_id, vote_power_owner_record_info)
}

pub fn get_vote_power_owner_record_data_for_delegation_activity(
    program_id: &Pubkey,
    vote_power_owner_record_info: &AccountInfo,
    governing_owner: &Pubkey,
    source: &VotePowerUnit,
    delegated_by_scope: &Pubkey,
) -> Result<VotePowerOwnerRecord, ProgramError> {
    let token_owner_record_data =
        get_vote_power_owner_record_data(program_id, vote_power_owner_record_info)?;

    token_owner_record_data.source.assert_compatible(source)?;

    if &token_owner_record_data.governing_owner != governing_owner {
        return Err(GovernanceError::InvalidGoverningTokenOwnerForVoteRecord.into());
    }
    /*   if match (
        &token_owner_record_data.delegated_by_scope,
        delegated_by_scope,
    ) {
        (Some(a), Some(b)) => a != b,
        (None, None) => true,
        _ => false,
    } {
        return Err(GovernanceError::InvalidScopeVoteRecord.into());
    } */

    if &token_owner_record_data.delegated_by_scope != delegated_by_scope {
        return Err(GovernanceError::InvalidScopeVoteRecord.into());
    }

    Ok(token_owner_record_data)
}
/*
pub fn get_vote_power_owner_record_data_for_delegation(
    program_id: &Pubkey,
    delegatee_vote_power_owner_record_info: &AccountInfo,
    scope_delegation_record: &ScopeDelegationRecordAccount,
    delegator_governing_owner: &AccountInfo,
) -> Result<VotePowerOwnerRecord, ProgramError> {
    if !delegator_governing_owner.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let token_owner_record_data =
        get_vote_power_owner_record_data(program_id, delegatee_vote_power_owner_record_info)?;

    if &token_owner_record_data.governing_token_mint != governing_token_mint {
        return Err(GovernanceError::InvalidGoverningMintForTokenOwnerRecord.into());
    }
    if &token_owner_record_data.governing_owner != governing_owner {
        return Err(GovernanceError::InvalidGoverningTokenOwnerForVoteRecord.into());
    }
    if match (
        &token_owner_record_data.delegated_by_scope,
        delegated_by_scope,
    ) {
        (Some(a), Some(b)) => a != b,
        (None, None) => true,
        _ => false,
    } {
        return Err(GovernanceError::InvalidScopeVoteRecord.into());
    }
    Ok(token_owner_record_data)
} */

/// Deserializes TokenOwnerRecord account and checks its PDA against the provided seeds
pub fn get_vote_power_owner_record_data_for_seeds(
    program_id: &Pubkey,
    vote_power_owner_record_info: &AccountInfo,
    vote_power_owner_record_seeds: &[&[u8]],
) -> Result<VotePowerOwnerRecord, ProgramError> {
    let (vote_power_owner_record_address, _) =
        Pubkey::find_program_address(vote_power_owner_record_seeds, program_id);

    if vote_power_owner_record_address != *vote_power_owner_record_info.key {
        return Err(GovernanceError::InvalidTokenOwnerRecordAccountAddress.into());
    }

    get_vote_power_owner_record_data(program_id, vote_power_owner_record_info)
}

/// Deserializes TokenOwnerRecord account and asserts it belongs to the given governing token owner
pub fn get_vote_power_owner_record_data_for_owner(
    program_id: &Pubkey,
    vote_power_owner_record_info: &AccountInfo,
    governing_owner_info: &AccountInfo,
) -> Result<VotePowerOwnerRecord, ProgramError> {
    if !governing_owner_info.is_signer {
        return Err(GovernanceError::GoverningTokenOwnerMustSign.into());
    }

    let token_owner_record_data =
        get_vote_power_owner_record_data(program_id, vote_power_owner_record_info)?;
    if &token_owner_record_data.governing_owner != governing_owner_info.key {
        return Err(GovernanceError::InvalidTokenOwner.into());
    }
    Ok(token_owner_record_data)
}

/*
/// Deserializes TokenOwnerRecord account and  asserts it belongs to the given realm and is for the given governing mint
pub fn get_vote_power_owner_record_data_for_realm_and_governing_mint(
    program_id: &Pubkey,
    vote_power_owner_record_info: &AccountInfo,
    realm: &Pubkey,
    governing_token_mint: &Pubkey,
) -> Result<VotePowerOwnerRecord, ProgramError> {
    let token_owner_record_data =
        get_vote_power_owner_record_data_for_realm(program_id, vote_power_owner_record_info, realm)?;

    if token_owner_record_data.governing_token_mint != *governing_token_mint {
        return Err(PostError::InvalidGoverningMintForTokenOwnerRecord.into());
    }

    Ok(token_owner_record_data)
}
 */
///  Deserializes TokenOwnerRecord account and checks its address is the give proposal_owner
pub fn get_vote_power_owner_record_data_for_proposal_owner(
    program_id: &Pubkey,
    vote_power_owner_record_info: &AccountInfo,
    proposal_owner: &Pubkey,
) -> Result<VotePowerOwnerRecord, ProgramError> {
    if vote_power_owner_record_info.key != proposal_owner {
        return Err(GovernanceError::InvalidProposalOwnerAccount.into());
    }

    get_vote_power_owner_record_data(program_id, vote_power_owner_record_info)
}

#[cfg(test)]
mod test {

    /*
      #[test]
    fn test_max_size() {
         let token_owner_record = VotePowerOwnerRecord {
             account_type: AccountType::VotePowerOwnerRecord,
             governing_token_mint: Pubkey::new_unique(),
             governing_owner: Pubkey::new_unique(),
             governing_token_deposit_amount: 10,
             unrelinquished_votes_count: 1,
             total_votes_count: 1,
             outstanding_proposal_count: 1,
             delegated_by_scope: Some(Pubkey::new_unique()),
             first_vote: None,
             latest_vote: None,
         };

         let size = get_packed_len::<VotePowerOwnerRecord>();
         assert_eq!(token_owner_record.get_max_size(), Some(size));
     } */
}
