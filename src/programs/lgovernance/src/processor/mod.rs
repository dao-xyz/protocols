/* use crate::state::{
    enums::{ProposalState, TransactionExecutionStatus},
    governance::get_governance_data,
    native_treasury::get_native_treasury_address_seeds,
    proposal::{get_proposal_data_for_governance, OptionVoteResult},
    proposal_transaction::get_proposal_transaction_data_for_proposal,
}; */
use crate::processor::{
    delegation::{
        process_create_delegatee::process_create_delegatee, process_delegate::process_delegate,
        process_delegate_history::process_delegate_history, process_undelegate::process_undelegate,
        process_undelegate_history::process_undelegate_history,
    },
    process_count_votes::{process_count_max_vote_weights, process_count_votes},
    process_create_governance::process_create_governance,
    process_create_native_treasury::process_create_native_treasury,
    process_create_proposal::process_create_proposal,
    process_create_proposal_option::process_create_proposal_option,
    process_create_token_owner_budget_record::process_create_token_owner_budget_record,
    process_execute_transaction::process_execute_transaction,
    process_finalize_draft::process_finalize_draft,
    process_insert_scope::process_insert_scope,
    process_scopes::process_create_scope,
    process_unvote::process_uncast_vote,
    process_vote::process_cast_vote,
};
use solana_program::{
    account_info::AccountInfo, borsh::try_from_slice_unchecked, entrypoint::ProgramResult, msg,
    pubkey::Pubkey,
};

use self::{
    process_create_realm::process_create_realm,
    process_deposit_governing_tokens::process_deposit_governing_tokens,
    process_insert_transaction::process_insert_transaction,
};

use super::instruction::PostInstruction;

pub mod delegation;
pub mod process_count_votes;
pub mod process_create_governance;
pub mod process_create_native_treasury;
pub mod process_create_proposal;
pub mod process_create_proposal_option;
pub mod process_create_realm;
pub mod process_create_token_owner_budget_record;
pub mod process_deposit_governing_tokens;
pub mod process_execute_transaction;
pub mod process_finalize_draft;
pub mod process_insert_scope;
pub mod process_insert_transaction;
pub mod process_scopes;
pub mod process_unvote;
pub mod process_vote;

pub struct Processor {}
impl Processor {
    // Create post

    /* pub fn process_finalize_p(program_id: &Pubkey,
    accounts: &[AccountInfo],
    post: CreatePost) */

