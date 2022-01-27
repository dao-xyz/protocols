use std::io::Result;

use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};
use solana_program::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    system_program,
};

use crate::{instruction::CHAT_INSTRUCTION_INDEX, shared::io_utils::try_to_vec_prepend};

use super::{
    find_user_account_program_address,
    state::{ProfilePicture, UserAccount, UserDescription},
};

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, BorshSchema, PartialEq)]
pub enum SocialInstruction {
    // Message builder is user to build a message that later can be submitted with the submitt message instruction
    CreateUser {
        name: String,
        profile: Option<ProfilePicture>,
        description: Option<UserDescription>,
        user_account_bump_seed: u8,
    },
}

impl SocialInstruction {
    /**
     * Prepends global instruction index
     */
    pub fn try_to_vec(&self) -> Result<Vec<u8>> {
        try_to_vec_prepend(CHAT_INSTRUCTION_INDEX, self)
    }
}

/// Creates a create user transction
pub fn create_user_transaction(program_id: &Pubkey, username: &str, payer: &Pubkey) -> Instruction {
    let (user_account, user_account_bump_seed) =
        find_user_account_program_address(program_id, username);
    Instruction {
        program_id: *program_id,
        data: SocialInstruction::CreateUser {
            name: username.into(),
            profile: None,
            description: None,
            user_account_bump_seed,
        }
        .try_to_vec()
        .unwrap(),
        accounts: vec![
            AccountMeta::new(*payer, true),
            AccountMeta::new(user_account, false),
            AccountMeta::new(system_program::id(), false),
        ],
    }
}
