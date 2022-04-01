use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};
use shared::content::ContentSource;
use solana_program::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    system_program,
};

use crate::{
    get_tag_program_address, get_tag_record_factory_program_address, get_tag_record_program_address,
};

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, BorshSchema, PartialEq)]
pub enum TagInstruction {
    CreateTag {
        #[allow(dead_code)] // but it's not
        tag: String,

        #[allow(dead_code)] // but it's not
        info: Option<ContentSource>,

        #[allow(dead_code)] // but it's not
        bump_seed: u8,
    },
    CreateTagRecord {
        #[allow(dead_code)] // but it's not
        bump_seed: u8,
    },
    DeleteTagRecord,

    CreateTagRecordFactory {
        #[allow(dead_code)] // but it's not
        tag: Pubkey,

        #[allow(dead_code)] // but it's not
        bump_seed: u8,
    },
}

/// Creates a tag transction
pub fn create_tag(
    program_id: &Pubkey,
    tag: &str,
    info: Option<ContentSource>,
    authority: &Pubkey,
    payer: &Pubkey,
) -> Instruction {
    let (tag_address, tag_account_bump_seed) = get_tag_program_address(program_id, tag);
    Instruction {
        program_id: *program_id,
        data: (TagInstruction::CreateTag {
            tag: tag.into(),
            info,
            bump_seed: tag_account_bump_seed,
        })
        .try_to_vec()
        .unwrap(),
        accounts: vec![
            AccountMeta::new(tag_address, false),
            AccountMeta::new_readonly(*authority, true),
            AccountMeta::new(*payer, true),
            AccountMeta::new(system_program::id(), false),
        ],
    }
}

pub fn create_tag_record(
    program_id: &Pubkey,
    tag: &Pubkey,
    owner: &Pubkey,
    factory: &Pubkey,
    authority: &Pubkey,
    payer: &Pubkey,
) -> Instruction {
    let (tag_record_address, tag_record_bump_seed) =
        get_tag_record_program_address(program_id, tag, factory, owner);

    Instruction {
        program_id: *program_id,
        data: (TagInstruction::CreateTagRecord {
            bump_seed: tag_record_bump_seed,
        })
        .try_to_vec()
        .unwrap(),
        accounts: vec![
            AccountMeta::new(tag_record_address, false),
            AccountMeta::new_readonly(*owner, false),
            AccountMeta::new(*factory, false),
            AccountMeta::new_readonly(*authority, true),
            AccountMeta::new(*payer, true),
            AccountMeta::new(system_program::id(), false),
        ],
    }
}

pub fn delete_tag_record_as_owner(
    program_id: &Pubkey,
    tag_record: &Pubkey,
    owner: &Pubkey,
    factory: &Pubkey,
    authority: &Pubkey,
    withdraw_destination: &Pubkey,
) -> Instruction {
    Instruction {
        program_id: *program_id,
        data: (TagInstruction::DeleteTagRecord).try_to_vec().unwrap(),
        accounts: vec![
            AccountMeta::new(*tag_record, false),
            AccountMeta::new_readonly(*owner, true),
            AccountMeta::new(*factory, false),
            AccountMeta::new_readonly(*authority, false),
            AccountMeta::new(*withdraw_destination, false),
        ],
    }
}

pub fn delete_tag_record_as_authority(
    program_id: &Pubkey,
    tag_record: &Pubkey,
    owner: &Pubkey,
    factory: &Pubkey,
    authority: &Pubkey,
    withdraw_destination: &Pubkey,
) -> Instruction {
    Instruction {
        program_id: *program_id,
        data: (TagInstruction::DeleteTagRecord).try_to_vec().unwrap(),
        accounts: vec![
            AccountMeta::new(*tag_record, false),
            AccountMeta::new_readonly(*owner, false),
            AccountMeta::new(*factory, false),
            AccountMeta::new_readonly(*authority, true),
            AccountMeta::new(*withdraw_destination, false),
        ],
    }
}
pub fn create_tag_record_factory(
    program_id: &Pubkey,
    tag: &Pubkey,
    authority: &Pubkey,
    payer: &Pubkey,
) -> Instruction {
    let (tag_record_factory, tag_record_factory_bump_seed) =
        get_tag_record_factory_program_address(program_id, tag, authority);

    Instruction {
        program_id: *program_id,
        data: (TagInstruction::CreateTagRecordFactory {
            tag: *tag,
            bump_seed: tag_record_factory_bump_seed,
        })
        .try_to_vec()
        .unwrap(),
        accounts: vec![
            AccountMeta::new(tag_record_factory, false),
            AccountMeta::new_readonly(*authority, true),
            AccountMeta::new_readonly(*tag, false),
            AccountMeta::new(*payer, true),
            AccountMeta::new(system_program::id(), false),
        ],
    }
}
