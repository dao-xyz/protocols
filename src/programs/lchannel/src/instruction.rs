use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};
use shared::content::ContentSource;
use solana_program::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    system_program,
};

use crate::find_channel_program_address;

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, BorshSchema, PartialEq)]
pub enum ChannelInstruction {
    // Create channel
    CreateChannel {
        #[allow(dead_code)] // but it's not
        creator: Pubkey,
        #[allow(dead_code)] // but it's not
        governence_mint: Pubkey,
        #[allow(dead_code)] // but it's not
        name: String,
        #[allow(dead_code)] // but it's not
        link: Option<ContentSource>,
        #[allow(dead_code)] // but it's not
        channel_account_bump_seed: u8,
        /*         #[allow(dead_code)] // but it's not
        create_rule_address_bump_seed: u8, */
    },

    // Update channel (the tail message)
    UpdateChannel {
        #[allow(dead_code)] // but it's not
        link: Option<ContentSource>,
    },
}

/// Creates a create channel transction
pub fn create_channel_transaction(
    program_id: &Pubkey,
    channel_name: &str,
    creator: &Pubkey,
    governence_mint: &Pubkey,
    link: Option<ContentSource>,
    payer: &Pubkey,
) -> Instruction {
    let (channel, channel_account_bump_seed) =
        find_channel_program_address(program_id, channel_name).unwrap();

    // The create rule address is a rule that defines what the approval critera is to allow the channel to create a new rule

    /* let (create_rule_address, create_rule_address_bump_seed) =
           find_create_rule_associated_program_address(
               program_id,
               &ActionType::ManageRule(RuleUpdateType::Create),
               &channel,
           );
    */
    Instruction {
        program_id: *program_id,
        data: (ChannelInstruction::CreateChannel {
            name: channel_name.into(),
            link,
            governence_mint: *governence_mint,
            creator: *creator,
            channel_account_bump_seed, /* ,
                                       create_rule_address_bump_seed, */
        })
        .try_to_vec()
        .unwrap(),
        accounts: vec![
            AccountMeta::new(*payer, true),
            AccountMeta::new(*creator, false),
            AccountMeta::new(channel, false),
            /*    AccountMeta::new(create_rule_address, false), */
            AccountMeta::new(system_program::id(), false),
        ],
    }
}

/// Creates a create channel transction
pub fn create_update_channel_transacation(
    program_id: &Pubkey,
    channel_name: &str,
    creator: &Pubkey,
    link: Option<ContentSource>,
    payer: &Pubkey,
) -> Instruction {
    let (channel, _) = find_channel_program_address(program_id, channel_name).unwrap();
    Instruction {
        program_id: *program_id,
        data: (ChannelInstruction::UpdateChannel { link })
            .try_to_vec()
            .unwrap(),
        accounts: vec![
            AccountMeta::new(*payer, true),
            AccountMeta::new(*creator, false),
            AccountMeta::new(channel, false),
        ],
    }
}
