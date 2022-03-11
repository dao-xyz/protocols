use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};

use ltag::get_tag_record_program_address;
use solana_program::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    system_program,
};


use crate::{
    find_post_program_address,
    state::{
        post::{PostContent},
        vote_record::get_vote_record_address,
    },
    Vote,
};

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, BorshSchema, PartialEq)]
pub enum CreateVoteConfig {
    Simple,
}

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, BorshSchema, PartialEq)]
pub struct CreatePost {
    pub hash: [u8; 32],
    pub vote_config: CreateVoteConfig,
    pub content: PostContent,
    pub post_bump_seed: u8,
}

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, BorshSchema, PartialEq)]
pub enum PostInstruction {
    // Create channel
    CreatePost(CreatePost),
    Vote {
        vote: Vote,
        vote_record_bump_seed: u8,
    },
    Unvote,
}
pub enum SigningChannelAuthority {
    AuthorityByTag {
        tag: Pubkey,
        authority: Pubkey,
        owner: Pubkey, // signer
    },
}

impl SigningChannelAuthority {
    pub fn add_account_infos(&self, accounts: &mut Vec<AccountMeta>) {
        match &self {
            SigningChannelAuthority::AuthorityByTag {
                tag,
                authority,
                owner,
            } => {
                let tag_record_address =
                    get_tag_record_program_address(&ltag::id(), tag, owner, authority).0;
                accounts.push(AccountMeta::new_readonly(tag_record_address, false));
                accounts.push(AccountMeta::new_readonly(*authority, false));
                accounts.push(AccountMeta::new_readonly(*owner, true));
            }
        }
    }
}

pub fn create_post(
    program_id: &Pubkey,
    payer: &Pubkey,
    channel: &Pubkey,
    hash: &[u8; 32],
    content: &PostContent,
    vote_config: &CreateVoteConfig,
    authority_config: &SigningChannelAuthority,
) -> Instruction {
    let (post_address, post_bump_seed) = find_post_program_address(program_id, hash);
    let mut accounts = vec![
        AccountMeta::new(post_address, false),
        AccountMeta::new(*channel, false),
        AccountMeta::new(*payer, true),
        AccountMeta::new_readonly(system_program::id(), false),
    ];

    authority_config.add_account_infos(&mut accounts);

    Instruction {
        program_id: *program_id,
        data: (PostInstruction::CreatePost(CreatePost {
            /*        creator: *user,
            channel: *channel,
            vote_mint_address: *vote_mint_address, */
            hash: *hash,
            post_bump_seed,
            content: content.clone(),
            vote_config: vote_config.clone(),
        }))
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
    authority_config: &SigningChannelAuthority,
    vote: Vote,
) -> Instruction {
    let (record_address, record_bump_seed) =
        get_vote_record_address(program_id, post, record_owner);
    let mut accounts = vec![
        AccountMeta::new(*post, false),
        AccountMeta::new_readonly(*channel, false),
        AccountMeta::new(record_address, false),
        AccountMeta::new_readonly(*record_owner, true),
        AccountMeta::new(*payer, true),
        AccountMeta::new(system_program::id(), false),
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
    record_owner: &Pubkey,
    destination_info: &Pubkey,
) -> Instruction {
    let (record_address, _) = get_vote_record_address(program_id, post, record_owner);
    let accounts = vec![
        AccountMeta::new(*post, false),
        AccountMeta::new(record_address, false),
        AccountMeta::new_readonly(*record_owner, true),
        AccountMeta::new(*destination_info, false),
        AccountMeta::new(system_program::id(), false),
    ];

    Instruction {
        program_id: *program_id,
        data: (PostInstruction::Unvote).try_to_vec().unwrap(),
        accounts,
    }
}
