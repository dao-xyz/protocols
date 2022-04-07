use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};
use shared::content::ContentSource;
use solana_program::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    system_program,
};

use crate::state::{
    channel::get_channel_program_address,
    channel_authority::{get_channel_authority_address, AuthorityCondition, AuthorityType},
    post::{get_post_program_address, PostContent},
};
use crate::{state::vote_record::get_vote_record_address, Vote};

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, BorshSchema, PartialEq)]
pub enum CreateVoteConfig {
    Simple,
}

pub enum SignedAuthorityCondition {
    Pubkey(Pubkey),
    Tag {
        record: Pubkey,
        owner: Pubkey,
        record_factory: Pubkey,
    },
    None,
}

pub struct SignedAuthority {
    pub channel_authority: Pubkey,
    pub condition: SignedAuthorityCondition,
}
impl SignedAuthority {
    pub fn add_account_infos(&self, accounts: &mut Vec<AccountMeta>) {
        accounts.push(AccountMeta::new_readonly(self.channel_authority, false));
        match &self.condition {
            SignedAuthorityCondition::Tag { owner, record, .. } => {
                accounts.push(AccountMeta::new_readonly(*record, false));
                accounts.push(AccountMeta::new_readonly(*owner, true));
            }
            SignedAuthorityCondition::Pubkey(key) => {
                accounts.push(AccountMeta::new_readonly(*key, true));
            }
            SignedAuthorityCondition::None => {}
        }
    }
}

impl From<SignedAuthority> for AuthorityCondition {
    fn from(signed_authority_condition: SignedAuthority) -> Self {
        match signed_authority_condition.condition {
            SignedAuthorityCondition::Pubkey(key) => AuthorityCondition::Pubkey(key),
            SignedAuthorityCondition::None => AuthorityCondition::None,
            SignedAuthorityCondition::Tag { record_factory, .. } => {
                AuthorityCondition::Tag { record_factory }
            }
        }
    }
}

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, BorshSchema, PartialEq)]
pub enum PostInstruction {
    CreateChannel {
        #[allow(dead_code)] // but it's not
        parent: Option<Pubkey>,

        #[allow(dead_code)] // but it's not
        name: String,

        #[allow(dead_code)] // but it's not
        info: Option<ContentSource>,

        #[allow(dead_code)] // but it's not
        channel_account_bump_seed: u8,

        #[allow(dead_code)] // but it's not
        channel_authority_seed: Pubkey,

        #[allow(dead_code)] // but it's not
        channel_authority_bump_seed: u8,
    },

    DeleteChannel,

    CreateAuthority {
        #[allow(dead_code)] // but it's not
        authority_types: Vec<AuthorityType>,

        #[allow(dead_code)] // but it's not
        condition: AuthorityCondition,

        #[allow(dead_code)] // but it's not
        seed: Pubkey,

        #[allow(dead_code)] // but it's not
        bump_seed: u8,
    },

    DeleteAuthority,

    UpdateChannelInfo {
        #[allow(dead_code)] // but it's not
        info: Option<ContentSource>,
    },

    CreatePost {
        #[allow(dead_code)] // but it's not
        content: PostContent,

        #[allow(dead_code)] // but it's not
        hash: [u8; 32],

        #[allow(dead_code)] // but it's not
        is_child: bool,

        #[allow(dead_code)] // but it's not
        vote_config: CreateVoteConfig,

        #[allow(dead_code)] // but it's not
        post_bump_seed: u8,
    },

    Vote {
        #[allow(dead_code)] // but it's not
        vote: Vote,

        #[allow(dead_code)] // but it's not
        vote_record_bump_seed: u8,
    },
    Unvote,
}

pub fn create_post(
    program_id: &Pubkey,

    // Accounts
    channel: &Pubkey,
    owner: &Pubkey,
    payer: &Pubkey,

    // Args
    content: PostContent,
    hash: &[u8; 32],
    parent_post: Option<Pubkey>,
    vote_config: &CreateVoteConfig,
    authority_config: &SignedAuthority,
) -> Instruction {
    let (post_address, post_bump_seed) = get_post_program_address(program_id, hash);
    let mut accounts = vec![
        AccountMeta::new(post_address, false),
        AccountMeta::new(*channel, false),
        AccountMeta::new_readonly(*owner, true),
        AccountMeta::new(*payer, true),
        AccountMeta::new_readonly(system_program::id(), false),
    ];

    if let Some(parent) = parent_post {
        accounts.push(AccountMeta::new_readonly(parent, false));
    }

    authority_config.add_account_infos(&mut accounts);

    Instruction {
        program_id: *program_id,
        data: (PostInstruction::CreatePost {
            hash: *hash,
            post_bump_seed,
            is_child: parent_post.is_some(),
            content,
            vote_config: vote_config.clone(),
        })
        .try_to_vec()
        .unwrap(),
        accounts,
    }
}

