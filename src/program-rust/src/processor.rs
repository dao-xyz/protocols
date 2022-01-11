use borsh::{BorshDeserialize, BorshSerialize};

use solana_program::{
    account_info::{next_account_info, AccountInfo},
    borsh::{get_instance_packed_len, get_packed_len, try_from_slice_unchecked},
    clock::Clock,
    entrypoint::ProgramResult,
    instruction::Instruction,
    msg,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
    rent::Rent,
    stake,
    sysvar::Sysvar,
};
use spl_stake_pool::{
    error::StakePoolError,
    find_deposit_authority_program_address, find_withdraw_authority_program_address,
    minimum_reserve_lamports,
    state::{AccountType, Fee, StakePool, ValidatorList},
};
use spl_token::state::Mint;

use crate::{
    account::create_and_serialize_account_signed,
    accounts::{deserialize_user_account, AccountContainer, MessageAccount},
    address::generate_seeds_from_string,
    instruction::ChatInstruction,
    owner::assert_is_signing_program_owner,
    rates::get_allowed_mint_amount,
    stake_pool_dep::{create_independent_reserve_stake_account, create_pool_mint},
    tokens::spl_utils::{
        create_escrow_account_bump_seeds, create_payer_program_multisig_account,
        create_program_account_mint_account, create_program_associated_token_account,
        create_program_key_token_account, create_user_post_token_account, spl_mint_to, transfer_to,
    },
};
use crate::{
    account::create_and_serialize_account_signed_verify_with_bump,
    accounts::{deserialize_post_account, PostAccount, PostContentAccount},
};
use solana_program::system_instruction::create_account;

pub static NULL_KEY: Pubkey = Pubkey::new_from_array([0_u8; 32]);

pub static MESSAGE_TRANSACTION_MAX_SIZE: usize = 1200;
const AUTHORITY_WITHDRAW: &[u8] = b"withdraw";

fn token_mint_to<'a>(
    stake_pool: &Pubkey,
    token_program: AccountInfo<'a>,
    mint: AccountInfo<'a>,
    destination: AccountInfo<'a>,
    authority: AccountInfo<'a>,
    authority_type: &[u8],
    bump_seed: u8,
    amount: u64,
) -> Result<(), ProgramError> {
    let me_bytes = stake_pool.to_bytes();
    let authority_signature_seeds = [&me_bytes[..32], authority_type, &[bump_seed]];
    let signers = &[&authority_signature_seeds[..]];

    let ix = spl_token::instruction::mint_to(
        token_program.key,
        mint.key,
        destination.key,
        authority.key,
        &[],
        amount,
    )?;

    invoke_signed(&ix, &[mint, destination, authority, token_program], signers)
}
fn check_account_owner(
    account_info: &AccountInfo,
    program_id: &Pubkey,
) -> Result<(), ProgramError> {
    if *program_id != *account_info.owner {
        msg!(
            "Expected account to be owned by program {}, received {}",
            program_id,
            account_info.owner
        );
        Err(ProgramError::IncorrectProgramId)
    } else {
        Ok(())
    }
}

