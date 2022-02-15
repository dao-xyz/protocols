use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};

#[repr(C)]
#[derive(Clone, Debug, PartialEq, BorshSerialize, BorshDeserialize, BorshSchema)]
pub enum S2GInstruction {
    StakePoolInstruction(crate::stake_pool::instruction::StakePoolInstruction),
    SocialInstruction(crate::socials::instruction::SocialInstruction),
}

pub const STAKE_POOL_INSTRUCTION_INDEX: u8 = 0;
pub const CHAT_INSTRUCTION_INDEX: u8 = 1;

#[repr(C)]
#[derive(Clone, Debug, PartialEq, BorshDeserialize, BorshSerialize, BorshSchema)]
pub enum S2GAccountType {
    //Uninitialized,
    StakePool,
    Social,
}

/* impl Default for S2GAccountType {
    fn default() -> Self {
        S2GAccountType::Uninitialized
    }
}
 */
pub const STAKE_POOL_ACCOUNT_INDEX: u8 = 0;
pub const CHAT_ACCOUNT_INDEX: u8 = 1;
