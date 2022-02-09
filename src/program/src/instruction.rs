use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};

#[repr(C)]
#[derive(Clone, Debug, PartialEq, BorshSerialize, BorshDeserialize, BorshSchema)]
pub enum SolveiInstruction {
    StakePoolInstruction(crate::stake_pool::instruction::StakePoolInstruction),
    SocialInstruction(crate::socials::instruction::SocialInstruction),
}

pub const STAKE_POOL_INSTRUCTION_INDEX: u8 = 0;
pub const CHAT_INSTRUCTION_INDEX: u8 = 1;