fn process_initialize(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    epoch_fee: Fee,
    withdrawal_fee: Fee,
    deposit_fee: Fee,
    referral_fee: u8,
    max_validators: u32,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let stake_pool_info = next_account_info(account_info_iter)?;
    let manager_info = next_account_info(account_info_iter)?;
    let staker_info = next_account_info(account_info_iter)?;
    let withdraw_authority_info = next_account_info(account_info_iter)?;
    let validator_list_info = next_account_info(account_info_iter)?;
    let reserve_stake_info = next_account_info(account_info_iter)?;
    let pool_mint_info = next_account_info(account_info_iter)?;
    let manager_fee_info = next_account_info(account_info_iter)?;
    let token_program_info = next_account_info(account_info_iter)?;

    let rent = Rent::get()?;

    if !manager_info.is_signer {
        msg!("Manager did not sign initialization");
        return Err(StakePoolError::SignatureMissing.into());
    }

    if stake_pool_info.key == validator_list_info.key {
        msg!("Cannot use same account for stake pool and validator list");
        return Err(StakePoolError::AlreadyInUse.into());
    }

    check_account_owner(stake_pool_info, program_id)?;
    let mut stake_pool = try_from_slice_unchecked::<StakePool>(&stake_pool_info.data.borrow())?;
    if !stake_pool.is_uninitialized() {
        msg!("Provided stake pool already in use");
        return Err(StakePoolError::AlreadyInUse.into());
    }

    check_account_owner(validator_list_info, program_id)?;
    let mut validator_list =
        try_from_slice_unchecked::<ValidatorList>(&validator_list_info.data.borrow())?;
    if !validator_list.header.is_uninitialized() {
        msg!("Provided validator list already in use");
        return Err(StakePoolError::AlreadyInUse.into());
    }

    let data_length = validator_list_info.data_len();
    let expected_max_validators = ValidatorList::calculate_max_validators(data_length);
    if expected_max_validators != max_validators as usize || max_validators == 0 {
        msg!(
            "Incorrect validator list size provided, expected {}, provided {}",
            expected_max_validators,
            max_validators
        );
        return Err(StakePoolError::UnexpectedValidatorListAccountSize.into());
    }
    validator_list.header.account_type = AccountType::ValidatorList;
    validator_list.header.max_validators = max_validators;
    validator_list.validators.clear();

    if !rent.is_exempt(stake_pool_info.lamports(), stake_pool_info.data_len()) {
        msg!("Stake pool not rent-exempt");
        return Err(ProgramError::AccountNotRentExempt);
    }

    if !rent.is_exempt(
        validator_list_info.lamports(),
        validator_list_info.data_len(),
    ) {
        msg!("Validator stake list not rent-exempt");
        return Err(ProgramError::AccountNotRentExempt);
    }

    // Numerator should be smaller than or equal to denominator (fee <= 1)
    if epoch_fee.numerator > epoch_fee.denominator
        || withdrawal_fee.numerator > withdrawal_fee.denominator
        || deposit_fee.numerator > deposit_fee.denominator
        || referral_fee > 100u8
    {
        return Err(StakePoolError::FeeTooHigh.into());
    }

    if *token_program_info.key != spl_token::id() {
        msg!(
            "Only the SPL token program is currently supported, expected {}, received {}",
            spl_token::id(),
            *token_program_info.key
        );
        return Err(ProgramError::IncorrectProgramId);
    }

    if manager_fee_info.owner != token_program_info.key {
        return Err(ProgramError::IncorrectProgramId);
    }

    if pool_mint_info.owner != token_program_info.key {
        return Err(ProgramError::IncorrectProgramId);
    }

    if *pool_mint_info.key
        != spl_token::state::Account::unpack_from_slice(&manager_fee_info.data.borrow())?.mint
    {
        msg!("Manager fee account wrong mint account");
        return Err(StakePoolError::WrongAccountMint.into());
    }

    let (stake_deposit_authority, sol_deposit_authority) =
        match next_account_info(account_info_iter) {
            Ok(deposit_authority_info) => (
                *deposit_authority_info.key,
                Some(*deposit_authority_info.key),
            ),
            Err(_) => (
                find_deposit_authority_program_address(program_id, stake_pool_info.key).0,
                None,
            ),
        };
    let (withdraw_authority_key, stake_withdraw_bump_seed) =
        find_withdraw_authority_program_address(program_id, stake_pool_info.key);
    if withdraw_authority_key != *withdraw_authority_info.key {
        msg!(
            "Incorrect withdraw authority provided, expected {}, received {}",
            withdraw_authority_key,
            withdraw_authority_info.key
        );
        return Err(StakePoolError::InvalidProgramAddress.into());
    }

    let pool_mint = Mint::unpack_from_slice(&pool_mint_info.data.borrow())?;

    if pool_mint.supply != 0 {
        return Err(StakePoolError::NonZeroPoolTokenSupply.into());
    }

    if !pool_mint.mint_authority.contains(&withdraw_authority_key) {
        return Err(StakePoolError::WrongMintingAuthority.into());
    }

    if pool_mint.freeze_authority.is_some() {
        return Err(StakePoolError::InvalidMintFreezeAuthority.into());
    }

    if *reserve_stake_info.owner != stake::program::id() {
        msg!("Reserve stake account not owned by stake program");
        return Err(ProgramError::IncorrectProgramId);
    }

    let stake_state =
        try_from_slice_unchecked::<stake::state::StakeState>(&reserve_stake_info.data.borrow())?;

    let total_lamports = if let stake::state::StakeState::Initialized(meta) = stake_state {
        if meta.lockup != stake::state::Lockup::default() {
            msg!("Reserve stake account has some lockup");
            return Err(StakePoolError::WrongStakeState.into());
        }

        //msg!(meta.authorized.withdrawer.to_string().as_str());
        msg!(withdraw_authority_key.to_string().as_str());
        if meta.authorized.staker != withdraw_authority_key {
            msg!(
                "Reserve stake account has incorrect staker {}, should be {}",
                meta.authorized.staker.to_string().as_str(),
                withdraw_authority_key.to_string().as_str()
            );
            return Err(StakePoolError::WrongStakeState.into());
        }

        /* if meta.authorized.staker != withdraw_authority_key {
            msg!(
                "Reserve stake account has incorrect staker {}, should be {}",
                meta.authorized.staker,
                withdraw_authority_key
            );
            return Err(StakePoolError::WrongStakeState.into());
        } */
        if true {
            return Ok(());
        }
        if meta.authorized.withdrawer != withdraw_authority_key {
            msg!(
                "Reserve stake account has incorrect withdrawer {}, should be {}",
                meta.authorized.staker,
                withdraw_authority_key
            );
            return Err(StakePoolError::WrongStakeState.into());
        }

        reserve_stake_info
            .lamports()
            .checked_sub(minimum_reserve_lamports(&meta))
            .ok_or(StakePoolError::CalculationFailure)?
    } else {
        msg!("Reserve stake account not in intialized state");
        return Err(StakePoolError::WrongStakeState.into());
    };

    if total_lamports > 0 {
        token_mint_to(
            stake_pool_info.key,
            token_program_info.clone(),
            pool_mint_info.clone(),
            manager_fee_info.clone(),
            withdraw_authority_info.clone(),
            AUTHORITY_WITHDRAW,
            stake_withdraw_bump_seed,
            total_lamports,
        )?;
    }

    validator_list.serialize(&mut *validator_list_info.data.borrow_mut())?;

    stake_pool.account_type = AccountType::StakePool;
    stake_pool.manager = *manager_info.key;
    stake_pool.staker = *staker_info.key;
    stake_pool.stake_deposit_authority = stake_deposit_authority;
    stake_pool.stake_withdraw_bump_seed = stake_withdraw_bump_seed;
    stake_pool.validator_list = *validator_list_info.key;
    stake_pool.reserve_stake = *reserve_stake_info.key;
    stake_pool.pool_mint = *pool_mint_info.key;
    stake_pool.manager_fee_account = *manager_fee_info.key;
    stake_pool.token_program_id = *token_program_info.key;
    stake_pool.total_lamports = total_lamports;
    stake_pool.pool_token_supply = 0;
    stake_pool.last_update_epoch = Clock::get()?.epoch;
    stake_pool.lockup = stake::state::Lockup::default();
    stake_pool.epoch_fee = epoch_fee;
    stake_pool.next_epoch_fee = None;
    stake_pool.preferred_deposit_validator_vote_address = None;
    stake_pool.preferred_withdraw_validator_vote_address = None;
    stake_pool.stake_deposit_fee = deposit_fee;
    stake_pool.stake_withdrawal_fee = withdrawal_fee;
    stake_pool.next_stake_withdrawal_fee = None;
    stake_pool.stake_referral_fee = referral_fee;
    stake_pool.sol_deposit_authority = sol_deposit_authority;
    stake_pool.sol_deposit_fee = deposit_fee;
    stake_pool.sol_referral_fee = referral_fee;
    stake_pool.sol_withdraw_authority = None;
    stake_pool.sol_withdrawal_fee = withdrawal_fee;
    stake_pool.next_sol_withdrawal_fee = None;
    stake_pool.last_epoch_pool_token_supply = 0;
    stake_pool.last_epoch_total_lamports = 0;

    stake_pool
        .serialize(&mut *stake_pool_info.data.borrow_mut())
        .map_err(|e| e.into())
}

