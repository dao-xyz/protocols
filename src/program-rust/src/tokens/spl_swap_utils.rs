use crate::math;

use {
    crate::account,
    solana_program::{
        account_info::AccountInfo, entrypoint::ProgramResult, msg, program::invoke,
        program_error::ProgramError,
    },
    spl_token_swap::instruction,
};

pub fn add_liquidity(
    accounts: &[AccountInfo],
    max_token_a_amount: u64,
    max_token_b_amount: u64,
) -> ProgramResult {
    msg!("Processing AmmInstruction::AddLiquidity");
    msg!("max_token_a_amount {} ", max_token_a_amount);
    msg!("max_token_b_amount {} ", max_token_b_amount);

    #[allow(clippy::deprecated_cfg_attr)]
    #[cfg_attr(rustfmt, rustfmt_skip)]
    if let [
        user_account,
        user_token_a_account,
        user_token_b_account,
        user_lp_token_account,
        pool_program_id,
        pool_token_a_account,
        pool_token_b_account,
        lp_token_mint,
        _spl_token_id,
        amm_id,
        amm_authority
        ] = accounts
    {

        let (lp_token_amount, token_a_amount, token_b_amount) = get_pool_deposit_amounts(
            pool_token_a_account,
            pool_token_b_account,
            lp_token_mint,
            max_token_a_amount,
            max_token_b_amount,
        )?;

        let initial_token_a_user_balance = account::get_token_balance(user_token_a_account)?;
        let initial_token_b_user_balance = account::get_token_balance(user_token_b_account)?;
        let initial_lp_token_user_balance = account::get_token_balance(user_lp_token_account)?;

        let data = instruction::DepositAllTokenTypes {
            pool_token_amount: lp_token_amount,
            maximum_token_a_amount: token_a_amount,
            maximum_token_b_amount: token_b_amount,
        };

        msg!("Deposit tokens into the pool. lp_token_amount: {}, token_a_amount: {}, token_b_amount: {}", lp_token_amount, token_a_amount, token_b_amount);
        let instruction = instruction::deposit_all_token_types(
            pool_program_id.key,
            &spl_token::id(),
            amm_id.key,
            amm_authority.key,
            user_account.key,
            user_token_a_account.key,
            user_token_b_account.key,
            pool_token_a_account.key,
            pool_token_b_account.key,
            lp_token_mint.key,
            user_lp_token_account.key,
            data,
        )?;

        invoke(&instruction, accounts)?;

        account::check_tokens_spent(
            user_token_a_account,
            initial_token_a_user_balance,
            token_a_amount,
        )?;
        account::check_tokens_spent(
            user_token_b_account,
            initial_token_b_user_balance,
            token_b_amount,
        )?;
        account::check_tokens_received(
            user_lp_token_account,
            initial_lp_token_user_balance,
            lp_token_amount,
        )?;
    } else {
        return Err(ProgramError::NotEnoughAccountKeys);
    }

    msg!("AmmInstruction::AddLiquidity complete");
    Ok(())
}

