use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};
use shared::account::{
    create_and_serialize_account_verify_with_bump, dispose_account, get_account_data, MaxSize,
};
use solana_program::{
    account_info::AccountInfo, msg, program_error::ProgramError, program_pack::IsInitialized,
    pubkey::Pubkey, rent::Rent,
};

use crate::{
    accounts::AccountType,
    error::GovernanceError,
    state::{
        vote_power_origin_record::VotePowerOriginRecord,
        vote_power_owner_record::VotePowerOwnerRecord,
    },
};

#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, BorshSchema, PartialEq)]
pub struct ScopeDelegationRecordAccount {
    pub account_type: AccountType,
    pub delegator_token_origin_record: Pubkey,
    pub delegatee_token_owner_record: Pubkey,
    pub amount: u64,

    /// At what vote "index" is this delegation used for.
    /// Empty, means that no votes had happened for the delegatee_token_owner_record
    /// Non empty value means that this delegation is effective from that vote index (including) and forward (linked list)
    /// Undelegation can only be done if the vote_head is equal to the latest vote key of the delegatee token owner record
    /// This means that this delegatee is not actively used in any voting
    pub vote_head: Option<Pubkey>,

    pub last_vote_head: Option<Pubkey>,
}

impl MaxSize for ScopeDelegationRecordAccount {
    fn get_max_size(&self) -> Option<usize> {
        Some(1 + 32 + 32 + 8 + 1 + 32 + 1 + 32)
    }
}
impl IsInitialized for ScopeDelegationRecordAccount {
    fn is_initialized(&self) -> bool {
        self.account_type == AccountType::DelegationRecord
    }
}

impl ScopeDelegationRecordAccount {
    pub fn delegate<'a>(
        program_id: &Pubkey,
        amount: u64,
        scope: &Pubkey,
        rent: &Rent,
        scope_delegation_record_info: &AccountInfo<'a>,
        scope_delegation_record_bump_seed: u8,
        token_origin_record: &VotePowerOriginRecord,
        token_origin_record_info: &AccountInfo<'a>,
        governing_owner_info: &AccountInfo<'a>,
        delegatee_token_owner_record: &VotePowerOwnerRecord,
        delegatee_vote_power_owner_record_info: &AccountInfo<'a>,
        payer_info: &AccountInfo<'a>,
        system_info: &AccountInfo<'a>,
    ) -> Result<(), ProgramError> {
        msg!("Z");

        // TODO check delegation owner redcord mint
        if scope_delegation_record_info.data_is_empty() {
            let bump_seeds = [scope_delegation_record_bump_seed];
            let seeds = get_scope_delegation_account_program_address_seeds(
                token_origin_record_info.key,
                delegatee_vote_power_owner_record_info.key,
                scope,
                &bump_seeds,
            );

            create_and_serialize_account_verify_with_bump::<ScopeDelegationRecordAccount>(
                payer_info,
                scope_delegation_record_info,
                &ScopeDelegationRecordAccount {
                    account_type: AccountType::DelegationRecord,
                    amount,
                    delegator_token_origin_record: *token_origin_record_info.key,
                    delegatee_token_owner_record: *delegatee_vote_power_owner_record_info.key,
                    vote_head: None,
                    last_vote_head: delegatee_token_owner_record.latest_vote,
                },
                &seeds,
                program_id,
                system_info,
                rent,
            )?;
        } else {
            let mut scope_delegation_record = get_scope_delegation_record_data(
                program_id,
                scope_delegation_record_info,
                token_origin_record,
                token_origin_record_info,
                governing_owner_info,
                delegatee_vote_power_owner_record_info,
            )?;

            // check the state (so we can update)
            if !match (
                &delegatee_token_owner_record.latest_vote,
                &scope_delegation_record.vote_head,
            ) {
                (Some(a), Some(b)) => a == b,
                (None, None) => true,
                _ => false,
            } {
                return Err(GovernanceError::InvalidDelegationStateForUpdates.into());
            }

            scope_delegation_record.amount =
                scope_delegation_record.amount.checked_add(amount).unwrap();
            scope_delegation_record
                .serialize(&mut *scope_delegation_record_info.data.borrow_mut())?;
        }
        Ok(())
    }

    pub fn undelegate<'a>(
        program_id: &Pubkey,
        amount: u64,
        scope_delegation_record_info: &AccountInfo<'a>,
        token_origin_record: &VotePowerOriginRecord,
        token_origin_record_info: &AccountInfo<'a>,
        governing_owner_info: &AccountInfo<'a>,
        delegatee_token_owner_record: &VotePowerOwnerRecord,
        delegatee_vote_power_owner_record_info: &AccountInfo<'a>,
        beneficiary_info: &AccountInfo<'a>,
    ) -> Result<(), ProgramError> {
        // TODO check delegation owner redcord mint
        let mut scope_delegation_record = get_scope_delegation_record_data(
            program_id,
            scope_delegation_record_info,
            token_origin_record,
            token_origin_record_info,
            governing_owner_info,
            delegatee_vote_power_owner_record_info,
        )?;

        // check the state (so we can update)
        if scope_delegation_record.vote_head.is_some()
            || delegatee_token_owner_record.latest_vote != scope_delegation_record.last_vote_head
        {
            return Err(GovernanceError::InvalidDelegationStateForUpdates.into());
        }

        scope_delegation_record.amount =
            scope_delegation_record.amount.checked_sub(amount).unwrap();

        if scope_delegation_record.amount == 0 {
            dispose_account(scope_delegation_record_info, beneficiary_info);
        } else {
            scope_delegation_record
                .serialize(&mut *scope_delegation_record_info.data.borrow_mut())?;
        }
        Ok(())
    }
}

