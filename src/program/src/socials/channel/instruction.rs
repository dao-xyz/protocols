use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};
use solana_program::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    system_program,
};

use crate::socials::instruction::SocialInstruction;

use super::{find_channel_program_address, state::ChannelAccount};

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, BorshSchema, PartialEq)]
pub enum ChannelInstruction {
    // Create channel
    CreateChannel {
        #[allow(dead_code)] // but it's not
        owner: Pubkey,
        #[allow(dead_code)] // but it's not
        name: String,
        #[allow(dead_code)] // but it's not
        link: Option<String>,
        #[allow(dead_code)] // but it's not
        channel_account_bump_seed: u8,
    },

    // Update channel (the tail message)
    UpdateChannel {
        #[allow(dead_code)] // but it's not
        link: Option<String>,
    },
}

/// Creates a create user transction
pub fn create_channel_transaction(
    program_id: &Pubkey,
    channel_name: &str,
    owner: &Pubkey,
    link: Option<String>,
    payer: &Pubkey,
) -> Instruction {
    let (channel, channel_account_bump_seed) =
        find_channel_program_address(program_id, channel_name).unwrap();

    Instruction {
        program_id: *program_id,
        data: SocialInstruction::ChannelInstruction(ChannelInstruction::CreateChannel {
            name: channel_name.into(),
            link,
            owner: *owner,
            channel_account_bump_seed,
        })
        .try_to_vec()
        .unwrap(),
        accounts: vec![
            AccountMeta::new(*payer, true),
            AccountMeta::new(*owner, false),
            AccountMeta::new(channel, false),
            AccountMeta::new(system_program::id(), false),
        ],
    }
}

/// Creates a create user transction
pub fn create_update_channel_transacation(
    program_id: &Pubkey,
    channel_name: &str,
    owner: &Pubkey,
    link: Option<String>,
    payer: &Pubkey,
) -> Instruction {
    let (channel, _) = find_channel_program_address(program_id, channel_name).unwrap();
    Instruction {
        program_id: *program_id,
        data: SocialInstruction::ChannelInstruction(ChannelInstruction::UpdateChannel { link })
            .try_to_vec()
            .unwrap(),
        accounts: vec![
            AccountMeta::new(*payer, true),
            AccountMeta::new(*owner, false),
            AccountMeta::new(channel, false),
        ],
    }
}
