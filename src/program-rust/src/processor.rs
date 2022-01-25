use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, pubkey::Pubkey};

use crate::{instruction::SolveiInstruction, stake_pool::instruction::StakePoolInstruction};
use borsh::BorshDeserialize;

pub struct Processor {}

impl Processor {
    pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], input: &[u8]) -> ProgramResult {
        let instruction = SolveiInstruction::try_from_slice(input)?;
        match instruction {
            SolveiInstruction::StakePoolInstruction(instruction) => {
                crate::stake_pool::processor::Processor::process(program_id, accounts, instruction)
            }
        }
    }
}
