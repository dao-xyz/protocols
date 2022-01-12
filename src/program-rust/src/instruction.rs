use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};
use solana_program::instruction::Instruction;

#[repr(C)]
#[derive(Clone, Debug, PartialEq, BorshSerialize, BorshDeserialize, BorshSchema)]
pub enum SolveiInstruction {
    ChatInstruction(crate::social::instruction::ChatInstruction),
    StakePoolInstruction(crate::stake_pool::instruction::StakePoolInstruction),
}

pub const CHAT_INSTRUCTION_INDEX: u8 = 0;
pub const STAKE_POOL_INSTRUCTION_INDEX: u8 = 1;
