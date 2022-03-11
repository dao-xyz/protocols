//! Program state processor

use borsh::BorshSerialize;
use lchannel::state::ChannelAccount;
use shared::account::get_account_data;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::Clock,
    entrypoint::ProgramResult,
    instruction::Instruction,
    program::invoke_signed,
    pubkey::Pubkey,
    sysvar::Sysvar,
};

use crate::{
    error::PostError,
    state::{
        enums::{ProposalState, TransactionExecutionStatus},
        native_treasury::get_native_treasury_address_seeds,
        proposal::{
            get_proposal_data_for_channel,
            proposal_option::{get_proposal_option_data, ProposalOptionType},
            proposal_transaction::get_proposal_transaction_data_for_proposal, VoteType,
        },
    },
};

/// Processes ExecuteTransaction instruction
pub fn process_execute_transaction(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    let channel_info = next_account_info(account_info_iter)?;
    let proposal_info = next_account_info(account_info_iter)?;
    let proposal_transaction_info = next_account_info(account_info_iter)?;
    let proposal_option_info = next_account_info(account_info_iter)?;

    let clock = Clock::get()?;

    let mut proposal_data =
        get_proposal_data_for_channel(program_id, proposal_info, channel_info.key)?;

    let mut proposal_transaction_data = get_proposal_transaction_data_for_proposal(
        program_id,
        proposal_transaction_info,
        proposal_info.key,
    )?;
    let proposal_option_data =
        get_proposal_option_data(program_id, proposal_option_info, proposal_info.key)?;
    proposal_data.assert_can_execute_transaction(
        &proposal_transaction_data,
        &proposal_option_data,
        clock.unix_timestamp,
    )?;

    // Execute instruction with Governance PDA as signer
    let instructions = proposal_transaction_data
        .instructions
        .iter()
        //.map(|i| i.instruction_data)
        .map(Instruction::from);

    // In the current implementation accounts for all instructions are passed to each instruction invocation
    // This is an overhead but shouldn't be a showstopper because if we can invoke the parent instruction with that many accounts
    // then we should also be able to invoke all the nested ones
    // TODO: Optimize the invocation to split the provided accounts for each individual instruction
    let instruction_account_infos = account_info_iter.as_slice();

    let mut signers_seeds: Vec<&[&[u8]]> = vec![];

    let channel_account = get_account_data::<ChannelAccount>(&lchannel::id(), channel_info)?;
    let governance_seeds = channel_account.create_channel_account_program_address_seeds()?;
    let mut governance_seed_slices = governance_seeds
        .iter()
        .map(|x| &x[..])
        .collect::<Vec<&[u8]>>();

    let (_, bump_seed) = Pubkey::find_program_address(&governance_seed_slices, program_id);
    let bump = &[bump_seed];
    governance_seed_slices.push(bump);

    signers_seeds.push(&governance_seed_slices[..]);

    // Sign the transaction using the governance treasury PDA if required by the instruction
    let mut treasury_seeds = get_native_treasury_address_seeds(channel_info.key).to_vec();
    let (treasury_address, treasury_bump_seed) =
        Pubkey::find_program_address(&treasury_seeds, program_id);
    let treasury_bump = &[treasury_bump_seed];

    if instruction_account_infos
        .iter()
        .any(|a| a.key == &treasury_address)
    {
        treasury_seeds.push(treasury_bump);
        signers_seeds.push(&treasury_seeds[..]);
    }

    for instruction in instructions {
        invoke_signed(&instruction, instruction_account_infos, &signers_seeds[..])?;
    }

    // Update proposal and instruction accounts
    if proposal_data.state == ProposalState::Succeeded {
        proposal_data.executing_at = Some(clock.unix_timestamp);
        proposal_data.state = ProposalState::Executing;
    }

    let option_info = next_account_info(account_info_iter)?;
    let mut option_data = get_proposal_option_data(program_id, option_info, proposal_info.key)?;

    option_data.option_type = if let ProposalOptionType::Instruction {
        label,
        transactions_count,
        transactions_executed_count,
        transactions_next_index,
    } = &option_data.option_type
    {
        let new_transaction_executed_count = transactions_executed_count.checked_add(1).unwrap();
        if &new_transaction_executed_count == transactions_count {
            proposal_data.options_executed_count =
                proposal_data.options_executed_count.checked_add(1).unwrap();
        }

        ProposalOptionType::Instruction {
            label: label.clone(),
            transactions_count: *transactions_count,
            transactions_executed_count: new_transaction_executed_count,
            transactions_next_index: *transactions_next_index,
        }
    } else {
        return Err(PostError::InvalidOptionForInstructions.into());
    };

    option_data.serialize(&mut *option_info.data.borrow_mut())?;

    // Checking for Executing and ExecutingWithErrors states because instruction can still be executed after being flagged with error
    // The check for instructions_executed_count ensures Proposal can't be transitioned to Completed state from ExecutingWithErrors
    if proposal_data.state == ProposalState::Executing
        || proposal_data.state == ProposalState::ExecutingWithErrors
    {
        let done = match &proposal_data.vote_type {
            VoteType::SingleChoice { .. } => proposal_data.options_executed_count == 1,
            VoteType::MultiChoice { .. } => {
                proposal_data.options_executed_count as usize == proposal_data.winning_options.len()
            }
        };
        if done {
            proposal_data.closed_at = Some(clock.unix_timestamp);
            proposal_data.state = ProposalState::Completed;
        }
    }

    proposal_data.serialize(&mut *proposal_info.data.borrow_mut())?;

    proposal_transaction_data.executed_at = Some(clock.unix_timestamp);
    proposal_transaction_data.execution_status = TransactionExecutionStatus::Success;
    proposal_transaction_data.serialize(&mut *proposal_transaction_info.data.borrow_mut())?;

    Ok(())
}
