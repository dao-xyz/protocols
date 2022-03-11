use crate::{
    accounts::AccountType,
    error::PostError,
    instruction::{CreateProposalOptionType},
    state::{
        proposal::{
            get_proposal_data,
            proposal_option::{
                create_proposal_option_program_address_seeds, ProposalOption, ProposalOptionType,
            },
            OptionVoteResult,
        },
    },
};


use shared::account::{create_and_serialize_account_signed};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    pubkey::Pubkey,
    rent::Rent,
    sysvar::Sysvar,
};

pub fn process_create_proposal_option(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    proposal_option_type: CreateProposalOptionType,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    let proposal_option_info = next_account_info(account_info_iter)?;
    if !proposal_option_info.data_is_empty() {
        return Err(PostError::OptionAlreadyExist.into());
    }

    let proposal_info = next_account_info(account_info_iter)?;

    let mut proposal_data = get_proposal_data(program_id, proposal_info)?;
    proposal_data.assert_can_edit_instructions();

    let option_index = proposal_data.options_count;
    proposal_data.options_count = proposal_data.options_count.checked_add(1).unwrap();

    let proposal_vote_weigths = proposal_data.rules_max_vote_weight.clone(); // will be an array with 0s
    proposal_data.serialize(&mut *proposal_info.data.borrow_mut())?;

    let payer_info = next_account_info(account_info_iter)?;
    let system_info = next_account_info(account_info_iter)?;

    let proposal_option_data = ProposalOption {
        account_type: AccountType::ProposalOption,
        option_type: match proposal_option_type {
            CreateProposalOptionType::Instruction(label) => ProposalOptionType::Instruction {
                label,
                transactions_count: 0,
                transactions_executed_count: 0,
                transactions_next_index: 0,
            },
            CreateProposalOptionType::Deny => ProposalOptionType::Deny,
        },
        vote_result: OptionVoteResult::None,
        proposal: *proposal_info.key,
        index: option_index,
        vote_weights: proposal_vote_weigths,
    };
    let rent = Rent::get()?;
    create_and_serialize_account_signed::<ProposalOption>(
        payer_info,
        proposal_option_info,
        &proposal_option_data,
        &create_proposal_option_program_address_seeds(
            proposal_info.key,
            &option_index.to_le_bytes(),
        ),
        program_id,
        system_info,
        &rent,
    )?;

    Ok(())
}
