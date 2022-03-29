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
    state::token_owner_record::{get_token_owner_record_data, TokenOwnerRecordV2},
};

#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, BorshSchema, PartialEq)]
pub struct RuleDelegationRecordAccount {
    pub account_type: AccountType,
    pub delegator_token_owner_record: Pubkey,
    pub delegatee_token_owner_record: Pubkey,
    pub amount: u64,

    /// At what vote "index" is this delegation used for.
    /// Empty, means that no votes had happened for the delegatee_token_owner_record
    /// Non empty value means that this delegation is effective from that vote index (including) and forward (linked list)
    /// Undelegation can only be done if the vote_head is equal to the latest vote key of the delegatee token owner record
    /// This means that this delegatee is not actively used in any voting
    pub vote_head: Option<Pubkey>,
}

impl MaxSize for RuleDelegationRecordAccount {
    fn get_max_size(&self) -> Option<usize> {
        Some(1 + 32 + 32 + 8 + 1 + 32)
    }
}
impl IsInitialized for RuleDelegationRecordAccount {
    fn is_initialized(&self) -> bool {
        self.account_type == AccountType::DelegationRecord
    }
}

impl RuleDelegationRecordAccount {
    pub fn delegate<'a>(
        program_id: &Pubkey,
        amount: u64,
        rule: &Pubkey,
        rent: &Rent,
        rule_delegation_record_info: &AccountInfo<'a>,
        rule_delegation_record_bump_seed: u8,
        token_owner_record: &TokenOwnerRecordV2,
        token_owner_record_info: &AccountInfo<'a>,
        governing_token_owner_info: &AccountInfo<'a>,
        delegatee_token_owner_record: &TokenOwnerRecordV2,
        delegatee_token_owner_record_info: &AccountInfo<'a>,
        payer_info: &AccountInfo<'a>,
        system_info: &AccountInfo<'a>,
    ) -> Result<(), ProgramError> {
        msg!("Z");

        // TODO check delegation owner redcord mint
        if rule_delegation_record_info.data_is_empty() {
            let bump_seeds = [rule_delegation_record_bump_seed];
            let seeds = get_rule_delegation_account_program_address_seeds(
                token_owner_record_info.key,
                delegatee_token_owner_record_info.key,
                &rule,
                &bump_seeds,
            );

            create_and_serialize_account_verify_with_bump::<RuleDelegationRecordAccount>(
                payer_info,
                rule_delegation_record_info,
                &RuleDelegationRecordAccount {
                    account_type: AccountType::DelegationRecord,
                    amount,
                    delegator_token_owner_record: *token_owner_record_info.key,
                    delegatee_token_owner_record: *delegatee_token_owner_record_info.key,
                    vote_head: delegatee_token_owner_record.latest_vote,
                },
                &seeds,
                program_id,
                system_info,
                &rent,
            )?;
        } else {
            let mut rule_delegation_record = get_rule_delegation_record_data(
                program_id,
                rule_delegation_record_info,
                token_owner_record,
                token_owner_record_info,
                governing_token_owner_info,
                delegatee_token_owner_record_info,
            )?;

            // check the state (so we can update)
            if !match (
                &delegatee_token_owner_record.latest_vote,
                &rule_delegation_record.vote_head,
            ) {
                (Some(a), Some(b)) => a == b,
                (None, None) => true,
                _ => false,
            } {
                return Err(GovernanceError::InvalidDelegationStateForUpdates.into());
            }

            rule_delegation_record.amount =
                rule_delegation_record.amount.checked_add(amount).unwrap();
            rule_delegation_record
                .serialize(&mut *rule_delegation_record_info.data.borrow_mut())?;
        }
        Ok(())
    }

    pub fn undelegate<'a>(
        program_id: &Pubkey,
        amount: u64,
        rule_delegation_record_info: &AccountInfo<'a>,
        token_owner_record: &TokenOwnerRecordV2,
        token_owner_record_info: &AccountInfo<'a>,
        governing_token_owner_info: &AccountInfo<'a>,
        delegatee_token_owner_record: &TokenOwnerRecordV2,
        delegatee_token_owner_record_info: &AccountInfo<'a>,
        beneficiary_info: &AccountInfo<'a>,
    ) -> Result<(), ProgramError> {
        // TODO check delegation owner redcord mint
        let mut rule_delegation_record = get_rule_delegation_record_data(
            program_id,
            rule_delegation_record_info,
            token_owner_record,
            token_owner_record_info,
            governing_token_owner_info,
            delegatee_token_owner_record_info,
        )?;

        // check the state (so we can update)
        if !match (
            &delegatee_token_owner_record.latest_vote,
            &rule_delegation_record.vote_head,
        ) {
            (Some(a), Some(b)) => a == b,
            (None, None) => true,
            _ => false,
        } {
            msg!(
                "VOTE HEAD NOT SYNC FOR UNDEL {} {}",
                delegatee_token_owner_record.latest_vote.is_none(),
                rule_delegation_record.vote_head.is_none()
            );
            return Err(GovernanceError::InvalidDelegationStateForUpdates.into());
        }

        rule_delegation_record.amount = rule_delegation_record.amount.checked_sub(amount).unwrap();

        if rule_delegation_record.amount == 0 {
            dispose_account(rule_delegation_record_info, beneficiary_info);
        } else {
            rule_delegation_record
                .serialize(&mut *rule_delegation_record_info.data.borrow_mut())?;
        }
        Ok(())
    }
}

