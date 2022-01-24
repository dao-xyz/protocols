use crate::{
    social::{instruction::OffsetCreateSettings, Vote},
    tokens::spl_utils::{
        create_program_account_mint_account_with_seed, create_program_token_account, spl_mint_to,
    },
};
use solana_program::{
    account_info::AccountInfo,
    msg,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::{Pubkey, PubkeyError},
    rent::Rent,
    system_instruction::{self, create_account},
    sysvar::Sysvar,
};
use spl_token_swap::curve::offset::OffsetCurve;

use super::create_post_mint_authority_program_address_seeds;

/// Seed for SWAP account
const SWAP: &[u8] = b"swap";

/// Seed for swap destination token account
const SWAP_MINT: &[u8] = b"swap_mint";

/// Seed for swap fee account
const SWAP_FEE: &[u8] = b"swap_fee";

/// Seed for swap destination token account
const SWAP_DEPOSIT: &[u8] = b"swap_deposit";

/// Seed for swap destination token account
const SWAP_ESCROW: &[u8] = b"swap_escrow";

/// Find swap program address
pub fn find_swap_authority_program_address(swap: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[swap.as_ref()], &spl_token_swap::id())
}

/// Create swap program address
pub fn create_swap_authority_program_address_seeds<'a>(
    swap: &'a Pubkey,
    bump_seed: &'a [u8],
) -> [&'a [u8]; 2] {
    [swap.as_ref(), bump_seed]
}

/// Find swap mint program address
pub fn find_swap_mint_program_address(program_id: &Pubkey, swap: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[SWAP_MINT, &swap.to_bytes()], program_id)
}

/// Create swap mint program address
pub fn create_swap_mint_program_address_seeds<'a>(
    swap: &'a Pubkey,
    bump_seed: &'a [u8],
) -> [&'a [u8]; 3] {
    [SWAP_MINT, swap.as_ref(), bump_seed]
}

/// Find swap program address
pub fn find_swap_program_address(program_id: &Pubkey, post: &Pubkey, vote: &Vote) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[SWAP, post.as_ref(), &[*vote as u8]], program_id)
}

/// Create swap program address
pub fn create_swap_program_address_seeds<'a>(
    post: &'a Pubkey,
    vote: &'a [u8],
    bump_seed: &'a [u8],
) -> [&'a [u8]; 4] {
    [SWAP, post.as_ref(), vote, bump_seed]
}

// Find swap fee account  program address for a mint
pub fn find_swap_token_fee_account_program_address(
    program_id: &Pubkey,
    swap: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[SWAP_FEE, &swap.to_bytes()], program_id)
}

/// Create swap fee account program address for a mint
pub fn create_swap_token_fee_account_program_address_seeds<'a>(
    swap: &'a Pubkey,
    bump_seed: &'a [u8],
) -> [&'a [u8]; 3] {
    [SWAP_FEE, swap.as_ref(), bump_seed]
}

// Find swap fee account  program address for a mint
pub fn find_swap_token_deposit_account_program_address(
    program_id: &Pubkey,
    swap: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[SWAP_DEPOSIT, &swap.to_bytes()], program_id)
}

/// Create swap fee account program address for a mint
pub fn create_swap_token_deposit_account_program_address<'a>(
    swap: &'a Pubkey,
    bump_seed: &'a [u8],
) -> [&'a [u8]; 3] {
    [SWAP_DEPOSIT, swap.as_ref(), bump_seed]
}

pub fn find_utility_account_program_address(
    program_id: &Pubkey,
    post: &Pubkey,
    vote: &Vote,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[SWAP_ESCROW, post.as_ref(), &[*vote as u8]], program_id)
}

pub fn create_utility_account_program_address_seeds<'a>(
    post: &'a Pubkey,
    vote: &'a [u8],
    bump_seed: &'a [u8],
) -> [&'a [u8]; 4] {
    [SWAP_ESCROW, post.as_ref(), vote, bump_seed]
}

/// Find swap token account  program address for a mint
pub fn find_swap_token_account_program_address(program_id: &Pubkey, mint: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[SWAP, &mint.to_bytes()], program_id)
}

/// Create swap token account program address for a mint
pub fn create_swap_token_account_program_address_seeds<'a>(
    mint: &'a Pubkey,
    bump_seed: &'a [u8],
) -> [&'a [u8]; 3] {
    [SWAP, mint.as_ref(), bump_seed]
}

