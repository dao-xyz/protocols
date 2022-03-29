use crate::state::{
    enums::TransactionExecutionStatus,
    proposal::{
        get_proposal_data,
        proposal_option::ProposalOptionType,
        proposal_transaction::{
            get_proposal_transaction_address_seeds, ConditionedInstruction, ProposalTransactionV2,
        },
    },
    rules::{
        rule::{get_rule_data, Rule},
        rule_weight::RuleWeight,
    },
};

use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    pubkey::Pubkey,
};

pub fn process_insert_rule(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let rule_info = next_account_info(account_info_iter)?; // 1
    let proposal_info = next_account_info(account_info_iter)?; // 1
    let creator_info = next_account_info(account_info_iter)?; // 0
    let mut proposal_data = get_proposal_data(program_id, proposal_info)?;
    proposal_data.assert_can_edit_rules(creator_info)?;

    let rule_data = get_rule_data(program_id, rule_info)?;

    proposal_data.rules_max_vote_weight.push(RuleWeight {
        rule: *rule_info.key,
        weight: 0, // max vote weight is calculated later
    });

    /*    proposal_data
    .common_rule_config
    .set_strictest(&rule_data.config.vote_config.vote_tipping); */

    proposal_data.serialize(&mut *proposal_info.data.borrow_mut())?;
    Ok(())
}
