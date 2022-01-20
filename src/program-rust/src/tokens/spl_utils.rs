use solana_program::{
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::{Pubkey, PubkeyError},
    rent::Rent,
    system_instruction,
    sysvar::Sysvar,
};

use spl_associated_token_account::get_associated_token_address;
use spl_token::{
    instruction::{burn, burn_checked, initialize_account, initialize_mint, mint_to},
    state::{Mint, Multisig},
};

pub const MINT_SEED: &[u8] = b"mint";
pub const UTILITY_MINT: &[u8] = b"utility";

pub const MINT_AUTHORTY_SEED: &[u8] = b"authority";
pub const ESCROW_ACCOUNT_SEED: &[u8] = b"escrow";

/// Find utility mint token address (unique/fixed for program)
pub fn find_utility_mint_program_address(program_id: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[MINT_SEED, UTILITY_MINT], program_id)
}

/// Create mint address (unique/fixed for program)
pub fn create_utility_mint_program_address_seeds<'a>(bump_seed: &'a [u8]) -> [&'a [u8]; 3] {
    [MINT_SEED, UTILITY_MINT, bump_seed]
}

/// Find mint address
pub fn find_mint_program_address(program_id: &Pubkey, some_account: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[MINT_SEED, &some_account.to_bytes()], program_id)
}

/// Create mint address
pub fn create_mint_program_address_seeds<'a>(
    account_seed: &'a Pubkey,
    bump_seed: &'a [u8],
) -> [&'a [u8]; 3] {
    [MINT_SEED, account_seed.as_ref(), bump_seed]
}

/// Generate mint authority address
pub fn find_mint_authority_program_address(program_id: &Pubkey, mint: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[MINT_AUTHORTY_SEED, &mint.to_bytes()], program_id)
}

/// Create mint authority address
pub fn create_mint_authority_program_address_seeds<'a>(
    mint: &'a Pubkey,
    bump_seed: &'a [u8],
) -> [&'a [u8]; 3] {
    [MINT_AUTHORTY_SEED, mint.as_ref(), bump_seed]
}

/// Generate mint escrow address
pub fn find_mint_escrow_program_address(program_id: &Pubkey, mint: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[ESCROW_ACCOUNT_SEED, &mint.to_bytes()], program_id)
}

/// Create mint authority address
pub fn create_mint_escrow_program_address_seeds<'a>(
    mint: &'a Pubkey,
    bump_seed: &'a [u8],
) -> [&'a [u8]; 3] {
    [ESCROW_ACCOUNT_SEED, mint.as_ref(), bump_seed]
}

/*
pub fn create_mint_authority_account_seeds<'a>(
    mint_account: &'a Pubkey,
    mint_authority_bump_seed: &'a [u8],
) -> [&'a [u8]; 2] {
    [mint_account.as_ref(), mint_authority_bump_seed]
}

pub fn create_user_post_token_account_seeds<'a>(
    user_account: &'a Pubkey,
    post_account: &'a Pubkey,
    user_post_token_account_bump_seeds: &'a [u8],
) -> [&'a [u8]; 3] {
    [
        user_account.as_ref(),
        post_account.as_ref(),
        user_post_token_account_bump_seeds,
    ]
}

pub fn create_escrow_account_bump_seeds<'a>(
    post_account: &'a Pubkey,
    escrow_account_bump_seed: &'a [u8],
) -> [&'a [u8]; 3] {
    [
        ESCROW_ACCOUNT_SEED,
        post_account.as_ref(),
        escrow_account_bump_seed,
    ]
} */

pub fn create_program_account_mint_account<'a>(
    mint_info: &AccountInfo<'a>,
    mint_account_seed: &Pubkey,
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
    let decimals = 9; // for now
    let mint_bump_seed = &[mint_bump_seed];
    let mint_account_seeds = create_mint_program_address_seeds(mint_account_seed, mint_bump_seed);
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

