/* use crate::state::{
    enums::{ProposalState, TransactionExecutionStatus},
    governance::get_governance_data,
    native_treasury::get_native_treasury_address_seeds,
    proposal::{get_proposal_data_for_governance, OptionVoteResult},
    proposal_transaction::get_proposal_transaction_data_for_proposal,
}; */
use crate::processor::{
    process_authority::{process_create_authority, process_delete_authority},
    process_channel::process_create_channel,
    process_post::process_create_post,
    process_vote::{process_post_unvote, process_post_vote},
};
use solana_program::{
    account_info::AccountInfo, borsh::try_from_slice_unchecked, entrypoint::ProgramResult, msg,
    pubkey::Pubkey,
};

use self::process_channel::process_update_channel_info;

use super::instruction::PostInstruction;

pub mod process_authority;
pub mod process_channel;
pub mod process_post;
pub mod process_vote;

pub struct Processor {}
impl Processor {
    pub fn process(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        let instruction = try_from_slice_unchecked::<PostInstruction>(instruction_data)?;
        match instruction {
            PostInstruction::CreateChannel {
                parent,
                name,
                info,
                channel_type,
                channel_account_bump_seed,
                channel_authority_seed,
                channel_authority_bump_seed,
                /* ,
                create_rule_address_bump_seed, */
            } => {
                msg!("Instruction: Create channel");
                process_create_channel(
                    program_id,
                    accounts,
                    channel_type,
                    parent,
                    name,
                    info,
                    channel_account_bump_seed,
                    channel_authority_seed,
                    channel_authority_bump_seed,
                )
            }
            PostInstruction::DeleteChannel => todo!(),
            PostInstruction::UpdateChannelInfo { info } => {
                msg!("Instruction: Update channel info");
                process_update_channel_info(program_id, accounts, info)
            }
            PostInstruction::CreateAuthority {
                authority_types,
                bump_seed,
                condition,
                seed,
            } => {
                msg!("Instruction: Create channel authority");
                process_create_authority(
                    program_id,
                    accounts,
                    authority_types,
                    condition,
                    seed,
                    bump_seed,
                )
            }

            PostInstruction::DeleteAuthority => {
                msg!("Instruction: Delete channel authority");
                process_delete_authority(program_id, accounts)
            }

            PostInstruction::CreatePost {
                content,
                hash,
                is_child,
                post_bump_seed,
                vote_config,
            } => {
                msg!("Instruction: Create post");
                process_create_post(
                    program_id,
                    accounts,
                    content,
                    hash,
                    is_child,
                    vote_config,
                    post_bump_seed,
                )
            }

            PostInstruction::Vote {
                vote,
                vote_record_bump_seed,
            } => {
                //let token_account_info = next_account_info(accounts_iter)?;
                msg!("Instruction: Create vote");
                process_post_vote(program_id, accounts, vote, vote_record_bump_seed)
            }

            PostInstruction::Unvote => {
                //let token_account_info = next_account_info(accounts_iter)?;
                msg!("Instruction: Create unvote");
                process_post_unvote(program_id, accounts)
            }
        }
    }
}
