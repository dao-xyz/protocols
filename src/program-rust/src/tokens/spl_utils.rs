use solana_program::{
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
    rent::Rent,
    system_instruction,
    sysvar::Sysvar,
};

use spl_token::{
    instruction::{initialize_account, initialize_mint, mint_to},
    state::{Mint, Multisig},
};

static MINT_ACCOUNT_SEED_PREFIX: &str = "mint";
static ESCROW_ACCOUNT_SEED_PREFIX: &str = "escrow";

pub fn create_account_mint_account_seeds<'a>(
    post_account: &'a Pubkey,
    mint_bump_seed: &'a [u8],
) -> [&'a [u8]; 3] {
    [
        MINT_ACCOUNT_SEED_PREFIX.as_bytes(),
        post_account.as_ref(),
        mint_bump_seed,
    ]
}

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
        ESCROW_ACCOUNT_SEED_PREFIX.as_bytes(),
        post_account.as_ref(),
        escrow_account_bump_seed,
    ]
}

pub fn create_program_account_mint_account<'a>(
    target_account: &Pubkey,
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
    let decimals = 9; // for now

    let mint_bump_seeds = &[mint_bump_seed];
    let mint_account_seeds = create_account_mint_account_seeds(target_account, mint_bump_seeds);
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
        &[&mint_account_seeds], // missing things here, we need the full seed for the mint accoutn
    )?;

    invoke(
        &initialize_mint(
            &spl_token::id(),
            mint_info.key,
            mint_authority_info.key,
            Some(mint_authority_info.key), //freeze_authority_pubkey.as_ref(),
            decimals,
        )?,
        &[mint_info.clone(), rent_info.clone()],
    )?;
    Ok(())
}

pub fn create_program_key_token_account<'a>(
    key: &str,
    account_info: &AccountInfo<'a>,
    account_bump_seed: u8,
    mint_info: &AccountInfo<'a>,
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

    let bump_seeds = &[account_bump_seed];
    let account_seeds = [key.as_bytes(), bump_seeds];
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
            mint_authority_info.key, //freeze_authority_pubkey.as_ref(),
        )?,
        &[
            account_info.clone(),
            mint_info.clone(),
            mint_authority_info.clone(),
            rent_info.clone(),
        ],
    )?;
    Ok(())
}

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
}

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
    let user_post_token_account_bump_seeds = &[user_post_token_account_bump_seed];
    let user_post_token_account_seeds = create_user_post_token_account_seeds(
        user_account,
        post_account,
        user_post_token_account_bump_seeds,
    );

    let address =
        Pubkey::create_program_address(&user_post_token_account_seeds, program_id).unwrap();
    if user_post_token_account.key != &address {
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
            user_post_token_account.key,
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
}

pub fn spl_mint_to<'a>(
    mint_to_account: &AccountInfo<'a>,
    mint_info: &AccountInfo<'a>,
    mint_authority_info: &AccountInfo<'a>,
    mint_authority_bump_seed: u8,
    amount: u64,
    program_id: &Pubkey,
) -> ProgramResult {
    // mint
    let seeds = &[mint_authority_bump_seed];
    let mint_authority_seeds = create_mint_authority_account_seeds(mint_info.key, seeds);
    let mint_authority_address =
        Pubkey::create_program_address(&mint_authority_seeds, program_id).unwrap();
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
        mint_authority_info.key,
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
