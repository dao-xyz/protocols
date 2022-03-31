use crate::{
    accounts::AccountType,
    error::GovernanceError,
    state::{
        enums::TransactionExecutionStatus,
        proposal::{
            get_proposal_data,
            proposal_option::{get_proposal_option_data, ProposalOptionType},
            proposal_transaction::{
                get_proposal_transaction_address_seeds, ConditionedInstruction,
                ProposalTransactionV2,
            },
        },
        scopes::scope::Scope,
    },
};
use std::cmp::Ordering;

use borsh::BorshSerialize;
use shared::account::{create_and_serialize_account_signed, get_account_data};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
    rent::Rent,
    sysvar::Sysvar,
};

pub fn process_insert_transaction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    option_index: u16,
    instruction_index: u16,
    hold_up_time: u32,
    instructions: Vec<ConditionedInstruction>,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let proposal_info = next_account_info(account_info_iter)?;
    let creator_info = next_account_info(account_info_iter)?;
    let proposal_transaction_info = next_account_info(account_info_iter)?;
    let option_info = next_account_info(account_info_iter)?;
    let payer_info = next_account_info(account_info_iter)?;
    let system_info = next_account_info(account_info_iter)?;
    let rent_sysvar_info = next_account_info(account_info_iter)?;
    let rent = &Rent::from_account_info(rent_sysvar_info)?;

    if !proposal_transaction_info.data_is_empty() {
        return Err(GovernanceError::TransactionAlreadyExists.into());
    }

    let proposal_data = get_proposal_data(program_id, proposal_info)?;

    proposal_data.assert_can_edit_instructions(creator_info)?;

    let mut scope_info = next_account_info(account_info_iter)?;
    // Make sure that hold up time is ok by all the scopes
    for instruction in &instructions {
        let scope = &instruction.scope;
        if scope != scope_info.key {
            scope_info = next_account_info(account_info_iter)?;
            if scope != scope_info.key {
                return Err(ProgramError::InvalidAccountData);
            }
        }
        let scope_data = get_account_data::<Scope>(program_id, scope_info)?;
        if hold_up_time < scope_data.config.time_config.min_transaction_hold_up_time {
            return Err(GovernanceError::TransactionHoldUpTimeBelowRequiredMin.into());
        }
        scope_data.scope_applicable(&instruction.instruction_data)?;

        if !proposal_data
            .scopes_max_vote_weight
            .iter()
            .any(|scope_weight| &scope_weight.scope == scope)
        {
            return Err(GovernanceError::InvalidVotescope.into());
        }
    }
    // CHECK AUTHORITY
    /*
       let token_owner_record_data = get_token_owner_record_data_for_proposal_owner(
           program_id,
           token_owner_record_info,
           &proposal_data.token_owner_record,
       )?;

       token_owner_record_data.assert_token_owner_or_delegate_is_signer(governance_authority_info)?;
    */
    let mut option_data = get_proposal_option_data(program_id, option_info, proposal_info.key)?;

    if let ProposalOptionType::Instruction {
        label,
        transactions_count,
        transactions_executed_count,
        transactions_next_index,
    } = &option_data.option_type
    {
        option_data.option_type = ProposalOptionType::Instruction {
            label: label.clone(),
            transactions_count: transactions_count.checked_add(1).unwrap(),
            transactions_executed_count: *transactions_executed_count,
            transactions_next_index: match instruction_index.cmp(transactions_next_index) {
                Ordering::Greater => return Err(GovernanceError::InvalidTransactionIndex.into()),
                // If the index is the same as instructions_next_index then we are adding a new instruction
                // If the index is below instructions_next_index then we are inserting into an existing empty space
                Ordering::Equal => transactions_next_index.checked_add(1).unwrap(),
                Ordering::Less => *transactions_next_index,
            },
        };
        option_data.serialize(&mut *option_info.data.borrow_mut())?;
    } else {
        return Err(GovernanceError::InvalidOptionForInstructions.into());
    }

    let proposal_transaction_data = ProposalTransactionV2 {
        account_type: AccountType::Transaction,
        option_index,
        transaction_index: instruction_index,
        hold_up_time,
        instructions,
        executed_at: None,
        execution_status: TransactionExecutionStatus::None,
        proposal: *proposal_info.key,
        vote_result_collected_at: None,
    };

    create_and_serialize_account_signed::<ProposalTransactionV2>(
        payer_info,
        proposal_transaction_info,
        &proposal_transaction_data,
        &get_proposal_transaction_address_seeds(
            proposal_info.key,
            &option_index.to_le_bytes(),
            &instruction_index.to_le_bytes(),
        ),
        program_id,
        system_info,
        rent,
    )?;

    Ok(())
}
