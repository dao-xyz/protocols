use solana_program::pubkey::Pubkey;

pub mod instruction;
pub mod processor;
pub mod state;

use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};
use solana_program::{
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    program_pack::Pack,
    rent::Rent,
    system_instruction,
    sysvar::Sysvar,
};
use spl_token::{instruction::initialize_mint, state::Mint};

use crate::tokens::spl_utils::{
    create_mint_authority_program_address_seeds, create_mint_escrow_program_address_seeds,
    find_mint_authority_program_address, find_mint_escrow_program_address, MINT_SEED,
};

use self::state::{Action, ActionType, RuleUpdateType, VotingRuleUpdate};

/// Seed for UPVOTE
const USER: &[u8] = b"user";

/// Seed for UPVOTE
const UPVOTE: &[u8] = b"up";

/// Seed for downvote
const DOWNVOTE: &[u8] = b"down";

/// Seed for MINT
const MINT: &[u8] = b"mint";

// Seed for stats
const STATS: &[u8] = b"stats";

const RULE: &[u8] = b"rule";

#[derive(Copy, Clone, Debug, BorshSerialize, BorshDeserialize, BorshSchema, PartialEq)]
pub enum Vote {
    UP = 0,
    DOWN = 1,
}

pub fn find_escrow_program_address(program_id: &Pubkey, post: &Pubkey) -> (Pubkey, u8) {
    find_mint_escrow_program_address(program_id, post)
}

pub fn create_escrow_program_address_seeds<'a>(
    post: &'a Pubkey,
    bump_seed: &'a [u8],
) -> [&'a [u8]; 3] {
    create_mint_escrow_program_address_seeds(post, bump_seed)
}

pub fn create_post_mint_program_account<'a>(
    post: &Pubkey,
    vote: Vote,
    mint_info: &AccountInfo<'a>,
    mint_bump_seed: u8,
    mint_authority_info: &AccountInfo<'a>,
    payer_info: &AccountInfo<'a>,
    rent_info: &AccountInfo<'a>,
    token_program_info: &AccountInfo<'a>,
    system_info: &AccountInfo<'a>,
    program_id: &Pubkey,
) -> ProgramResult {
    let rent = Rent::get()?;
    let mint_rent = rent.minimum_balance(Mint::LEN);
    let decimals = spl_token::native_mint::DECIMALS; // for now
    let mint_bump_seed = &[mint_bump_seed];
    let mint_account_seeds = match vote {
        Vote::UP => create_post_upvote_mint_program_address_seeds(post, mint_bump_seed),
        Vote::DOWN => create_post_downvote_mint_program_address_seeds(post, mint_bump_seed),
    };

    let address = Pubkey::create_program_address(&mint_account_seeds, program_id).unwrap();
    if mint_info.key != &address {
        msg!(
            "Create account with PDA: {:?} was requested while PDA: {:?} was expected",
            mint_info.key,
            address
        );
        return Err(ProgramError::InvalidSeeds);
    }
    invoke_signed(
        &system_instruction::create_account(
            payer_info.key,
            mint_info.key,
            mint_rent,
            Mint::LEN as u64,
            &spl_token::id(),
        ),
        &[
            payer_info.clone(),
            mint_info.clone(),
            system_info.clone(),
            token_program_info.clone(),
        ],
        &[&mint_account_seeds],
    )?;

    invoke(
        &initialize_mint(
            &spl_token::id(),
            mint_info.key,
            mint_authority_info.key,
            None,
            decimals,
        )?,
        &[mint_info.clone(), rent_info.clone()],
    )?;
    Ok(())
}

pub fn find_post_program_address(program_id: &Pubkey, hash: &[u8; 32]) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[hash], program_id)
}

/// Find address for the token upvote mint for the post account
pub fn find_post_upvote_mint_program_address(program_id: &Pubkey, post: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[MINT_SEED, UPVOTE, &post.to_bytes()], program_id)
}

/// Create post mint upvote program address
pub fn create_post_upvote_mint_program_address_seeds<'a>(
    post: &'a Pubkey,
    bump_seed: &'a [u8],
) -> [&'a [u8]; 4] {
    [MINT_SEED, UPVOTE, post.as_ref(), bump_seed]
}

/// Find post stats account address
pub fn find_post_stats_program_address(program_id: &Pubkey, post: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[STATS, &post.to_bytes()], program_id)
}

/// Create post stats acount address
pub fn create_post_stats_program_address_seeds<'a>(
    post: &'a Pubkey,
    bump_seed: &'a [u8],
) -> [&'a [u8]; 3] {
    [STATS, post.as_ref(), bump_seed]
}

/// Find address for the token downvote mint for the post account
pub fn find_post_downvote_mint_program_address(program_id: &Pubkey, post: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[MINT_SEED, DOWNVOTE, &post.to_bytes()], program_id)
}

/// Create post mint downvote program address
pub fn create_post_downvote_mint_program_address_seeds<'a>(
    post: &'a Pubkey,
    bump_seed: &'a [u8],
) -> [&'a [u8]; 4] {
    [MINT_SEED, DOWNVOTE, post.as_ref(), bump_seed]
}

/// Find rule account address
pub fn find_create_rule_associated_program_address(
    program_id: &Pubkey,
    action_type: &ActionType,
    channel: &Pubkey,
) -> (Pubkey, u8) {
    match action_type {
        ActionType::DeletePost => {
            Pubkey::find_program_address(&[RULE, b"delete", channel.as_ref()], program_id)
        }
        ActionType::CustomEvent(event_type) => {
            Pubkey::find_program_address(&[RULE, event_type.as_ref(), channel.as_ref()], program_id)
        }
        ActionType::ManageRule(manage_rule) => match manage_rule {
            RuleUpdateType::Create => {
                Pubkey::find_program_address(&[RULE, b"rule_create", channel.as_ref()], program_id)
            }
            RuleUpdateType::Delete => {
                Pubkey::find_program_address(&[RULE, b"rule_delete", channel.as_ref()], program_id)
            }
        },
        ActionType::TransferTreasury => Pubkey::find_program_address(
            &[RULE, b"transfer_treasury", channel.as_ref()],
            program_id,
        ),
    }
}

/// Create rule account address
pub fn create_rule_associated_program_address_seeds<'a>(
    channel: &'a Pubkey,
    action_type: &'a ActionType,
    bump_seed: &'a [u8],
) -> [&'a [u8]; 4] {
    match action_type {
        ActionType::CustomEvent(key) => [RULE, key.as_ref(), channel.as_ref(), bump_seed],
        ActionType::DeletePost => [RULE, b"delete", channel.as_ref(), bump_seed],
        ActionType::ManageRule(manage_rule) => match manage_rule {
            RuleUpdateType::Create => [RULE, b"rule_create", channel.as_ref(), bump_seed],
            RuleUpdateType::Delete => [RULE, b"rule_delete", channel.as_ref(), bump_seed],
        },
        ActionType::TransferTreasury => [RULE, b"transfer_treasury", channel.as_ref(), bump_seed],
    }
}

/// Find address for the token mint authority for the post account
pub fn find_post_mint_authority_program_address(
    program_id: &Pubkey,
    post: &Pubkey,
) -> (Pubkey, u8) {
    find_mint_authority_program_address(program_id, post)
}

/// Create post mint authority program address
pub fn create_post_mint_authority_program_address_seeds<'a>(
    post: &'a Pubkey,
    bump_seed: &'a [u8],
) -> [&'a [u8]; 3] {
    create_mint_authority_program_address_seeds(post, bump_seed)
}