pub fn swap(
    accounts: &[AccountInfo],
    token_a_amount_in: u64,
    token_b_amount_in: u64,
    min_token_amount_out: u64,
) -> ProgramResult {
    msg!("Processing AmmInstruction::Swap");
    msg!("token_a_amount_in {} ", token_a_amount_in);
    msg!("token_b_amount_in {} ", token_b_amount_in);
    msg!("min_token_amount_out {} ", min_token_amount_out);

    #[allow(clippy::deprecated_cfg_attr)]
    #[cfg_attr(rustfmt, rustfmt_skip)]
    if let [
        user_account,
        user_token_a_account,
        user_token_b_account,
        pool_program_id,
        pool_token_a_account,
        pool_token_b_account,
        lp_token_mint,
        _spl_token_id,
        amm_id,
        amm_authority,
        fees_account
        ] = accounts
    {

        let (amount_in, mut minimum_amount_out) = get_pool_swap_amounts(
            pool_token_a_account,
            pool_token_b_account,
            token_a_amount_in,
            token_b_amount_in,
        )?;
        if min_token_amount_out > minimum_amount_out {
            minimum_amount_out = min_token_amount_out;
        }

        let data = instruction::Swap {
            amount_in,
            minimum_amount_out,
        };

        msg!(
            "Swap tokens in the pool. amount_in: {}, minimum_amount_out: {}",
            amount_in,
            minimum_amount_out
        );

        if token_a_amount_in == 0 {
            let initial_balance_in = account::get_token_balance(user_token_b_account)?;
            let initial_balance_out = account::get_token_balance(user_token_a_account)?;

            let instruction = instruction::swap(
                pool_program_id.key,
                &spl_token::id(),
                amm_id.key,
                amm_authority.key,
                user_account.key,
                user_token_b_account.key,
                pool_token_b_account.key,
                pool_token_a_account.key,
                user_token_a_account.key,
                lp_token_mint.key,
                fees_account.key,
                None,
                data,
            )?;
            invoke(&instruction, accounts)?;

            account::check_tokens_spent(user_token_b_account, initial_balance_in, amount_in)?;
            account::check_tokens_received(
                user_token_a_account,
                initial_balance_out,
                minimum_amount_out,
            )?;
        } else {
            let initial_balance_in = account::get_token_balance(user_token_a_account)?;
            let initial_balance_out = account::get_token_balance(user_token_b_account)?;

            let instruction = instruction::swap(
                pool_program_id.key,
                &spl_token::id(),
                amm_id.key,
                amm_authority.key,
                user_account.key,
                user_token_a_account.key,
                pool_token_a_account.key,
                pool_token_b_account.key,
                user_token_b_account.key,
                lp_token_mint.key,
                fees_account.key,
                None,
                data,
            )?;
            invoke(&instruction, accounts)?;

            account::check_tokens_spent(user_token_a_account, initial_balance_in, amount_in)?;
            account::check_tokens_received(
                user_token_b_account,
                initial_balance_out,
                minimum_amount_out,
            )?;
        }
    } else {
        return Err(ProgramError::NotEnoughAccountKeys);
    }

    msg!("AmmInstruction::Swap complete");
    Ok(())
}

pub fn get_pool_token_balances<'a, 'b>(
    pool_token_a_account: &'a AccountInfo<'b>,
    pool_token_b_account: &'a AccountInfo<'b>,
) -> Result<(u64, u64), ProgramError> {
    Ok((
        account::get_token_balance(pool_token_a_account)?,
        account::get_token_balance(pool_token_b_account)?,
    ))
}

pub fn get_pool_deposit_amounts<'a, 'b>(
    pool_token_a_account: &'a AccountInfo<'b>,
    pool_token_b_account: &'a AccountInfo<'b>,
    lp_token_mint: &'a AccountInfo<'b>,
    max_token_a_amount: u64,
    max_token_b_amount: u64,
) -> Result<(u64, u64, u64), ProgramError> {
    if max_token_a_amount == 0 && max_token_b_amount == 0 {
        msg!("Error: At least one of token amounts must be non-zero");
        return Err(ProgramError::InvalidArgument);
    }
    let mut token_a_amount = max_token_a_amount;
    let mut token_b_amount = max_token_b_amount;
    let (token_a_balance, token_b_balance) =
        get_pool_token_balances(pool_token_a_account, pool_token_b_account)?;

    if token_a_balance == 0 || token_b_balance == 0 {
        if max_token_a_amount == 0 || max_token_b_amount == 0 {
            msg!("Error: Both amounts must be specified for the initial deposit to an empty pool");
            return Err(ProgramError::InvalidArgument);
        } else {
            return Ok((1, max_token_a_amount, max_token_b_amount));
        }
    }

    if max_token_a_amount == 0 {
        let estimated_coin_amount = math::checked_as_u64(
            token_a_balance as f64 * max_token_b_amount as f64 / (token_b_balance as f64),
        )?;
        token_a_amount = if estimated_coin_amount > 1 {
            estimated_coin_amount - 1
        } else {
            0
        };
    } else if max_token_b_amount == 0 {
        token_b_amount = math::checked_as_u64(
            token_b_balance as f64 * max_token_a_amount as f64 / (token_a_balance as f64),
        )?;
    }

    let min_lp_tokens_out = estimate_lp_tokens_amount(
        lp_token_mint,
        token_a_amount,
        token_b_amount,
        token_a_balance,
        token_b_balance,
    )?;

    Ok((
        min_lp_tokens_out,
        token_a_amount,
        math::checked_add(token_b_amount, 1)?,
    ))
}