// Program entrypoint's implementation
pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], input: &[u8]) -> ProgramResult {
    let instruction = ChatInstruction::try_from_slice(input)?;

    // Iterating accounts is safer then indexing
    let accounts_iter = &mut accounts.iter();

    let system_account = next_account_info(accounts_iter)?;
    let program_account = next_account_info(accounts_iter)?;
    let payer_account = next_account_info(accounts_iter)?;

    match instruction {
        ChatInstruction::InitializeToken(initialize) => {
            // initialize multisig owner mint with escrow
            let owner_account = next_account_info(accounts_iter)?;
            let escrow_account_info = next_account_info(accounts_iter)?;
            let mint_account_info = next_account_info(accounts_iter)?;
            let multisig_account_info = next_account_info(accounts_iter)?;
            let owner_token_account = next_account_info(accounts_iter)?;
            let rent_info = next_account_info(accounts_iter)?;
            let token_program_info = next_account_info(accounts_iter)?;
            let rent = Rent::get()?;

            assert_is_signing_program_owner(owner_account, owner_token_account)?;

            create_payer_program_multisig_account(
                multisig_account_info,
                initialize.multisig_bump_seed,
                payer_account,
                owner_account,
                rent_info,
                token_program_info,
                program_account,
                system_account,
            )?;

            create_program_account_mint_account(
                program_id,
                mint_account_info,
                initialize.mint_bump_seed,
                multisig_account_info,
                payer_account,
                rent_info,
                token_program_info,
                system_account,
                program_id,
            )?;

            // create empty escrow account
            let escrow_bump_seeds = &[initialize.escrow_bump_seed];
            let escrow_account_seeds =
                create_escrow_account_bump_seeds(&program_id, escrow_bump_seeds);
            let expected_escrow_address =
                Pubkey::create_program_address(&escrow_account_seeds, program_id).unwrap();

            if escrow_account_info.key != &expected_escrow_address {
                msg!(
                    "Create account with PDA: {:?} was requested while PDA: {:?} was expected",
                    escrow_account_info.key,
                    expected_escrow_address
                );
                return Err(ProgramError::InvalidSeeds);
            }
            let minimum_balance_as_stake = rent.minimum_balance(0);
            let create_account_instruction = create_account(
                payer_account.key,
                escrow_account_info.key,
                minimum_balance_as_stake,
                0 as u64,
                program_id,
            );
            invoke_signed(
                &create_account_instruction,
                &[
                    payer_account.clone(),
                    escrow_account_info.clone(),
                    system_account.clone(),
                ],
                &[&escrow_account_seeds],
            )?;
        }
        ChatInstruction::SetupStakePool(initialize) => {
            let rent = Rent::get()?;
            let program_owner_info = next_account_info(accounts_iter)?;
            let program_owner_token_info = next_account_info(accounts_iter)?;
            let stake_pool_info = next_account_info(accounts_iter)?;
            let manager_info = next_account_info(accounts_iter)?;
            let staker_info = next_account_info(accounts_iter)?;
            let withdraw_authority_info = next_account_info(accounts_iter)?;
            let validator_list_info = next_account_info(accounts_iter)?;
            let reserve_stake_info = next_account_info(accounts_iter)?;
            let pool_mint_info = next_account_info(accounts_iter)?;
            let manager_fee_info = next_account_info(accounts_iter)?;
            let rent_info = next_account_info(accounts_iter)?;
            let token_program_info = next_account_info(accounts_iter)?;
            let stake_program_info = next_account_info(accounts_iter)?;

            assert_is_signing_program_owner(program_owner_info, program_owner_token_info)?;

            let max_validators = 10;
            create_pool_mint(
                stake_pool_info.key,
                pool_mint_info,
                initialize.pool_mint_bump_seed,
                withdraw_authority_info.key,
                payer_account,
                rent_info,
                token_program_info,
                system_account,
                program_id,
            )?;

            // Reserve stake account
            create_independent_reserve_stake_account(
                reserve_stake_info,
                initialize.reserve_stake_bump_seed,
                stake_pool_info.key,
                payer_account,
                withdraw_authority_info.key,
                stake_program_info,
                rent_info,
                program_id,
            )?;

            // Manager fee account
            create_program_associated_token_account(
                manager_fee_info,
                initialize.manager_fee_account_bump_seed,
                pool_mint_info,
                payer_account,
                manager_info,
                rent_info,
                token_program_info,
                system_account,
                program_id,
            )?;
        }

        ChatInstruction::InitializeStakePool(initialize) => {
            let rent = Rent::get()?;
            let program_owner_info = next_account_info(accounts_iter)?;
            let program_owner_token_info = next_account_info(accounts_iter)?;
            let stake_pool_info = next_account_info(accounts_iter)?;
            let manager_info = next_account_info(accounts_iter)?;
            let staker_info = next_account_info(accounts_iter)?;
            let withdraw_authority_info = next_account_info(accounts_iter)?;
            let validator_list_info = next_account_info(accounts_iter)?;
            let reserve_stake_info = next_account_info(accounts_iter)?;
            let pool_mint_info = next_account_info(accounts_iter)?;
            let manager_fee_info = next_account_info(accounts_iter)?;
            let rent_info = next_account_info(accounts_iter)?;
            let token_program_info = next_account_info(accounts_iter)?;
            /* let ss = next_account_info(accounts_iter)?;
             */
            let ss2 = next_account_info(accounts_iter)?;

            let max_validators = 10;
            /* assert_is_signing_program_owner(program_owner_info, program_owner_token_info)?;


            create_pool_mint(
                stake_pool_info.key,
                pool_mint_info,
                initialize.pool_mint_bump_seed,
                withdraw_authority_info.key,
                payer_account,
                rent_info,
                token_program_info,
                system_account,
                program_id,
            )?;

            // Reserve stake account
            create_independent_reserve_stake_account(
                reserve_stake_info,
                initialize.reserve_stake_bump_seed,
                stake_pool_info.key,
                payer_account,
                withdraw_authority_info.key,
                stake_program_info,
                rent_info,
                program_id,
            )?;

            // Manager fee account
            create_program_associated_token_account(
                manager_fee_info,
                initialize.manager_fee_account_bump_seed,
                pool_mint_info,
                payer_account,
                manager_info,
                rent_info,
                token_program_info,
                system_account,
                program_id,
            )?;
            */
            /* let manager_account_bump_seeds = [
                "manager".as_bytes(),
                &stake_pool_info.key.to_bytes(),
                &[initialize.manager_bump_seed],
            ];
            let manager_account_address =
                Pubkey::create_program_address(&manager_account_bump_seeds, program_id).unwrap();

            if manager_info.key != &manager_account_address {
                msg!(
                    "Create account with PDA: {:?} was requested while PDA: {:?} was expected",
                    manager_info.key,
                    manager_account_address
                );
                return Err(ProgramError::InvalidSeeds);
            } */

            // -------- create stake pool ----------
            // Validator list account
            let empty_validator_list = ValidatorList::new(max_validators);
            let validator_list_size = get_instance_packed_len(&empty_validator_list)?;
            invoke_signed(
                &create_account(
                    payer_account.key,
                    validator_list_info.key,
                    rent.minimum_balance(validator_list_size),
                    validator_list_size as u64,
                    &spl_stake_pool::id(),
                ),
                &[payer_account.clone(), validator_list_info.clone()],
                &[&[
                    "validator_list".as_bytes(),
                    &stake_pool_info.key.to_bytes(),
                    &[initialize.validator_list_bump_seed],
                ]],
            )?;

            // Stake ppol account
            let xs = ["stake_pool".as_bytes(), &[initialize.stake_pool_bump_seed]];
            let x = Pubkey::create_program_address(&xs, &program_id).unwrap();

            if stake_pool_info.key != &x {
                msg!(
                    "Create account with PDA: {:?} was requested while PDA: {:?} was expected",
                    stake_pool_info.key,
                    x
                );
                return Err(ProgramError::InvalidSeeds);
            }
            invoke_signed(
                &create_account(
                    payer_account.key,
                    stake_pool_info.key,
                    rent.minimum_balance(initialize.stake_pool_packed_len as usize), // assume 8 bytes
                    initialize.stake_pool_packed_len,
                    /* rent.minimum_balance(get_packed_len::<spl_stake_pool::state::StakePool>()),
                    get_packed_len::<spl_stake_pool::state::StakePool>() as u64, */
                    &spl_stake_pool::id(),
                ),
                &[payer_account.clone(), stake_pool_info.clone()],
                &[&["stake_pool".as_bytes(), &[initialize.stake_pool_bump_seed]]],
            )?;

            msg!("INIT");
            msg!("???");
            let instruction = spl_stake_pool::instruction::initialize(
                &spl_stake_pool::id(),
                stake_pool_info.key,
                manager_info.key,
                staker_info.key,
                validator_list_info.key,
                reserve_stake_info.key,
                pool_mint_info.key,
                manager_fee_info.key,
                token_program_info.key,
                None,
                spl_stake_pool::state::Fee {
                    numerator: 5, // 5%
                    denominator: 100,
                },
                spl_stake_pool::state::Fee {
                    numerator: 0,
                    denominator: 1,
                },
                spl_stake_pool::state::Fee {
                    numerator: 0,
                    denominator: 1,
                },
                0,
                10,
            );

            invoke(
                &instruction,
                &[
                    stake_pool_info.clone(),
                    manager_info.clone(),
                    staker_info.clone(),
                    withdraw_authority_info.clone(),
                    validator_list_info.clone(),
                    reserve_stake_info.clone(),
                    pool_mint_info.clone(),
                    manager_fee_info.clone(),
                    token_program_info.clone(),
                    ss2.clone(),
                    rent_info.clone(),
                    program_account.clone(),
                    system_account.clone(),
                    payer_account.clone(),
                ],
            )?;

            /* rent_info.clone(),
            program_account.clone(),
            system_account.clone(), */
            /* process_initialize(
                &spl_stake_pool::id(),
                &[
                    stake_pool_info.clone(),
                    manager_info.clone(),
                    staker_info.clone(),
                    withdraw_authority_info.clone(),
                    validator_list_info.clone(),
                    reserve_stake_info.clone(),
                    pool_mint_info.clone(),
                    manager_fee_info.clone(),
                    token_program_info.clone(),
                ],
                spl_stake_pool::state::Fee {
                    numerator: 5, // 5%
                    denominator: 100,
                },
                spl_stake_pool::state::Fee {
                    numerator: 0,
                    denominator: 1,
                },
                spl_stake_pool::state::Fee {
                    numerator: 0,
                    denominator: 1,
                },
                0,
                10,
            )?; */
        }

        /*
         instruction::set_fee(
            &id(),
            &stake_pool.pubkey(),
            &manager.pubkey(),
            FeeType::SolDeposit(*sol_deposit_fee),
        ),
        instruction::set_fee(
            &id(),
            &stake_pool.pubkey(),
            &manager.pubkey(),
            FeeType::SolReferral(sol_referral_fee),
        ),
        */

        /*  invoke_signed(
            &spl_stake_pool::instruction::set_fee(
                &spl_stake_pool::id(),
                &stake_pool_info.key,
                &manager_info.key,
                spl_stake_pool::state::FeeType::SolDeposit(*sol_deposit_fee),
            ),
            &[
                stake_pool_info.clone(),
                manager_info.clone(),
                staker_info.clone(),
                validator_list_info.clone(),
                reserve_stake_info.clone(),
                pool_mint_info.clone(),
                manager_fee_info.clone(),
                token_program_info.clone(),
            ],
            &[&manager_account_bump_seeds],
        )?; */
        ChatInstruction::StakePoolInstruction(instruction) => {
            return crate::stake_pool::processor::Processor::process(program_id, accounts, input);
        }
        ChatInstruction::CreateUser(user) => {
            if user.name.is_empty() {
                return Err(ProgramError::InvalidArgument);
            }
            // check if leading or trailing spaces, if so name is invalid
            let mut chars = user.name.chars();
            if chars.next().unwrap().is_whitespace() || chars.last().unwrap_or('_').is_whitespace()
            {
                return Err(ProgramError::InvalidArgument);
            }

            if &user.owner != payer_account.key {
                return Err(ProgramError::IllegalOwner); // requires payer as owner (for now)
            }

            let user_acount_info = next_account_info(accounts_iter)?;
            let rent = Rent::get()?;
            let seeds = generate_seeds_from_string(&user.name)?;
            let seed_slice = &seeds.iter().map(|x| &x[..]).collect::<Vec<&[u8]>>()[..];
            create_and_serialize_account_signed(
                payer_account,
                user_acount_info,
                &AccountContainer::UserAccount(user),
                seed_slice,
                program_id,
                system_account,
                &rent,
            )?;
        }

        ChatInstruction::CreateChannel(channel) => {
            let user_account_info = next_account_info(accounts_iter)?;
            let user = deserialize_user_account(user_account_info.data.borrow().as_ref());
            if &user.owner != payer_account.key {
                return Err(ProgramError::IllegalOwner); // requires payer as owner (for now)
            }

            let channel_account_info = next_account_info(accounts_iter)?;

            let rent = Rent::get()?;
            let seeds = generate_seeds_from_string(&channel.name)?;
            let seed_slice = &seeds.iter().map(|x| &x[..]).collect::<Vec<&[u8]>>()[..];
            create_and_serialize_account_signed(
                payer_account,
                channel_account_info,
                &AccountContainer::ChannelAccount(channel),
                seed_slice,
                program_id,
                system_account,
                &rent,
            )?;
        }
        ChatInstruction::UpdateChannel(_) => {
            /*  let channel_account_info = next_account_info(accounts_iter)?;

            // Don't allow channel name to be updated, since it would require us to resize the account size
            // This would also mean that the PDA would change!
            channel.serialize(&mut *channel_account_info.data.borrow_mut())? */
        }

        ChatInstruction::SendMessage(send_message) => {
            // Initializes an account for us that lets us build an message
            let user_account_info = next_account_info(accounts_iter)?;
            let channel_account_info = next_account_info(accounts_iter)?;
            let message_account = MessageAccount::new(
                send_message.user,
                send_message.channel,
                send_message.timestamp,
                send_message.message,
            );
            let message_account_info = next_account_info(accounts_iter)?;
            let rent = Rent::get()?;
            let user = deserialize_user_account(user_account_info.data.borrow().as_ref());
            if &user.owner != payer_account.key {
                return Err(ProgramError::IllegalOwner); // requires payer as owner (for now)
            }
            create_and_serialize_account_signed_verify_with_bump(
                payer_account,
                message_account_info,
                &AccountContainer::MessageAccount(message_account),
                &[
                    &user_account_info.key.to_bytes(),
                    &channel_account_info.key.to_bytes(),
                    &send_message.timestamp.to_le_bytes(),
                ],
                program_id,
                system_account,
                &rent,
                send_message.bump_seed,
            )?;
        }

        ChatInstruction::CreatePost(post) => {
            //let token_account_info = next_account_info(accounts_iter)?;
            let user_account_info = next_account_info(accounts_iter)?;
            let user = deserialize_user_account(user_account_info.data.borrow().as_ref());
            if &user.owner != payer_account.key {
                // Can not create a post for another user
                return Err(ProgramError::InvalidArgument);
            }

            let post_account_info = next_account_info(accounts_iter)?;
            let escrow_account_info = next_account_info(accounts_iter)?;
            let mint_account_info = next_account_info(accounts_iter)?;
            let mint_authority_account_info = next_account_info(accounts_iter)?;
            let user_post_token_account_info = next_account_info(accounts_iter)?;

            let rent_info = next_account_info(accounts_iter)?;
            let token_program_info = next_account_info(accounts_iter)?;
            let rent = Rent::get()?;

            create_and_serialize_account_signed_verify_with_bump(
                payer_account,
                post_account_info,
                &AccountContainer::PostAccount(PostAccount {
                    channel: post.channel,
                    content: post.content,
                    spread_factor: post.spread_factor,
                    timestamp: post.timestamp,
                    token: *mint_account_info.key,
                    user: *user_account_info.key,
                }),
                &[
                    user_account_info.key.as_ref(),
                    post.channel.as_ref(),
                    &post.timestamp.to_le_bytes(),
                ],
                program_id,
                system_account,
                &rent,
                post.post_bump_seed,
            )?;

            create_program_account_mint_account(
                post_account_info.key,
                mint_account_info,
                post.mint_bump_seed,
                mint_authority_account_info,
                payer_account,
                rent_info,
                token_program_info,
                system_account,
                program_id,
            )?;

            // create empty escrow account
            let escrow_bump_seeds = &[post.escrow_account_bump_seed];
            let escrow_account_seeds =
                create_escrow_account_bump_seeds(&post_account_info.key, escrow_bump_seeds);
            let expected_escrow_address =
                Pubkey::create_program_address(&escrow_account_seeds, program_id).unwrap();

            if escrow_account_info.key != &expected_escrow_address {
                msg!(
                    "Create account with PDA: {:?} was requested while PDA: {:?} was expected",
                    escrow_account_info.key,
                    expected_escrow_address
                );
                return Err(ProgramError::InvalidSeeds);
            }
            let minimum_balance_as_stake = rent.minimum_balance(0);

            let create_account_instruction = create_account(
                payer_account.key,
                escrow_account_info.key,
                minimum_balance_as_stake,
                0 as u64,
                program_id,
            );
            invoke_signed(
                &create_account_instruction,
                &[
                    payer_account.clone(),
                    escrow_account_info.clone(),
                    system_account.clone(),
                ],
                &[&escrow_account_seeds],
            )?;

            // create user stake account
            create_user_post_token_account(
                &user_account_info.key,
                &post_account_info.key,
                user_post_token_account_info,
                post.user_post_token_account_bump_seed,
                mint_account_info,
                mint_authority_account_info,
                payer_account,
                rent_info,
                token_program_info,
                system_account,
                program_id,
            )?;

            // Mint for the minimum balance, as we hade to put some balance into the escrow account
            spl_mint_to(
                user_post_token_account_info,
                mint_account_info,
                mint_authority_account_info,
                post.mint_authority_bump_seed,
                get_allowed_mint_amount(
                    escrow_account_info,
                    minimum_balance_as_stake,
                    post.spread_factor,
                ),
                program_id,
            )?;
        }
        ChatInstruction::CreatePostContent(content) => {
            let post_content_account_info = next_account_info(accounts_iter)?;
            let hash = content.message.hash();
            let rent = Rent::get()?;
            // If creation can be signed hash and bump seed, we know hash is correct
            create_and_serialize_account_signed_verify_with_bump(
                payer_account,
                post_content_account_info,
                &AccountContainer::PostContentAccount(PostContentAccount {
                    message: content.message,
                }),
                &[&hash],
                program_id,
                system_account,
                &rent,
                content.bump_seed,
            )?;
        }

        ChatInstruction::StakePost(stake) => {
            //let token_account_info = next_account_info(accounts_iter)?;
            msg!("New stake");
            msg!(stake.stake.to_string().as_str());
            let post_account_info = next_account_info(accounts_iter)?;
            let post_account = deserialize_post_account(post_account_info.data.borrow().as_ref());
            let escrow_account_info = next_account_info(accounts_iter)?;
            let mint_account_info = next_account_info(accounts_iter)?;
            let mint_authority_account_info = next_account_info(accounts_iter)?;
            let user_post_token_account_info = next_account_info(accounts_iter)?;
            let rent_info = next_account_info(accounts_iter)?;
            let token_program_info = next_account_info(accounts_iter)?;

            // Verify escrow account is correct
            let escrow_bump_seeds = &[stake.escrow_account_bump_seed];
            let escrow_account_seeds =
                create_escrow_account_bump_seeds(&stake.post, escrow_bump_seeds);
            let expected_escrow_address =
                Pubkey::create_program_address(&escrow_account_seeds, program_id).unwrap();

            if escrow_account_info.key != &expected_escrow_address {
                msg!(
                    "Create account with PDA: {:?} was requested while PDA: {:?} was expected",
                    escrow_account_info.key,
                    expected_escrow_address
                );
                return Err(ProgramError::InvalidSeeds);
            }

            create_user_post_token_account(
                &stake.user,
                &stake.post,
                user_post_token_account_info,
                stake.user_post_token_account_bump_seed,
                mint_account_info,
                mint_authority_account_info,
                payer_account,
                rent_info,
                token_program_info,
                system_account,
                program_id,
            )?;

            // deduct SOL
            transfer_to(payer_account, escrow_account_info, stake.stake)?;

            // for some LIKES
            spl_mint_to(
                user_post_token_account_info,
                mint_account_info,
                mint_authority_account_info,
                stake.mint_authority_bump_seed,
                get_allowed_mint_amount(
                    escrow_account_info,
                    stake.stake,
                    post_account.spread_factor,
                ),
                program_id,
            )?;
        }
    }

    Ok(())
}
