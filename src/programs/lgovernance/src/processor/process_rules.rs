use crate::{
    accounts::AccountType,
    state::rules::rule::{
        create_rule_associated_program_address_seeds, Rule, RuleTimeConfig, RuleVoteConfig,
    },
};
use shared::account::{
    check_account_owner, check_system_program, create_and_serialize_account_verify_with_bump,
};

use lchannel::state::ChannelAccount;
use shared::account::get_account_data;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program_error::ProgramError,
    pubkey::Pubkey,
    rent::Rent,
    sysvar::Sysvar,
};

pub fn process_create_rule(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    rule_id: &Pubkey,
    vote_config: RuleVoteConfig,
    time_config: RuleTimeConfig,
    new_rule_bump_seed: u8,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let channel_info = next_account_info(accounts_iter)?;
    let channel = get_account_data::<ChannelAccount>(program_id, channel_info)?;
    let authority_info = next_account_info(accounts_iter)?;
    channel.check_authority(authority_info)?;

    let new_rule_account_info = next_account_info(accounts_iter)?;
    let payer_info = next_account_info(accounts_iter)?;
    let system_info = next_account_info(accounts_iter)?;

    check_account_owner(channel_info, program_id)?;
    if !channel_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    check_system_program(system_info.key)?;

    let rent = Rent::get()?;
    let new_rule_bump_seeds = [new_rule_bump_seed];
    let create_rule_seeds =
        create_rule_associated_program_address_seeds(rule_id, &new_rule_bump_seeds);
    create_and_serialize_account_verify_with_bump::<Rule>(
        payer_info,
        new_rule_account_info,
        &Rule {
            account_type: AccountType::Rule,
            channel: *channel_info.key,
            id: *rule_id,
            vote_config,
            time_config,
            voting_proposal_count: 0,
            deleted: false,
        },
        &create_rule_seeds,
        program_id,
        system_info,
        &rent,
    )?;
    Ok(())
}
/*
pub fn process_create_rule_vote_mint_weight(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    rule_id: &Pubkey,
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
        create_rule_vote_weight_program_address_seeds(rule_id, vote_mint, &vote_weight_bump_seeds);
    create_and_serialize_account_verify_with_bump::<RuleVoteWeight>(
        payer_info,
        vote_weight_account_info,
        &RuleVoteWeight {
            account_type: AccountType::RuleVoteWeight,
            mint: *vote_mint,
            weight: vote_weight,
            rule: *rule_id,
        },
        &vote_weight_seeds,
        program_id,
        system_info,
        &rent,
    )?;
    Ok(())
}
 */
/* pub fn process_create_first_rule(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    id: &Pubkey,
    config: &RuleCondition,
    rule_bump_seed: u8,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let new_rule_account_info = next_account_info(accounts_iter)?;
    let payer_account = next_account_info(accounts_iter)?;
    let system_account = next_account_info(accounts_iter)?;
    check_system_program(system_account.key);

    // Create a rule with acceptance criteria on the channel that allows
    // proposals to made to create other rules

}
 */
