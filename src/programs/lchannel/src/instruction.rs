use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};
use shared::content::ContentSource;
use solana_program::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    system_program,
};

use crate::state::{find_channel_program_address, ChannelAuthority};

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, BorshSchema, PartialEq)]
pub enum ChannelInstruction {
    // Create channel
    CreateChannel {
        #[allow(dead_code)] // but it's not
        parent: Option<Pubkey>,
        #[allow(dead_code)] // but it's not
        name: String,
        #[allow(dead_code)] // but it's not
        link: Option<ContentSource>,
        #[allow(dead_code)] // but it's not
        channel_account_bump_seed: u8,

        // Tag that lets users create posts
        #[allow(dead_code)] // but it's not
        channel_authority_config: ChannelAuthority,
    },

    // Update channel
    UpdateInfo {
        #[allow(dead_code)] // but it's not
        link: Option<ContentSource>,
    },

    // Update authority
    UpdateAuthority(Pubkey),
}

/// Creates a create channel transction
pub fn create_channel(
    program_id: &Pubkey,
    channel_name: &str,
    creator: &Pubkey,
    parent_and_authority: Option<(Pubkey, Pubkey)>,
    link: Option<ContentSource>,
    channel_authority_config: ChannelAuthority,
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
    let mut accounts = vec![
        AccountMeta::new(channel, false),
        AccountMeta::new_readonly(*creator, true),
        AccountMeta::new(*payer, true),
        /*    AccountMeta::new(create_rule_address, false), */
        AccountMeta::new(system_program::id(), false),
    ];
    let mut parent_address: Option<Pubkey> = None;
    if parent_and_authority.is_some() {
        let (parent, parent_authority) = parent_and_authority.unwrap();
        parent_address = Some(parent);
        accounts.push(AccountMeta::new_readonly(parent, false));
        accounts.push(AccountMeta::new_readonly(parent_authority, true));
    }
    Instruction {
        program_id: *program_id,
        data: (ChannelInstruction::CreateChannel {
            name: channel_name.into(),
            link,
            parent: parent_address,
            channel_account_bump_seed,
            channel_authority_config,
        })
        .try_to_vec()
        .unwrap(),
        accounts,
    }
}

/// Creates a update info transaction
pub fn create_update_info_transacation(
    program_id: &Pubkey,
    channel_name: &str,
    link: Option<ContentSource>,
    authority: &Pubkey,
) -> Instruction {
    let (channel, _) = find_channel_program_address(program_id, channel_name).unwrap();
    Instruction {
        program_id: *program_id,
        data: (ChannelInstruction::UpdateInfo { link })
            .try_to_vec()
            .unwrap(),
        accounts: vec![
            AccountMeta::new(channel, false),
            AccountMeta::new(*authority, true),
        ],
    }
}

/// Creates a update info transaction
pub fn create_update_authority_transacation(
    program_id: &Pubkey,
    channel_name: &str,
    new_authority: &Pubkey,
    authority: &Pubkey,
) -> Instruction {
    let (channel, _) = find_channel_program_address(program_id, channel_name).unwrap();
    Instruction {
        program_id: *program_id,
        data: (ChannelInstruction::UpdateAuthority(*new_authority))
            .try_to_vec()
            .unwrap(),
        accounts: vec![
            AccountMeta::new(channel, false),
            AccountMeta::new(*authority, true),
        ],
    }
}