pub fn create_program_account_mint_account_with_seed<'a>(
    mint_info: &AccountInfo<'a>,
    mint_account_seeds: &[&[u8]],
    mint_authority_info: &AccountInfo<'a>,
    payer_info: &AccountInfo<'a>,
    rent_info: &AccountInfo<'a>,
    token_program_info: &AccountInfo<'a>,
    system_info: &AccountInfo<'a>,
    program_id: &Pubkey,
) -> ProgramResult {
    let rent = Rent::get()?;
    let mint_rent = rent.minimum_balance(Mint::LEN);
    let decimals = 9; // for now
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
pub fn create_program_token_account<'a>(
    account_info: &AccountInfo<'a>,
    account_seeds: &[&[u8]],
    mint_info: &AccountInfo<'a>,
    authority_info: &AccountInfo<'a>,
    payer_info: &AccountInfo<'a>,
    rent_info: &AccountInfo<'a>,
    token_program_info: &AccountInfo<'a>,
    system_info: &AccountInfo<'a>,
    program_id: &Pubkey,
) -> ProgramResult {
    let rent = Rent::get()?;
    let address = Pubkey::create_program_address(&account_seeds, program_id).unwrap();
    if account_info.key != &address {
        msg!(
            "Create account with PDA: {:?} was requested while PDA: {:?} was expected",
            account_info.key,
            address
        );
        return Err(ProgramError::InvalidSeeds);
    }

    invoke_signed(
        &system_instruction::create_account(
            payer_info.key,
            account_info.key,
            rent.minimum_balance(spl_token::state::Account::LEN),
            spl_token::state::Account::LEN as u64,
            &spl_token::id(),
        ),
        &[
            payer_info.clone(),
            account_info.clone(),
            system_info.clone(),
            token_program_info.clone(),
        ],
        &[&account_seeds], // missing things here, we need the full seed for the mint accoutn
    )?;

    invoke(
        &initialize_account(
            token_program_info.key,
            account_info.key,
            mint_info.key,
            authority_info.key, //freeze_authority_pubkey.as_ref(),
        )?,
        &[
            account_info.clone(),
            mint_info.clone(),
            authority_info.clone(),
            rent_info.clone(),
        ],
    )?;
    Ok(())
}

/*
 */
/* NOT USED ANYMORE SINCE WE USE STAKE POOL
pub fn create_program_associated_token_account<'a>(
    account: &AccountInfo<'a>,
    account_bump_seed: u8,
    mint_info: &AccountInfo<'a>,
    payer_info: &AccountInfo<'a>,
    owner_info: &AccountInfo<'a>,
    rent_info: &AccountInfo<'a>,
    token_program_info: &AccountInfo<'a>,
    system_info: &AccountInfo<'a>,
    program_id: &Pubkey,
) -> ProgramResult {
    if !account.data_is_empty() {
        msg!("Account already exist, this will be a restake to same account");
        return Ok(());
    }

    let rent = Rent::get()?;
    let seeds: [&[u8]; 4] = [
        // Associated
        &payer_info.key.to_bytes(),
        &token_program_info.key.to_bytes(),
        &mint_info.key.to_bytes(),
        &[account_bump_seed],
    ];

    let address = Pubkey::create_program_address(&seeds, program_id).unwrap();
    if account.key != &address {
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
            account.key,
            rent.minimum_balance(spl_token::state::Account::LEN),
            spl_token::state::Account::LEN as u64,
            &spl_token::id(),
        ),
        &[
            payer_info.clone(),
            account.clone(),
            system_info.clone(),
            token_program_info.clone(),
        ],
        &[&seeds], // missing things here, we need the full seed for the mint accoutn
    )?;

    invoke(
        &initialize_account(
            token_program_info.key,
            account.key,
            mint_info.key,
            owner_info.key, //freeze_authority_pubkey.as_ref(),
        )?,
        &[
            account.clone(),
            mint_info.clone(),
            owner_info.clone(),
            rent_info.clone(),
        ],
    )?;
    Ok(())
} */
/*
pub fn create_user_post_token_account<'a>(
    user_account: &Pubkey,
    post_account: &Pubkey,
    user_post_token_account: &AccountInfo<'a>,
    user_post_token_account_bump_seed: u8,
    mint_info: &AccountInfo<'a>,
    mint_authority_info: &AccountInfo<'a>,
    payer_info: &AccountInfo<'a>,
    rent_info: &AccountInfo<'a>,
    token_program_info: &AccountInfo<'a>,
    system_info: &AccountInfo<'a>,
    program_id: &Pubkey,
) -> ProgramResult {
    if !user_post_token_account.data_is_empty() {
        msg!("Account already exist, this will be a restake to same account");
        return Ok(());
    }
    let rent = Rent::get()?;
    let bump_seed = &[user_post_token_account_bump_seed];
    let user_post_token_account_seeds =
        create_user_post_token_program_address_seeds(post_account, user_account, bump_seed);

    let (address, _) = find_user_post_token_program_address(program_id, post_account, user_account);
    //Pubkey::create_program_address(&user_post_token_account_seeds, program_id).unwrap();
    if user_post_token_account.key != &address {
        msg!(
            "Create account with PDA: {:?} was requested while PDA: {:?} was expected",
            mint_info.key,
            address
        );
        return Err(ProgramError::InvalidSeeds);
    }
    let token_address = get_associated_token_address(payer_info.key, mint_info.key);

    invoke_signed(
        &system_instruction::create_account(
            payer_info.key,
            &token_address,
            rent.minimum_balance(spl_token::state::Account::LEN),
            spl_token::state::Account::LEN as u64,
            &spl_token::id(),
        ),
        &[
            payer_info.clone(),
            user_post_token_account.clone(),
            system_info.clone(),
            token_program_info.clone(),
        ],
        &[&user_post_token_account_seeds], // missing things here, we need the full seed for the mint accoutn
    )?;

    invoke(
        &initialize_account(
            token_program_info.key,
            user_post_token_account.key,
            mint_info.key,
            mint_authority_info.key, //freeze_authority_pubkey.as_ref(),
        )?,
        &[
            user_post_token_account.clone(),
            mint_info.clone(),
            mint_authority_info.clone(),
            rent_info.clone(),
        ],
    )?;
    Ok(())
}*/

