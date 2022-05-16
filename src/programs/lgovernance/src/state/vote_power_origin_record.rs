use shared::account::{create_and_serialize_account_verify_with_bump, get_account_data, MaxSize};

use crate::{
    accounts::AccountType, error::GovernanceError, state::scopes::scope::VotePowerUnit,
    PROGRAM_AUTHORITY_SEED,
};

use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};
use solana_program::{
    account_info::AccountInfo, program_error::ProgramError, program_pack::IsInitialized,
    pubkey::Pubkey, rent::Rent,
};
/*
#[repr(C)]
#[derive(Clone, Debug, PartialEq, BorshDeserialize, BorshSerialize, BorshSchema)]
pub enum VotePowerSource {
    Token {
        /// Governing Token Mint the TokenOwnerRecord holds deposit for
        governing_token_mint: Pubkey,

        /// The amount of governing tokens deposited into the Realm
        /// This amount is the voter weight used when voting on proposals
        governing_token_deposit_amount: u64,
    },

    Tag {
        // Tag record issuer
        record_factory: Pubkey,

        // Amount
        amount: u64,
    },
} */
/*
impl From<&VoteSource> for VotePowerSource {
    fn from(vote_power_unit: &VoteSource) -> Self {
        match vote_power_unit {
            VoteSource::Mint(mint) => VotePowerSource::Token {
                governing_token_mint: *mint,
                governing_token_deposit_amount: 0,
            },
            VoteSource::Tag { record_factory } => VotePowerSource::Tag {
                record_factory: *record_factory,
                amount: 0,
            },
        }
    }
} */

/*
#[repr(C)]
#[derive(Clone, Debug, PartialEq, BorshDeserialize, BorshSerialize, BorshSchema)]
pub enum VotePowerSourceOrigin {
    Token {
        /// Governing Token Mint the TokenOwnerRecord holds deposit for
        governing_token_mint: Pubkey,

        /// The amount of governing tokens deposited into the Realm
        /// This amount is the voter weight used when voting on proposals
        governing_token_deposit_amount: u64,

        /// The amount of governing available for delegation
        governing_token_available: u64,
    },

    Tag {
        // Tag record issuer
        record_factory: Pubkey,
    },
} */

#[repr(C)]
#[derive(Clone, Debug, PartialEq, BorshDeserialize, BorshSerialize, BorshSchema)]
pub struct VotePowerOriginRecord {
    /// Governance account type
    pub account_type: AccountType,

    /// Source type of "power"
    pub source: VotePowerUnit,

    /// Amount
    pub amount: u64,

    /// The owner (either single or multisig) of the deposited governing SPL Tokens
    /// This is who can authorize a withdrawal of the tokens
    pub governing_owner: Pubkey,
}

impl MaxSize for VotePowerOriginRecord {
    fn get_max_size(&self) -> Option<usize> {
        Some(1 + 1 + 32 + 8 + 32)
    }
}

impl IsInitialized for VotePowerOriginRecord {
    fn is_initialized(&self) -> bool {
        self.account_type == AccountType::VotePowerOriginRecord
    }
}

impl VotePowerOriginRecord {
    pub fn create<'a>(
        program_id: &Pubkey,
        source: VotePowerUnit,
        amount: u64,
        rent: &Rent,
        token_origin_record_info: &AccountInfo<'a>,
        token_origin_record_bump_seed: u8,
        governing_owner_info: &AccountInfo<'a>,
        // governing_token_mint: &Pubkey,
        payer_info: &AccountInfo<'a>,
        system_info: &AccountInfo<'a>,
    ) -> Result<(), ProgramError> {
        let bump_seeds = [token_origin_record_bump_seed];
        let vote_power_owner_record_address_seeds = get_vote_power_origin_record_address_seeds(
            &source,
            governing_owner_info.key,
            &bump_seeds,
        );

        if token_origin_record_info.data_is_empty() {
            if !(governing_owner_info.is_signer) {
                return Err(GovernanceError::GoverningTokenOwnerMustSign.into());
            }

            let token_owner_record_data = VotePowerOriginRecord {
                account_type: AccountType::VotePowerOriginRecord,
                governing_owner: *governing_owner_info.key,
                source: source.clone(),
                amount,
            };

            create_and_serialize_account_verify_with_bump(
                payer_info,
                token_origin_record_info,
                &token_owner_record_data,
                &vote_power_owner_record_address_seeds,
                program_id,
                system_info,
                rent,
            )?;
        } else {
            return Err(GovernanceError::VotePowerOriginRecordAlreadyExist.into());
        }
        Ok(())
    }
}

/// Returns TokenOwnerRecord PDA address
pub fn get_vote_power_origin_record_address(
    program_id: &Pubkey,
    source: &VotePowerUnit,
    governing_owner: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &match source {
            VotePowerUnit::Mint(governing_token_mint) => [
                PROGRAM_AUTHORITY_SEED,
                governing_token_mint.as_ref(),
                governing_owner.as_ref(),
            ],
            VotePowerUnit::Tag { record_factory } => [
                PROGRAM_AUTHORITY_SEED,
                record_factory.as_ref(),
                governing_owner.as_ref(),
            ],
        },
        program_id,
    )
}

/// Returns TokenOwnerRecord PDA seeds
pub fn get_vote_power_origin_record_address_seeds<'a>(
    //  governing_token_mint: &'a Pubkey,
    source: &'a VotePowerUnit,
    governing_owner: &'a Pubkey,
    bump_seed: &'a [u8],
) -> [&'a [u8]; 4] {
    match source {
        VotePowerUnit::Mint(governing_token_mint) => [
            PROGRAM_AUTHORITY_SEED,
            governing_token_mint.as_ref(),
            governing_owner.as_ref(),
            bump_seed,
        ],
        VotePowerUnit::Tag { record_factory } => [
            PROGRAM_AUTHORITY_SEED,
            record_factory.as_ref(),
            governing_owner.as_ref(),
            bump_seed,
        ],
    }
}

pub fn get_vote_power_origin_record_data(
    program_id: &Pubkey,
    vote_power_origin_record_info: &AccountInfo,
) -> Result<VotePowerOriginRecord, ProgramError> {
    get_account_data::<VotePowerOriginRecord>(program_id, vote_power_origin_record_info)
}

pub fn get_vote_power_origin_record_data_for_owner(
    program_id: &Pubkey,
    vote_power_origin_record_info: &AccountInfo,
    governing_owner_info: &AccountInfo,
) -> Result<VotePowerOriginRecord, ProgramError> {
    if !governing_owner_info.is_signer {
        return Err(GovernanceError::GoverningTokenOwnerMustSign.into());
    }

    let vote_power_origin_record_data =
        get_vote_power_origin_record_data(program_id, vote_power_origin_record_info)?;
    if &vote_power_origin_record_data.governing_owner != governing_owner_info.key {
        return Err(GovernanceError::InvalidTokenOwner.into());
    }
    Ok(vote_power_origin_record_data)
}