pub fn get_pool_withdrawal_amounts<'a, 'b>(
    pool_token_a_account: &'a AccountInfo<'b>,
    pool_token_b_account: &'a AccountInfo<'b>,
    lp_token_mint: &'a AccountInfo<'b>,
    lp_token_amount: u64,
) -> Result<(u64, u64), ProgramError> {
    if lp_token_amount == 0 {
        msg!("Error: LP token amount must be non-zero");
        return Err(ProgramError::InvalidArgument);
    }
    let (token_a_balance, token_b_balance) =
        get_pool_token_balances(pool_token_a_account, pool_token_b_account)?;
    if token_a_balance == 0 && token_b_balance == 0 {
        return Ok((0, 0));
    }
    let lp_token_supply = account::get_token_supply(lp_token_mint)?;
    if lp_token_supply == 0 {
        return Ok((0, 0));
    }
    let stake = lp_token_amount as f64 / lp_token_supply as f64;

    Ok((
        math::checked_as_u64(token_a_balance as f64 * stake)?,
        math::checked_as_u64(token_b_balance as f64 * stake)?,
    ))
}

pub fn get_pool_swap_amounts<'a, 'b>(
    pool_token_a_account: &'a AccountInfo<'b>,
    pool_token_b_account: &'a AccountInfo<'b>,
    token_a_amount_in: u64,
    token_b_amount_in: u64,
) -> Result<(u64, u64), ProgramError> {
    
    let A_FEE = 0_f64;

    if (token_a_amount_in == 0 && token_b_amount_in == 0)
        || (token_a_amount_in > 0 && token_b_amount_in > 0)
    {
        msg!("Error: One and only one of token amounts must be non-zero");
        return Err(ProgramError::InvalidArgument);
    }
    let (token_a_balance, token_b_balance) =
        get_pool_token_balances(pool_token_a_account, pool_token_b_account)?;
    if token_a_balance == 0 || token_b_balance == 0 {
        msg!("Error: Can't swap in an empty pool");
        return Err(ProgramError::Custom(412));
    }
    let token_a_balance = token_a_balance as f64;
    let token_b_balance = token_b_balance as f64;
    if token_a_amount_in == 0 {
        // b to a
        let amount_in_no_fee = ((token_b_amount_in as f64 * (1.0 - A_FEE)) as u64) as f64;
        let estimated_token_a_amount = (token_a_balance
            - token_a_balance * token_b_balance / (token_b_balance + amount_in_no_fee))
            as u64;

        Ok((token_b_amount_in, estimated_token_a_amount))
    } else {
        // a to b
        let amount_in_no_fee = ((token_a_amount_in as f64 * (1.0 - A_FEE)) as u64) as f64;
        let estimated_token_b_amount = (token_b_balance
            - token_a_balance * token_b_balance / (token_a_balance + amount_in_no_fee))
            as u64;

        Ok((token_a_amount_in, estimated_token_b_amount))
    }
}

pub fn estimate_lp_tokens_amount(
    lp_token_mint: &AccountInfo,
    token_a_deposit: u64,
    token_b_deposit: u64,
    pool_token_a_balance: u64,
    pool_token_b_balance: u64,
) -> Result<u64, ProgramError> {
    if pool_token_a_balance != 0 && pool_token_b_balance != 0 {
        Ok(std::cmp::min(
            math::checked_as_u64(
                (token_a_deposit as f64 / pool_token_a_balance as f64)
                    * account::get_token_supply(lp_token_mint)? as f64,
            )?,
            math::checked_as_u64(
                (token_b_deposit as f64 / pool_token_b_balance as f64)
                    * account::get_token_supply(lp_token_mint)? as f64,
            )?,
        ))
    } else if pool_token_a_balance != 0 {
        math::checked_as_u64(
            (token_a_deposit as f64 / pool_token_a_balance as f64)
                * account::get_token_supply(lp_token_mint)? as f64,
        )
    } else if pool_token_b_balance != 0 {
        math::checked_as_u64(
            (token_b_deposit as f64 / pool_token_b_balance as f64)
                * account::get_token_supply(lp_token_mint)? as f64,
        )
    } else {
        Ok(0)
    }
}