    /* pub fn process_execute_post(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
        let accounts_iter = &mut accounts.iter();
        let post_account_info = next_account_info(accounts_iter)?;
        check_account_owner(post_account_info, program_id)?;
        let mut post = deserialize_post_account(*post_account_info.data.borrow())?;
        let channel_account_info = next_account_info(accounts_iter)?;
        check_account_owner(channel_account_info, &lchannel::id())?;
        let channel = deserialize_channel_account(*channel_account_info.data.borrow())?;
        let governence_mint_info = next_account_info(accounts_iter)?;
        let action_scope_info = next_account_info(accounts_iter)?;
        if action_scope_info.data_is_empty() {
            return Err(ProgramError::InvalidArgument);
        }
        let action_scope = deserialize_action_scope_account(*action_scope_info.data.borrow())?;

        if &action_scope.channel != channel_account_info.key {
            return Err(ProgramError::InvalidArgument);
        }

        if &action_scope.vote_mint != governence_mint_info.key {
            return Err(ProgramError::InvalidArgument);
        }
        check_account_owner(action_scope_info, program_id)?;

        if &post.channel != channel_account_info.key {
            return Err(ProgramError::InvalidArgument);
        }

        let supply = get_token_supply(governence_mint_info)?;

        post.post_type = match post.post_type {
            PostType::ActionPost(mut action) => {
                // check if already executed
                if action.status != ActionStatus::Pending {
                    return Err(ProgramError::Custom(123));
                }

                // check if vote is settled
                if action.expires_at <= Clock::get()?.unix_timestamp as u64 {
                    return Err(ProgramError::Custom(124)); // Not ready yet!
                }

                if action_scope
                    .is_approved(action.upvotes, action.downvotes, supply)
                    .unwrap()
                {
                    msg!("APPROVED");
                    action.status = ActionStatus::Approved;
                    /* match &action.action {
                        Action::DeletePost(post_to_delete) => {
                            let delete_post_account_info = next_account_info(accounts_iter)?;
                            if post_to_delete != delete_post_account_info.key {
                                return Err(ProgramError::InvalidArgument);
                            }
                            let mut post_deleted =
                                deserialize_post_account(*delete_post_account_info.data.borrow())?;
                            post_deleted.deleted = true;
                            post_deleted
                                .serialize(&mut *delete_post_account_info.data.borrow_mut())?;
                        }
                        Action::Treasury(treasury_action) => match treasury_action {
                            TreasuryAction::Transfer {
                                from,
                                to,
                                amount,
                                bump_seed,
                            } => {
                                let from_info = next_account_info(accounts_iter)?;
                                let to_info = next_account_info(accounts_iter)?;
                                if from != from_info.key {
                                    return Err(ProgramError::InvalidArgument);
                                }
                                if to != to_info.key {
                                    return Err(ProgramError::InvalidArgument);
                                }
                                let transfer_authority = next_account_info(accounts_iter)?;
                                let token_program_info = next_account_info(accounts_iter)?;
                                let bump_seeds = &[*bump_seed];
                                let seeds = create_authority_program_address_seeds(
                                    from_info.key,
                                    bump_seeds,
                                );
                                invoke_signed(
                                    &spl_token::instruction::transfer(
                                        token_program_info.key,
                                        from_info.key,
                                        to_info.key,
                                        transfer_authority.key,
                                        &[],
                                        *amount,
                                    )?,
                                    &[
                                        from_info.clone(),
                                        to_info.clone(),
                                        transfer_authority.clone(),
                                        token_program_info.clone(),
                                    ],
                                    &[&seeds],
                                )?;
                            }
                            TreasuryAction::Create { mint } => {
                                let payer_info = next_account_info(accounts_iter)?;
                                let mint_info = next_account_info(accounts_iter)?;
                                if mint == mint_info.key {
                                    let token_account_info = next_account_info(accounts_iter)?;
                                    let token_account_authority_info =
                                        next_account_info(accounts_iter)?;

                                    let system_program_info = next_account_info(accounts_iter)?;
                                    let token_program_info = next_account_info(accounts_iter)?;
                                    let rent_info = next_account_info(accounts_iter)?;

                                    let (treasury_token_address, treasury_token_address_bump_seed) =
                                        find_treasury_token_account_address(
                                            &post.channel,
                                            mint_info.key,
                                            token_program_info.key,
                                            program_id,
                                        );
                                    if &treasury_token_address != token_account_info.key {
                                        return Err(ProgramError::InvalidArgument);
                                    }

                                    let bump_seeds = &[treasury_token_address_bump_seed];
                                    let token_account_seeds =
                                        create_treasury_token_account_address_seeds(
                                            &post.channel,
                                            mint_info.key,
                                            token_program_info.key,
                                            bump_seeds,
                                        );

                                    let (token_account_authority, _) =
                                        find_authority_program_address(
                                            program_id,
                                            &treasury_token_address,
                                        );

                                    if &token_account_authority != token_account_authority_info.key
                                    {
                                        return Err(ProgramError::InvalidArgument);
                                    }

                                    create_program_token_account(
                                        token_account_info,
                                        &token_account_seeds,
                                        mint_info,
                                        token_account_authority_info,
                                        payer_info,
                                        rent_info,
                                        token_program_info,
                                        system_program_info,
                                        program_id,
                                    )?;
                                } else {
                                    return Err(ProgramError::InvalidArgument);
                                }
                            }
                        },
                        Action::ManageScope(modification) => match modification {
                            VotingScopeUpdate::Create { scope, bump_seed } => {
                                let payer_info = next_account_info(accounts_iter)?;
                                let new_scope_info = next_account_info(accounts_iter)?;
                                let system_account = next_account_info(accounts_iter)?;
                                check_system_program(system_account.key)?;
                                let create_scope_bump_seeds = &[*bump_seed];
                                let seeds = create_scope_associated_program_address_seeds(
                                    channel_account_info.key,
                                    &scope.action,
                                    create_scope_bump_seeds,
                                );

                                create_and_serialize_account_signed_verify(
                                    payer_info,
                                    new_scope_info,
                                    &Scope {
                                        account_type: AccountType::Scope,
                                        action: scope.action.clone(),
                                        channel: scope.channel,
                                        vote_mint: scope.vote_mint,
                                        info: scope.info.clone(),
                                        name: scope.name.clone(),
                                        criteria: scope.criteria.clone(),
                                        deleted: false,
                                    },
                                    &seeds,
                                    program_id,
                                    system_account,
                                    &Rent::get()?,
                                )?;
                            }
                            VotingScopeUpdate::Delete(scope) => {
                                let scope_info = next_account_info(accounts_iter)?;
                                check_account_owner(scope_info, program_id)?;

                                if scope_info.key != scope {
                                    return Err(ProgramError::InvalidArgument);
                                }
                                let mut scope =
                                    deserialize_action_scope_account(*scope_info.data.borrow())?;
                                if &scope.channel != channel_account_info.key {
                                    return Err(ProgramError::InvalidArgument);
                                }
                                scope.deleted = true;
                                scope.serialize(&mut *scope_info.data.borrow_mut())?;
                            }
                        },
                        Action::CustomEvent {
                            data: _,
                            event_type,
                        } => {
                            // well we dont need to do anything since the data is already on chain and the approved status has/will be set, so integration can be made
                            // but we have to check that the action event_type matches the scope event type
                            // since scopes for custom events are controlled by their event type

                            if let ActionType::CustomEvent(expected_event_type) = action_scope.action
                            {
                                if &expected_event_type != event_type {
                                    return Err(ProgramError::InvalidArgument);
                                }
                            } else {
                                // This should not happen, since the rul eaction type will also be of type
                                return Err(ProgramError::InvalidArgument);
                            }
                        }
                    } */
                } else {
                    action.status = ActionStatus::Rejected;
                }
                PostType::ActionPost(action)
            }
            _ => {
                panic!("Can not execute a non action post")
            }
        };

        post.serialize(&mut *post_account_info.data.borrow_mut())?;

        Ok(())
    } */

    /*
    /// Processes ExecuteTransaction instruction
    pub fn process_execute_transaction(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();

        let governance_info = next_account_info(account_info_iter)?; // 0
        let proposal_info = next_account_info(account_info_iter)?; // 1
        let proposal_transaction_info = next_account_info(account_info_iter)?; // 2

        let clock = Clock::get()?;

        let governance_data = get_governance_data(program_id, *,)?;

        let mut proposal_data =
            get_proposal_data_for_governance(program_id, proposal_info, governance_info.key)?;

        let mut proposal_transaction_data = get_proposal_transaction_data_for_proposal(
            program_id,
            proposal_transaction_info,
            proposal_info.key,
        )?;

        proposal_data
            .assert_can_execute_transaction(&proposal_transaction_data, clock.unix_timestamp)?;

        // Execute instruction with Governance PDA as signer
        let instructions = proposal_transaction_data
            .instructions
            .iter()
            .map(Instruction::from);

        // In the current implementation accounts for all instructions are passed to each instruction invocation
        // This is an overhead but shouldn't be a showstopper because if we can invoke the parent instruction with that many accounts
        // then we should also be able to invoke all the nested ones
        // TODO: Optimize the invocation to split the provided accounts for each individual instruction
        let instruction_account_infos = account_info_iter.as_slice();

        let mut signers_seeds: Vec<&[&[u8]]> = vec![];

        // Sign the transaction using the governance PDA
        let mut governance_seeds = governance_data.get_governance_address_seeds()?.to_vec();
        let (_, bump_seed) = Pubkey::find_program_address(&governance_seeds, program_id);
        let bump = &[bump_seed];
        governance_seeds.push(bump);

        signers_seeds.push(&governance_seeds[..]);

        // Sign the transaction using the governance treasury PDA if required by the instruction
        let mut treasury_seeds = get_native_treasury_address_seeds(governance_info.key).to_vec();
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

        let mut option =
            &mut proposal_data.options[proposal_transaction_data.option_index as usize];
        option.transactions_executed_count =
            option.transactions_executed_count.checked_add(1).unwrap();

        // Checking for Executing and ExecutingWithErrors states because instruction can still be executed after being flagged with error
        // The check for instructions_executed_count ensures Proposal can't be transitioned to Completed state from ExecutingWithErrors
        if (proposal_data.state == ProposalState::Executing
            || proposal_data.state == ProposalState::ExecutingWithErrors)
            && proposal_data
                .options
                .iter()
                .filter(|o| o.vote_result == OptionVoteResult::Succeeded)
                .all(|o| o.transactions_executed_count == o.transactions_count)
        {
            proposal_data.closed_at = Some(clock.unix_timestamp);
            proposal_data.state = ProposalState::Completed;
        }

        proposal_data.serialize(&mut *proposal_info.data.borrow_mut())?;

        proposal_transaction_data.executed_at = Some(clock.unix_timestamp);
        proposal_transaction_data.execution_status = TransactionExecutionStatus::Success;
        proposal_transaction_data.serialize(&mut *proposal_transaction_info.data.borrow_mut())?;

        Ok(())
    } */

    pub fn process(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        let instruction = try_from_slice_unchecked::<PostInstruction>(instruction_data)?;
        match instruction {
            PostInstruction::CreateProposal {
                bump_seed,
                scopes_count,
                source,
                vote_type,
            } => {
                msg!("Instruction: Create proposal");
                process_create_proposal(
                    program_id,
                    accounts,
                    vote_type,
                    scopes_count,
                    source,
                    bump_seed,
                )
            }
            PostInstruction::CreateRealm { bump_seed } => {
                msg!("Instruction: Create realm");
                process_create_realm(program_id, accounts, bump_seed)
            }
            PostInstruction::InsertScope => {
                msg!("Instruction: Insert scope");
                process_insert_scope(program_id, accounts)
            }

            /* ChatInstruction::CreatePostContent(content) => {
                msg!("Create post content");
                Self::process_create_post_content(program_id, accounts, content)
            } */
            PostInstruction::Vote {
                vote_record_bump_seed,
            } => {
                msg!("Instruction: Vote");
                process_cast_vote(program_id, accounts, vote_record_bump_seed)
            }

            PostInstruction::Unvote => {
                msg!("Instruction: Unvote");
                process_uncast_vote(program_id, accounts)
            }

            PostInstruction::ExecuteProposal {
                governance_bump_seed,
            } => {
                //let token_account_info = next_account_info(accounts_iter)?;
                msg!("Instruction: Execute proposal");
                process_execute_transaction(program_id, accounts, governance_bump_seed)
            }

            PostInstruction::CreateScope {
                id,
                bump_seed,
                config,
            } => {
                msg!("Instruction: Create scope");
                process_create_scope(program_id, accounts, &id, config, bump_seed)
            }
            PostInstruction::CreateProposalOption {
                option_type,
                bump_seed,
            } => {
                msg!("Instruction: Create proposal option");
                process_create_proposal_option(program_id, accounts, option_type, bump_seed)
            }
            PostInstruction::InsertTransaction {
                hold_up_time,
                instruction_index,
                instructions,
                option_index,
            } => {
                msg!("Instruction: Insert transaction");
                process_insert_transaction(
                    program_id,
                    accounts,
                    option_index,
                    instruction_index,
                    hold_up_time,
                    instructions,
                )
            }
            PostInstruction::DepositGoverningTokens {
                amount,
                token_owner_record_bump_seed,
            } => {
                msg!("Instruction: Deposit governing tokens");
                process_deposit_governing_tokens(
                    program_id,
                    accounts,
                    amount,
                    token_owner_record_bump_seed,
                )
            }

            PostInstruction::CreateGovernance { bump_seed } => {
                msg!("Instruction: Create governance");
                process_create_governance(program_id, accounts, bump_seed)
            }

            PostInstruction::FinalizeDraft => {
                msg!("Instruction: Finalize draft");
                process_finalize_draft(program_id, accounts)
            }

            PostInstruction::CountMaxVoteWeights => {
                msg!("Instruction: Count max vote weights");
                process_count_max_vote_weights(program_id, accounts)
            }

            PostInstruction::CountVotes => {
                msg!("Instruction: Count votes");
                process_count_votes(program_id, accounts)
            }
            PostInstruction::CreateNativeTreasury => {
                msg!("Instruction: Create native treasury");
                process_create_native_treasury(program_id, accounts)
            }
            PostInstruction::Delegate {
                amount,
                delegation_record_bump_seed,
            } => {
                msg!("Instruction: Delegate");
                process_delegate(program_id, accounts, amount, delegation_record_bump_seed)
            }

            PostInstruction::Undelegate { amount } => {
                msg!("Instruction: Undelegate");
                process_undelegate(program_id, accounts, amount)
            }

            PostInstruction::CreateDelegatee {
                token_owner_record_bump_seed,
                governing_token_mint,
                scope,
            } => {
                msg!("Instruction: Create delegate");
                process_create_delegatee(
                    program_id,
                    accounts,
                    scope,
                    governing_token_mint,
                    token_owner_record_bump_seed,
                )
            }
            PostInstruction::CreateTokenOwnerBudgetRecord {
                scope,
                token_owner_budget_record_bump_seed,
            } => {
                msg!("Instruction: Create token owner budget record");
                process_create_token_owner_budget_record(
                    program_id,
                    accounts,
                    scope,
                    token_owner_budget_record_bump_seed,
                )
            }
            PostInstruction::DelegateHistory => {
                msg!("Instruction: Delegate history");
                process_delegate_history(program_id, accounts)
            }

            PostInstruction::UndelegateHistory => {
                msg!("Instruction: Undelegate history");
                process_undelegate_history(program_id, accounts)
            }
        }
    }
}