pub fn get_delegation_record_data_for_delegator(
    program_id: &Pubkey,
    rule_delegation_record_info: &AccountInfo,
    delegator_token_owner_record: &Pubkey,
) -> Result<RuleDelegationRecordAccount, ProgramError> {
    let data =
        get_account_data::<RuleDelegationRecordAccount>(program_id, rule_delegation_record_info)?;
    if &data.delegator_token_owner_record != delegator_token_owner_record {
        return Err(GovernanceError::InvalidTokenOwnerRecordAccountAddress.into());
    }
    Ok(data)
}

pub fn get_rule_delegation_account_program_address(
    program_id: &Pubkey,
    from_token_owner_record: &Pubkey,
    to_token_owner_record: &Pubkey,
    rule: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            b"rule_delegation",
            from_token_owner_record.as_ref(),
            to_token_owner_record.as_ref(),
            rule.as_ref(),
        ],
        program_id,
    )
}
pub fn get_rule_delegation_account_program_address_seeds<'a>(
    from_token_owner_record: &'a Pubkey,
    to_token_owner_record: &'a Pubkey,
    rule: &'a Pubkey,
    bump_seed: &'a [u8],
) -> [&'a [u8]; 5] {
    return [
        b"rule_delegation",
        from_token_owner_record.as_ref(),
        to_token_owner_record.as_ref(),
        rule.as_ref(),
        bump_seed,
    ];
}

/// Deserializes RuleDelegationAccount account and asserts it belongs to the given realm
pub fn get_rule_delegation_record_data(
    program_id: &Pubkey,
    delegation_record_info: &AccountInfo,
    token_owner_record: &TokenOwnerRecordV2,
    token_owner_record_info: &AccountInfo,
    governing_token_owner_info: &AccountInfo,
    delegatee_token_owner_record_info: &AccountInfo,
) -> Result<RuleDelegationRecordAccount, ProgramError> {
    if !governing_token_owner_info.is_signer {
        return Err(GovernanceError::GoverningTokenOwnerMustSign.into());
    }
    if &token_owner_record.governing_token_owner != governing_token_owner_info.key {
        return Err(GovernanceError::InvalidTokenOwner.into());
    }

    msg!("Y");
    let rule_delegation_data =
        get_account_data::<RuleDelegationRecordAccount>(program_id, delegation_record_info)?;
    msg!("YY");

    if &rule_delegation_data.delegator_token_owner_record != token_owner_record_info.key {
        return Err(GovernanceError::InvalidTokenOwnerRecordAccountAddress.into());
    }
    if &rule_delegation_data.delegatee_token_owner_record != delegatee_token_owner_record_info.key {
        return Err(GovernanceError::InvalidTokenOwnerRecordAccountAddress.into());
    }

    Ok(rule_delegation_data)
}
