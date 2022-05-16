use crate::{
    accounts::AccountType,
    error::GovernanceError,
    state::{
        governance::GovernanceV2,
        scopes::scope::{get_scope_program_address_seeds, Scope, ScopeConfig},
    },
};
use shared::account::{check_system_program, create_and_serialize_account_verify_with_bump};

use shared::account::get_account_data;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program_error::ProgramError,
    pubkey::Pubkey,
    rent::Rent,
    sysvar::Sysvar,
};

pub fn process_create_scope(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    scope_id: &Pubkey,
    config: ScopeConfig,
    new_scope_bump_seed: u8,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let new_scope_account_info = next_account_info(accounts_iter)?;
    let governance_info = next_account_info(accounts_iter)?;
    let payer_info = next_account_info(accounts_iter)?;
    let system_info = next_account_info(accounts_iter)?;

    let governance_data = get_account_data::<GovernanceV2>(program_id, governance_info)?;
    if !governance_info.is_signer {
        // Load channel, or parent(s) and check authority
        /*  let channel_authority_info = next_account_info(accounts_iter)?;
        if !channel_authority_info.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let mut channel_info = next_account_info(accounts_iter)?;

        if &governance_data.channel != channel_info.key {
            return Err(ProgramError::InvalidAccountData);
        }

        let mut channel = get_account_data::<ChannelAccount>(&lchannel::id(), channel_info)?;
        while &channel.authority != channel_authority_info.key {
            if let Some(parent) = &channel.parent {
                let parent_channel_info = next_account_info(accounts_iter)?;
                if parent != parent_channel_info.key {
                    return Err(ProgramError::InvalidAccountData);
                }
                channel_info = parent_channel_info;
                channel = get_account_data::<ChannelAccount>(&lchannel::id(), channel_info)?;
            } else {
                return Err(ProgramError::InvalidAccountData);
            }
        }
        if &channel.authority != channel_authority_info.key {
            return Err(ProgramError::InvalidAccountData);
        } */

        if let Some(option_authority) = &governance_data.optional_authority {
            let authority_info = next_account_info(accounts_iter)?;

            if option_authority != authority_info.key {
                return Err(GovernanceError::InvalidAuthorityForGovernance.into());
            }

            if !authority_info.is_signer {
                return Err(ProgramError::MissingRequiredSignature);
            }
        } else {
            return Err(ProgramError::MissingRequiredSignature);
        }
    }

    check_system_program(system_info.key)?;

    let rent = Rent::get()?;
    let new_scope_bump_seeds = [new_scope_bump_seed];
    let create_scope_seeds = get_scope_program_address_seeds(scope_id, &new_scope_bump_seeds);
    create_and_serialize_account_verify_with_bump::<Scope>(
        payer_info,
        new_scope_account_info,
        &Scope {
            account_type: AccountType::Scope,
            governance: *governance_info.key,
            id: *scope_id,
            config,
            voting_proposal_count: 0,
            proposal_count: 0,
            deleted: false,
        },
        &create_scope_seeds,
        program_id,
        system_info,
        &rent,
    )?;
    Ok(())
}
/*
pub fn process_create_scope_vote_mint_weight(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    scope_id: &Pubkey,
    vote_mint: &Pubkey,
    vote_weight: u64,
    vote_weight_bump_seed: u8,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let channel_info = next_account_info(accounts_iter)?;
    let channel = get_account_data::<ChannelAccount>(program_id, channel_info)?;
    let authority_info = next_account_info(accounts_iter)?;
    channel.check_authority(authority_info)?;

    let vote_weight_account_info = next_account_info(accounts_iter)?;
    let payer_info = next_account_info(accounts_iter)?;
    let system_info = next_account_info(accounts_iter)?;

    check_account_owner(channel_info, program_id)?;
    if !channel_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    check_system_program(system_info.key);
    let rent = Rent::get()?;
    let vote_weight_bump_seeds = [vote_weight_bump_seed];
    let vote_weight_seeds =
        create_scope_vote_weight_program_address_seeds(scope_id, vote_mint, &vote_weight_bump_seeds);
    create_and_serialize_account_verify_with_bump::<ScopeVoteWeight>(
        payer_info,
        vote_weight_account_info,
        &ScopeVoteWeight {
            account_type: AccountType::ScopeVoteWeight,
            mint: *vote_mint,
            weight: vote_weight,
            scope: *scope_id,
        },
        &vote_weight_seeds,
        program_id,
        system_info,
        &rent,
    )?;
    Ok(())
}
 */
/* pub fn process_create_first_scope(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    id: &Pubkey,
    config: &ScopeCondition,
    scope_bump_seed: u8,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let new_scope_account_info = next_account_info(accounts_iter)?;
    let payer_account = next_account_info(accounts_iter)?;
    let system_account = next_account_info(accounts_iter)?;
    check_system_program(system_account.key);

    // Create a scope with acceptance criteria on the channel that allows
    // proposals to made to create other scopes

}
 */