pub fn create_and_initalize_swap_pool<'a>(
    program_id: &Pubkey,
    payer_account: &AccountInfo<'a>,
    post_account_info: &AccountInfo<'a>,
    swap_account_info: &AccountInfo<'a>,
    swap_authority_info: &AccountInfo<'a>,
    swap_pool_mint: &AccountInfo<'a>,
    swap_pool_fee_token_account: &AccountInfo<'a>,
    swap_pool_deposit_token_account: &AccountInfo<'a>,
    utility_mint_account_info: &AccountInfo<'a>,
    utility_token_account_info: &AccountInfo<'a>,
    target_mint_account_info: &AccountInfo<'a>,
    target_token_account_info: &AccountInfo<'a>,
    rent_info: &AccountInfo<'a>,
    token_program_info: &AccountInfo<'a>,
    post_mint_authority_info: &AccountInfo<'a>,
    post_mint_authority_bump_seed: u8,
    system_account: &AccountInfo<'a>,
    swap_program_info: &AccountInfo<'a>,
    settings: &OffsetCreateSettings,
    offset: u64,
    vote: &Vote,
    rent: &Rent,
) -> Result<(), ProgramError> {
    // Creates a swap pool with offset b equal to supply of a (minted by authority 'mint_authority_info')
    let vote_mint_authority_bump_seeds = &[post_mint_authority_bump_seed];
    let vote_mint_authority_seeds = create_post_mint_authority_program_address_seeds(
        post_account_info.key,
        vote_mint_authority_bump_seeds,
    );

    let (swap_account, swap_bump_seed_expected) =
        find_swap_program_address(program_id, post_account_info.key, vote);

    let swap_bump_seeds = &[settings.swap_bump_seed];
    let vote_seeds = &[*vote as u8];
    let swap_seeds =
        create_swap_program_address_seeds(&post_account_info.key, vote_seeds, swap_bump_seeds);

    if &swap_account != swap_account_info.key {
        msg!("Invalid swap account address");
        return Err(ProgramError::InvalidAccountData);
    }

    if settings.swap_bump_seed != swap_bump_seed_expected {
        msg!("Invalid swap bump seed");
        return Err(ProgramError::InvalidArgument);
    }

    let swap_authority_bump_seeds = &[settings.swap_authority_bump_seed];
    let swap_authority_seeds = create_swap_authority_program_address_seeds(
        swap_account_info.key,
        swap_authority_bump_seeds,
    );
    let swap_authority =
        Pubkey::create_program_address(&swap_authority_seeds, &spl_token_swap::id()).unwrap();

    if &swap_authority != swap_authority_info.key {
        msg!("Invalid swap authority address");
        return Err(ProgramError::InvalidAccountData);
    }

    // Initialize accounts required for swap pool

    let create_account_instruction = create_account(
        payer_account.key,
        &swap_account,
        rent.minimum_balance(spl_token_swap::state::SwapVersion::LATEST_LEN),
        spl_token_swap::state::SwapVersion::LATEST_LEN as u64,
        &spl_token_swap::id(),
    );
    invoke_signed(
        &create_account_instruction,
        &[
            payer_account.clone(),
            swap_account_info.clone(),
            system_account.clone(),
        ],
        &[&swap_seeds],
    )?;

    // Supply account for the utility token
    create_program_token_account(
        utility_token_account_info,
        &create_utility_account_program_address_seeds(
            post_account_info.key,
            &[*vote as u8],
            &[settings.token_utility_account_bump_seed],
        ),
        utility_mint_account_info,
        swap_authority_info,
        payer_account,
        rent_info,
        token_program_info,
        system_account,
        program_id,
    )?;

    // Supply account for the upvote token

    create_program_token_account(
        target_token_account_info,
        &create_swap_token_account_program_address_seeds(
            target_mint_account_info.key,
            &[settings.token_target_account_bump_seed],
        ),
        target_mint_account_info,
        swap_authority_info,
        payer_account,
        rent_info,
        token_program_info,
        system_account,
        program_id,
    )?;

    spl_mint_to(
        target_token_account_info,
        target_mint_account_info,
        post_mint_authority_info,
        &vote_mint_authority_seeds,
        offset,
        program_id,
    )?;

    // Pool token mint
    create_program_account_mint_account_with_seed(
        swap_pool_mint,
        &create_swap_mint_program_address_seeds(&swap_account, &[settings.swap_mint_bump_seed]),
        swap_authority_info,
        payer_account,
        rent_info,
        token_program_info,
        system_account,
        program_id,
    )?;

    // Swap fee token account
    create_program_token_account(
        swap_pool_fee_token_account,
        &create_swap_token_fee_account_program_address_seeds(
            &swap_account,
            &[settings.swap_fee_token_account_bump_seed],
        ),
        swap_pool_mint,
        post_mint_authority_info,
        payer_account,
        rent_info,
        token_program_info,
        system_account,
        program_id,
    )?;

    // Swap deposit token account
    create_program_token_account(
        swap_pool_deposit_token_account,
        &create_swap_token_deposit_account_program_address(
            &swap_account,
            &[settings.swap_deposit_token_account_bump_seed],
        ),
        swap_pool_mint,
        post_mint_authority_info,
        payer_account,
        rent_info,
        token_program_info,
        system_account,
        program_id,
    )?;

    // Create swap pool

    // Initialize SWAP pool
    let curve = spl_token_swap::curve::base::SwapCurve {
        curve_type: spl_token_swap::curve::base::CurveType::Offset,
        calculator: Box::new(OffsetCurve {
            token_b_offset: offset,
        }),
    };
    //https://github.com/thetardigrades/SolanaGameServer/blob/f706b842427caac22eb3647fee496e6753094c6c/third_party_pinned/hearttoken_old/tests/scratch.rs_
    invoke_signed(
        &spl_token_swap::instruction::initialize(
            &spl_token_swap::id(),
            token_program_info.key,
            swap_account_info.key,
            swap_authority_info.key,
            target_token_account_info.key,
            utility_token_account_info.key,
            swap_pool_mint.key,
            swap_pool_fee_token_account.key,
            swap_pool_deposit_token_account.key,
            settings.swap_authority_bump_seed, // bump seed
            spl_token_swap::curve::fees::Fees {
                // 0 fees for now
                trade_fee_numerator: 0,
                trade_fee_denominator: 1,
                owner_trade_fee_numerator: 0,
                owner_trade_fee_denominator: 1,
                owner_withdraw_fee_numerator: 0,
                owner_withdraw_fee_denominator: 1,
                host_fee_numerator: 0,
                host_fee_denominator: 1,
            },
            curve,
        )?,
        &[
            swap_account_info.clone(),
            swap_authority_info.clone(),
            target_token_account_info.clone(),
            utility_token_account_info.clone(),
            swap_pool_mint.clone(),
            swap_pool_fee_token_account.clone(),
            swap_pool_deposit_token_account.clone(),
            token_program_info.clone(),
            swap_program_info.clone(),
        ],
        &[&swap_seeds],
    )?;
    Ok(())
}
