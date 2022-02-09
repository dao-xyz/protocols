use std::io::Result;

use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};

use crate::{instruction::CHAT_INSTRUCTION_INDEX, shared::io_utils::try_to_vec_prepend};

use super::{
    channel::instruction::ChannelInstruction, post::instruction::PostInstruction,
    user::instruction::UserInstruction,
};

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, BorshSchema, PartialEq)]
pub enum SocialInstruction {
    // Message builder is user to build a message that later can be submitted with the submitt message instruction
    UserInstruction(UserInstruction),
    ChannelInstruction(ChannelInstruction),
    PostInstruction(PostInstruction),
}

impl SocialInstruction {
    /**
     * Prepends global instruction index
     */
    pub fn try_to_vec(&self) -> Result<Vec<u8>> {
        try_to_vec_prepend(CHAT_INSTRUCTION_INDEX, self)
    }
}
