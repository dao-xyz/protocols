use solana_program::{account_info::AccountInfo, program_error::ProgramError};
static LAMPORT_SOL_FACTOR: f64 = 1_f64 / 1000000000_f64;

fn get_exchange_rate_at_stake(supply: f64, stagnation_factor: f64) -> f64 {
    1_f64 / (1_f64 + supply * stagnation_factor * LAMPORT_SOL_FACTOR)
}

fn get_allowed_mint_amount_from_sol_supply_and_sol_stake(
    sol_supply: u64,
    sol_stake: u64,
    spread: u64,
) -> Result<u64, ProgramError> {
    // let token_supply = Mint::unpack_from_slice(&mint_account_info.data.borrow())?.supply;
    let sol_stake_float = sol_stake as f64;
    let half_stake = sol_stake_float / 2_f64;
    let rate = get_exchange_rate_at_stake(sol_supply as f64 + half_stake, spread as f64);
    // This should be fine tuned to promote wanted behaviours of the system
    Ok((rate * sol_stake_float) as u64)
}

pub fn get_allowed_mint_amount<'a>(
    escrow_account_info: &AccountInfo<'a>,
    stake: u64,
    stagnation_factor: u64,
) -> u64 {
    let supply = escrow_account_info.lamports();

    // let token_supply = Mint::unpack_from_slice(&mint_account_info.data.borrow())?.supply;

    // This should be fine tuned to promote wanted behaviours of the system
    get_allowed_mint_amount_from_sol_supply_and_sol_stake(supply, stake, stagnation_factor)
        .unwrap()
    //get_allowed_mint_amount_from_sol_supply_and_sol_stake(sol_supply, sol_stake);
}

#[cfg(test)]
mod test {

    use super::get_allowed_mint_amount_from_sol_supply_and_sol_stake;

    #[test]
    fn test_constant() {
        let x = get_allowed_mint_amount_from_sol_supply_and_sol_stake(100, 100, 0).unwrap();
        assert_eq!(x, 100);
    }

    #[test]
    fn test_constant_offset() {
        let x = get_allowed_mint_amount_from_sol_supply_and_sol_stake(10000, 100, 0).unwrap();
        assert_eq!(x, 100);
    }

    #[test]
    fn test_linear() {
        let x = get_allowed_mint_amount_from_sol_supply_and_sol_stake(100, 100, 1).unwrap();
        assert_eq!(x, 99);
    }

    #[test]
    fn test_linear_2() {
        let x = get_allowed_mint_amount_from_sol_supply_and_sol_stake(100, 100000, 10000).unwrap();
        assert_eq!(x, 66622);
    }
}
