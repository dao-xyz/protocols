use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};
use shared::content::ContentSource;
use solana_program::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    system_program,
};

use crate::get_sign_for_me_program_address;

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, BorshSchema, PartialEq)]
pub enum TagInstruction {
    CreateSignForMe {
        #[allow(dead_code)] // but it's not
        signer: Pubkey,

        #[allow(dead_code)] // but it's not
        scope: Pubkey,

        #[allow(dead_code)] // but it's not
        bump_seed: u8,
    },
    DeleteSignForMe,
}

pub fn create_sign_for_me(
    program_id: &Pubkey,

    // Accounts
    owner: &Pubkey,
    signer: &Pubkey,
    scope: &Pubkey,
    payer: &Pubkey,
    // Args
) -> Instruction {
    let (sign_for_me_address, bump_seed) =
        get_sign_for_me_program_address(program_id, owner, signer, scope);

    Instruction {
        program_id: *program_id,
        data: (TagInstruction::CreateSignForMe {
            scope: *scope,
            signer: *signer,
            bump_seed,
        })
        .try_to_vec()
        .unwrap(),
        accounts: vec![
            AccountMeta::new(sign_for_me_address, false),
            AccountMeta::new_readonly(*owner, true),
            AccountMeta::new(*payer, true),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
    }
}

pub fn delete_sign_for_me_as_owner(
    program_id: &Pubkey,
    // Accounts
    sign_for_me: &Pubkey,
    owner: &Pubkey,
    withdraw_destination: &Pubkey,
    // Args
) -> Instruction {
    Instruction {
        program_id: *program_id,
        data: (TagInstruction::DeleteSignForMe).try_to_vec().unwrap(),
        accounts: vec![
            AccountMeta::new(*sign_for_me, false),
            AccountMeta::new_readonly(*owner, true),
            AccountMeta::new(*withdraw_destination, false),
        ],
    }
}
