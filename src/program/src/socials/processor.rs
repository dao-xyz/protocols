use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, pubkey::Pubkey};

use super::instruction::SocialInstruction;

pub struct Processor {}
impl Processor {
    // Program entrypoint's implementation

    pub fn process(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction: SocialInstruction,
    ) -> ProgramResult {
        match instruction {
            SocialInstruction::UserInstruction(instruction) => {
                super::user::processor::Processor::process(program_id, accounts, instruction)
            }
            SocialInstruction::ChannelInstruction(instruction) => {
                super::channel::processor::Processor::process(program_id, accounts, instruction)
            }
            SocialInstruction::PostInstruction(instruction) => {
                super::post::processor::Processor::process(program_id, accounts, instruction)
            }
        }
    }
}
