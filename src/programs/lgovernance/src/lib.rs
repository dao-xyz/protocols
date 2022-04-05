pub mod accounts;
pub mod entrypoint;
pub mod error;
pub mod instruction;
pub mod pack;
pub mod processor;
pub mod shared;
pub mod state;
pub mod tokens;
solana_program::declare_id!("GhhdZ7v99edo9v6XmitqEoKT5jev1mpCpVWim6bgKsh1");

use solana_program::pubkey::Pubkey;

/* use crate::tokens::spl_utils::{
    create_authority_program_address_seeds, create_mint_escrow_program_address_seeds,
    find_authority_program_address, find_mint_escrow_program_address, MINT_SEED,
};
 */
/// Seed for UPVOTE

const PROGRAM_AUTHORITY_SEED: &[u8] = b"p_authority";

const DELEGATEE_SEED: &[u8] = b"delegatee";
/*
#[derive(Copy, Clone, Debug, BorshSerialize, BorshDeserialize, BorshSchema, PartialEq)]
pub enum Vote {
    Up = 0,
    Down = 1,
} */

/*
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
        Vote::Up => create_post_upvote_mint_program_address_seeds(post, mint_bump_seed),
        Vote::Down => create_post_downvote_mint_program_address_seeds(post, mint_bump_seed),
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
 */

/*
/// Find scope account address
pub fn find_create_scope_associated_program_address(
    program_id: &Pubkey,
    action_type: &ActionType,
    channel: &Pubkey,
) -> (Pubkey, u8) {
    match action_type {
        ActionType::DeletePost => {
            Pubkey::find_program_address(&[scope, b"delete", channel.as_ref()], program_id)
        }
        ActionType::CustomEvent(event_type) => {
            Pubkey::find_program_address(&[scope, event_type.as_ref(), channel.as_ref()], program_id)
        }
        ActionType::ManageScope(manage_scope) => match manage_scope {
            ScopeUpdateType::Create => {
                Pubkey::find_program_address(&[scope, b"scope_create", channel.as_ref()], program_id)
            }
            ScopeUpdateType::Delete => {
                Pubkey::find_program_address(&[scope, b"scope_delete", channel.as_ref()], program_id)
            }
        },
        ActionType::Treasury(treasury_action) => match treasury_action {
            TreasuryActionType::Transfer { from, to } => Pubkey::find_program_address(
                &[
                    from.as_ref().map_or(b"treasury_from", |key| key.as_ref()),
                    to.as_ref().map_or(b"treasury_to", |key| key.as_ref()),
                    channel.as_ref(),
                ],
                program_id,
            ),
            TreasuryActionType::Create => Pubkey::find_program_address(
                &[scope, b"treasury_create", channel.as_ref()],
                program_id,
            ),
        },
    }
}

/// Create scope account address
pub fn create_scope_associated_program_address_seeds<'a>(
    channel: &'a Pubkey,
    action_type: &'a ActionType,
    bump_seed: &'a [u8],
) -> [&'a [u8]; 4] {
    match action_type {
        ActionType::CustomEvent(key) => [scope, key.as_ref(), channel.as_ref(), bump_seed],
        ActionType::DeletePost => [scope, b"delete", channel.as_ref(), bump_seed],
        ActionType::ManageScope(manage_scope) => match manage_scope {
            ScopeUpdateType::Create => [scope, b"scope_create", channel.as_ref(), bump_seed],
            ScopeUpdateType::Delete => [scope, b"scope_delete", channel.as_ref(), bump_seed],
        },
        ActionType::Treasury(treasury_action) => match treasury_action {
            TreasuryActionType::Transfer { from, to } => [
                from.as_ref().map_or(b"treasury_from", |key| key.as_ref()),
                to.as_ref().map_or(b"treasury_to", |key| key.as_ref()),
                channel.as_ref(),
                bump_seed,
            ],
            TreasuryActionType::Create => [scope, b"treasury_create", channel.as_ref(), bump_seed],
        },
    }
} */
/*
/// Find address for the token mint authority for the post account
pub fn find_post_mint_authority_program_address(
    program_id: &Pubkey,
    post: &Pubkey,
) -> (Pubkey, u8) {
    find_authority_program_address(program_id, post)
}

/// Create post mint authority program address
pub fn create_post_mint_authority_program_address_seeds<'a>(
    post: &'a Pubkey,
    bump_seed: &'a [u8],
) -> [&'a [u8]; 3] {
    create_authority_program_address_seeds(post, bump_seed)
}
 */
/// Find treasury account address

pub fn find_treasury_token_account_address(
    program_id: &Pubkey,
    channel: &Pubkey,
    spl_token_mint_address: &Pubkey,
    token_program_id: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            &channel.to_bytes(),
            &token_program_id.to_bytes(),
            &spl_token_mint_address.to_bytes(),
        ],
        program_id,
    )
}

pub fn create_treasury_token_account_address_seeds<'a>(
    channel: &'a Pubkey,
    spl_token_mint_address: &'a Pubkey,
    token_program_id: &'a Pubkey,
    bump_seed: &'a [u8],
) -> [&'a [u8]; 4] {
    [
        channel.as_ref(),
        token_program_id.as_ref(),
        spl_token_mint_address.as_ref(),
        bump_seed,
    ]
}