pub fn spl_mint_to<'a>(
    mint_to_account: &AccountInfo<'a>,
    mint_info: &AccountInfo<'a>,
    mint_authority_info: &AccountInfo<'a>,
    mint_authority_seeds: &[&[u8]],
    amount: u64,
    program_id: &Pubkey,
) -> ProgramResult {
    // mint
    /* let mint_authority_bump_seed = &[mint_authority_bump_seed];
    let mint_authority_seeds =
        create_mint_authority_program_address_seeds(mint_info.key, mint_authority_bump_seed); */
    let mint_authority_address =
        Pubkey::create_program_address(mint_authority_seeds, program_id).unwrap();
    if mint_authority_info.key != &mint_authority_address {
        msg!(
            "Create account with PDA: {:?} was requested while PDA: {:?} was expected",
            mint_authority_info.key,
            mint_authority_address
        );
        return Err(ProgramError::InvalidSeeds);
    }

    let ix = mint_to(
        &spl_token::id(),
        mint_info.key,
        mint_to_account.key,
        &mint_authority_address,
        &[],
        amount,
    )?;
    invoke_signed(
        &ix,
        &[
            mint_info.clone(),
            mint_to_account.clone(),
            mint_authority_info.clone(),
        ],
        &[&mint_authority_seeds],
    )?;

    Ok(())
}

pub fn transfer_to<'a>(
    payer_info: &AccountInfo<'a>,
    to_account_info: &AccountInfo<'a>,
    amount: u64,
) -> ProgramResult {
    // take from payer
    invoke(
        &system_instruction::transfer(
            payer_info.key, // mby this should be program id
            to_account_info.key,
            amount,
        ),
        &[payer_info.clone(), to_account_info.clone()],
    )?;
    Ok(())
}
/// Issue a spl_token `Transfer` instruction.
#[allow(clippy::too_many_arguments)]
pub fn token_transfer<'a>(
    token_program: AccountInfo<'a>,
    source: AccountInfo<'a>,
    destination: AccountInfo<'a>,
    authority: AccountInfo<'a>,
    amount: u64,
) -> Result<(), ProgramError> {
    let ix = spl_token::instruction::transfer(
        token_program.key,
        source.key,
        destination.key,
        authority.key,
        &[],
        amount,
    )?;
    invoke(&ix, &[source, destination, authority, token_program])
}

/*
pub fn create_payer_program_multisig_account<'a>(
    multisig_info: &AccountInfo<'a>,
    multisig_bump_seed: u8,
    payer_info: &AccountInfo<'a>,
    owner_info: &AccountInfo<'a>,
    rent_info: &AccountInfo<'a>,
    token_program_info: &AccountInfo<'a>,
    program_info: &AccountInfo<'a>,
    system_info: &AccountInfo<'a>,
) -> ProgramResult {
    let rent = Rent::get()?;
    let multisig_rent = rent.minimum_balance(Multisig::LEN);
    let seeds = &[
        "token".as_bytes(),
        owner_info.key.as_ref(),
        &[multisig_bump_seed],
    ];
    let expected_multisig_address =
        Pubkey::create_program_address(seeds, program_info.key).unwrap();

    if multisig_info.key != &expected_multisig_address {
        msg!(
            "Create account with PDA: {:?} was requested while PDA: {:?} was expected",
            multisig_info.key,
            expected_multisig_address
        );
        return Err(ProgramError::InvalidSeeds);
    }

    invoke_signed(
        &system_instruction::create_account(
            &payer_info.key,
            &multisig_info.key,
            multisig_rent,
            Multisig::LEN as u64,
            &spl_token::id(),
        ),
        &[
            payer_info.clone(),
            multisig_info.clone(),
            system_info.clone(),
            token_program_info.clone(),
        ],
        &[seeds],
    )?;

    invoke(
        &spl_token::instruction::initialize_multisig(
            &spl_token::id(),
            &multisig_info.key,
            &[payer_info.key, program_info.key],
            1, // assume only either the payer or the program has to sign to mint
        )?,
        &[
            multisig_info.clone(),
            rent_info.clone(),
            payer_info.clone(),
            program_info.clone(),
        ],
    )?;

    Ok(())
}
 */
