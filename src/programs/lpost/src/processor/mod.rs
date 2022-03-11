/* use crate::state::{
    enums::{ProposalState, TransactionExecutionStatus},
    governance::get_governance_data,
    native_treasury::get_native_treasury_address_seeds,
    proposal::{get_proposal_data_for_governance, OptionVoteResult},
    proposal_transaction::get_proposal_transaction_data_for_proposal,
}; */
use crate::processor::{
    process_post::process_create_post,
    process_vote::{process_post_unvote, process_post_vote},
};
use solana_program::{
    account_info::AccountInfo, borsh::try_from_slice_unchecked, entrypoint::ProgramResult, msg,
    pubkey::Pubkey,
};

use super::instruction::PostInstruction;

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
            PostInstruction::CreatePost(post) => {
                msg!("Instruction: Create post");
                process_create_post(program_id, accounts, post)
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