pub fn get_delegation_record_data_for_delegator_and_delegatee(
    program_id: &Pubkey,
    scope_delegation_record_info: &AccountInfo,
    delegator_token_origin_record: &AccountInfo,
    delegatee_token_owner_record: &AccountInfo,
) -> Result<ScopeDelegationRecordAccount, ProgramError> {
    let data =
        get_account_data::<ScopeDelegationRecordAccount>(program_id, scope_delegation_record_info)?;
    if &data.delegator_token_origin_record != delegator_token_origin_record.key
        || &data.delegatee_token_owner_record != delegatee_token_owner_record.key
    {
        return Err(GovernanceError::InvalidTokenOwnerRecordAccountAddress.into());
    }

    Ok(data)
}

pub fn get_scope_delegation_account_program_address(
    program_id: &Pubkey,
    from_token_owner_record: &Pubkey,
    to_token_owner_record: &Pubkey,
    scope: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            b"scope_delegation",
            from_token_owner_record.as_ref(),
            to_token_owner_record.as_ref(),
            scope.as_ref(),
        ],
        program_id,
    )
}
pub fn get_scope_delegation_account_program_address_seeds<'a>(
    from_token_owner_record: &'a Pubkey,
    to_token_owner_record: &'a Pubkey,
    scope: &'a Pubkey,
    bump_seed: &'a [u8],
) -> [&'a [u8]; 5] {
    return [
        b"scope_delegation",
        from_token_owner_record.as_ref(),
        to_token_owner_record.as_ref(),
        scope.as_ref(),
        bump_seed,
    ];
}

/// Deserializes ScopeDelegationAccount account and asserts it belongs to the given realm
pub fn get_scope_delegation_record_data(
    program_id: &Pubkey,
    delegation_record_info: &AccountInfo,
    token_origin_record: &VotePowerOriginRecord,
    token_origin_record_info: &AccountInfo,
    governing_owner_info: &AccountInfo,
    delegatee_vote_power_owner_record_info: &AccountInfo,
) -> Result<ScopeDelegationRecordAccount, ProgramError> {
    if !governing_owner_info.is_signer {
        return Err(GovernanceError::GoverningTokenOwnerMustSign.into());
    }
    if &token_origin_record.governing_owner != governing_owner_info.key {
        return Err(GovernanceError::InvalidTokenOwner.into());
    }

    msg!("Y");
    let scope_delegation_data =
        get_account_data::<ScopeDelegationRecordAccount>(program_id, delegation_record_info)?;
    msg!("YY");

    if &scope_delegation_data.delegator_token_origin_record != token_origin_record_info.key {
        return Err(GovernanceError::InvalidTokenOwnerRecordAccountAddress.into());
    }
    if &scope_delegation_data.delegatee_token_owner_record
        != delegatee_vote_power_owner_record_info.key
    {
        return Err(GovernanceError::InvalidTokenOwnerRecordAccountAddress.into());
    }

    Ok(scope_delegation_data)
}