pub fn cast_vote(
    program_id: &Pubkey,
    payer: &Pubkey,
    post: &Pubkey,
    channel: &Pubkey,
    record_owner: &Pubkey,
    vote: Vote,
    authority_config: &SignedAuthority,
) -> Instruction {
    let (record_address, record_bump_seed) =
        get_vote_record_address(program_id, post, record_owner);
    let mut accounts = vec![
        AccountMeta::new(*post, false),
        AccountMeta::new_readonly(*channel, false),
        AccountMeta::new(record_address, false),
        AccountMeta::new_readonly(*record_owner, true),
        AccountMeta::new(*payer, true),
        AccountMeta::new_readonly(system_program::id(), false),
    ];

    authority_config.add_account_infos(&mut accounts);

    Instruction {
        program_id: *program_id,
        data: (PostInstruction::Vote {
            vote,
            vote_record_bump_seed: record_bump_seed,
        })
        .try_to_vec()
        .unwrap(),
        accounts,
    }
}

pub fn uncast_vote(
    program_id: &Pubkey,
    post: &Pubkey,
    channel: &Pubkey,
    record_owner: &Pubkey,
    destination_info: &Pubkey,
    authority_config: &SignedAuthority,
) -> Instruction {
    let (record_address, _) = get_vote_record_address(program_id, post, record_owner);

    let mut accounts = vec![
        AccountMeta::new(*post, false),
        AccountMeta::new_readonly(*channel, false),
        AccountMeta::new(record_address, false),
        AccountMeta::new_readonly(*record_owner, true),
        AccountMeta::new(*destination_info, false),
    ];

    authority_config.add_account_infos(&mut accounts);

    Instruction {
        program_id: *program_id,
        data: (PostInstruction::Unvote).try_to_vec().unwrap(),
        accounts,
    }
}

/// Creates a create channel transction
pub fn create_channel(
    program_id: &Pubkey,

    // Accounts
    parent: Option<Pubkey>,
    creator: &Pubkey,
    authority: &Pubkey,
    payer: &Pubkey,

    // Args
    channel_name: &str,
    info: Option<ContentSource>,
    channel_authority_seed: &Pubkey,
    authority_config: Option<&SignedAuthority>,
) -> Instruction {
    let (channel, channel_account_bump_seed) =
        get_channel_program_address(program_id, channel_name).unwrap();
    let (channel_authority, channel_authority_bump_seed) =
        get_channel_authority_address(program_id, &channel, channel_authority_seed);
    let mut accounts = vec![
        AccountMeta::new(channel, false),
        AccountMeta::new_readonly(*creator, true),
        AccountMeta::new(channel_authority, false),
        AccountMeta::new_readonly(*authority, true),
        AccountMeta::new(*payer, true),
        AccountMeta::new(system_program::id(), false),
    ];
    // Only needed (is Some) if channel is a subchannel
    if let Some(authority_config) = authority_config {
        accounts.push(AccountMeta::new_readonly(parent.unwrap(), false));
        authority_config.add_account_infos(&mut accounts);
    }

    Instruction {
        program_id: *program_id,
        data: (PostInstruction::CreateChannel {
            name: channel_name.into(),
            info,
            parent,
            channel_account_bump_seed,
            channel_authority_seed: *channel_authority_seed,
            channel_authority_bump_seed,
        })
        .try_to_vec()
        .unwrap(),
        accounts,
    }
}

/// Creates a update info transaction
pub fn update_info(
    program_id: &Pubkey,
    channel: &Pubkey,
    info: Option<ContentSource>,
    authority_config: &SignedAuthority,
) -> Instruction {
    let mut accounts = vec![AccountMeta::new(*channel, false)];

    authority_config.add_account_infos(&mut accounts);

    Instruction {
        program_id: *program_id,
        data: (PostInstruction::UpdateChannelInfo { info })
            .try_to_vec()
            .unwrap(),
        accounts,
    }
}

pub fn create_channel_authority(
    program_id: &Pubkey,

    // Accounts
    channel: &Pubkey,
    payer: &Pubkey,

    // Args
    authority_types: &Vec<AuthorityType>,
    condition: &AuthorityCondition,
    seed: &Pubkey,
    authority_config: &SignedAuthority,
) -> Instruction {
    let (authority_address, authority_address_bump_seed) =
        get_channel_authority_address(program_id, channel, seed);

    let mut accounts = vec![
        AccountMeta::new(authority_address, false),
        AccountMeta::new_readonly(*channel, false),
        AccountMeta::new(*payer, true),
        AccountMeta::new(system_program::id(), false),
    ];

    authority_config.add_account_infos(&mut accounts);

    Instruction {
        program_id: *program_id,
        data: (PostInstruction::CreateAuthority {
            authority_types: authority_types.clone(),
            bump_seed: authority_address_bump_seed,
            condition: condition.clone(),
            seed: *seed,
        })
        .try_to_vec()
        .unwrap(),
        accounts,
    }
}

pub fn delete_channel_authority(
    program_id: &Pubkey,

    // Accounts
    channel_authority: &Pubkey,
    channel: &Pubkey,
    payer: &Pubkey,

    // Args
    authority_config: &SignedAuthority,
) -> Instruction {
    let mut accounts = vec![
        AccountMeta::new(*channel_authority, false),
        AccountMeta::new_readonly(*channel, false),
        AccountMeta::new(*payer, true),
        AccountMeta::new(system_program::id(), false),
    ];

    authority_config.add_account_infos(&mut accounts);

    Instruction {
        program_id: *program_id,
        data: PostInstruction::DeleteAuthority.try_to_vec().unwrap(),
        accounts,
    }
}
