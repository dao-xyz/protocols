use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, pubkey::Pubkey};

use crate::instruction::S2GInstruction;
use borsh::BorshDeserialize;

pub struct Processor {}
impl Processor {
    pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], input: &[u8]) -> ProgramResult {
        let instruction = S2GInstruction::try_from_slice(input)?;
        match instruction {
            S2GInstruction::StakePoolInstruction(instruction) => {
                crate::stake_pool::processor::Processor::process(program_id, accounts, instruction)
            }
            S2GInstruction::SocialInstruction(instruction) => {
                crate::socials::processor::Processor::process(program_id, accounts, instruction)
            }
        }
    }
}
