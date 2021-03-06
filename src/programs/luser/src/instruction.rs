use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};
use shared::content::ContentSource;
use solana_program::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    system_program,
};

use crate::find_user_account_program_address;

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, BorshSchema, PartialEq)]
pub enum UserInstruction {
    // Message builder is user to build a message that later can be submitted with the submitt message instruction
    CreateUser {
        #[allow(dead_code)] // but it's not
        name: String,
        #[allow(dead_code)] // but it's not
        profile: Option<ContentSource>,
        #[allow(dead_code)] // but it's not
        user_account_bump_seed: u8,
    },
    UpdateUser {
        #[allow(dead_code)] // but it's not
        profile: Option<ContentSource>,
    },
}

/// Creates a create user transction
pub fn create_user_transaction(
    program_id: &Pubkey,
    username: &str,
    profile: Option<ContentSource>,
    owner: &Pubkey,
    payer: &Pubkey,
) -> Instruction {
    let (user_account, user_account_bump_seed) =
        find_user_account_program_address(program_id, username);
    Instruction {
        program_id: *program_id,
        data: (UserInstruction::CreateUser {
            name: username.into(),
            profile,
            user_account_bump_seed,
        })
        .try_to_vec()
        .unwrap(),
        accounts: vec![
            AccountMeta::new(user_account, false),
            AccountMeta::new_readonly(*owner, true),
            AccountMeta::new(*payer, true),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
    }
}

/// Creates a create user transction
pub fn create_update_user_transaction(
    program_id: &Pubkey,
    username: &str,
    profile: Option<ContentSource>,
    owner: &Pubkey,
) -> Instruction {
    let (user_account, _) = find_user_account_program_address(program_id, username);
    Instruction {
        program_id: *program_id,
        data: (UserInstruction::UpdateUser { profile })
            .try_to_vec()
            .unwrap(),
        accounts: vec![
            AccountMeta::new(user_account, false),
            AccountMeta::new_readonly(*owner, true),
        ],
    }
}
