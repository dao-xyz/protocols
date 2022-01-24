use std::convert::TryInto;

use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};
use solana_program::{
    account_info::AccountInfo, program_error::ProgramError, program_pack::Pack, pubkey::Pubkey,
};
use spl_math::checked_ceil_div::CheckedCeilDiv;
use spl_token_swap::error::SwapError;

use crate::{
    shared::account::{get_token_balance, get_token_supply},
    social::Vote,
    tokens::spl_utils::{token_transfer, token_transfer_signed},
};

pub fn find_post_mint_token_account(
    program_id: &Pubkey,
    post: &Pubkey,
    mint: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[post.as_ref(), mint.as_ref()], program_id)
}
/// Encodes all results of swapping from a source token to a destination token
#[derive(Debug, PartialEq)]
pub struct SwapWithoutFeesResult {
    /// Amount of source token swapped
    pub source_amount_swapped: u128,
    /// Amount of destination token swapped
    pub destination_amount_swapped: u128,
}
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, BorshSchema, PartialEq)]
pub enum TradeDirection {
    AToB,
    BToA,
}

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, BorshSchema, PartialEq)]
pub struct LongShortSwap {
    pub curve: LongShortCurve,
    pub utility_token_account: Pubkey,
    pub long_token_account: Pubkey,
    pub short_token_account: Pubkey,
    pub token_program_id: Pubkey,
}

pub enum LongShortSwapDirection {
    BuyLong,
    SellLong,
    BuyShort,
    SellShort,
}
fn to_u64(val: u128) -> Result<u64, SwapError> {
    val.try_into().map_err(|_| SwapError::ConversionFailure)
}

impl LongShortSwap {
    pub fn unpack_token_account(
        &self,
        account_info: &AccountInfo,
    ) -> Result<spl_token::state::Account, ProgramError> {
        if account_info.owner != &self.token_program_id {
            Err(ProgramError::InvalidArgument)
        } else {
            spl_token::state::Account::unpack(&account_info.data.borrow())
        }
    }

    pub fn swap<'a>(
        &self,
        source_token_account_info: &AccountInfo<'a>,
        destination_token_account_info: &AccountInfo<'a>,
        utility_token_account_info: &AccountInfo<'a>,
        long_token_account_info: &AccountInfo<'a>,
        short_token_account_info: &AccountInfo<'a>,
        trade_direction: LongShortSwapDirection,
        source_amount: u128,
        user_transfer_authority_info: &AccountInfo<'a>,
        swap_authority_info: &AccountInfo<'a>,
        swap_authority_seeds: &[&[u8]],
        token_program_info: &AccountInfo<'a>,
    ) -> Result<(), ProgramError> {
        let utility_token_account = self.unpack_token_account(utility_token_account_info)?;
        let long_token_account = self.unpack_token_account(long_token_account_info)?;
        let short_token_account = self.unpack_token_account(short_token_account_info)?;
        let (result, swap_source_info, swap_destination_info) = match trade_direction {
            LongShortSwapDirection::BuyLong => (
                self.curve
                    .swap(
                        source_amount,
                        utility_token_account.amount as u128,
                        long_token_account.amount as u128,
                        short_token_account.amount as u128,
                        TradeDirection::BToA,
                    )
                    .unwrap(),
                utility_token_account_info,
                long_token_account_info,
            ),
            LongShortSwapDirection::SellLong => (
                self.curve
                    .swap(
                        source_amount,
                        long_token_account.amount as u128,
                        utility_token_account.amount as u128,
                        short_token_account.amount as u128,
                        TradeDirection::AToB,
                    )
                    .unwrap(),
                long_token_account_info,
                utility_token_account_info,
            ),
            LongShortSwapDirection::BuyShort => (
                self.curve
                    .swap(
                        source_amount,
                        utility_token_account.amount as u128,
                        short_token_account.amount as u128,
                        long_token_account.amount as u128,
                        TradeDirection::BToA,
                    )
                    .unwrap(),
                utility_token_account_info,
                short_token_account_info,
            ),

            LongShortSwapDirection::SellShort => (
                self.curve
                    .swap(
                        source_amount,
                        short_token_account.amount as u128,
                        utility_token_account.amount as u128,
                        long_token_account.amount as u128,
                        TradeDirection::AToB,
                    )
                    .unwrap(),
                short_token_account_info,
                utility_token_account_info,
            ),
        };

        token_transfer(
            token_program_info.clone(),
            source_token_account_info.clone(),
            swap_source_info.clone(),
            user_transfer_authority_info.clone(), // Signed authority
            to_u64(result.source_amount_swapped)?,
        )?;

        token_transfer_signed(
            token_program_info.clone(),
            swap_destination_info.clone(),
            destination_token_account_info.clone(),
            swap_authority_info.clone(),
            swap_authority_seeds,
            to_u64(result.destination_amount_swapped)?,
        )?;

        Ok(())
    }
}

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, BorshSchema, PartialEq)]
pub struct LongShortCurve {
    pub token_b_offset: u64,
}

/// Helper function for mapping to SwapError::CalculationFailure
pub fn map_zero_to_none(x: u128) -> Option<u128> {
    if x == 0 {
        None
    } else {
        Some(x)
    }
}

// Constant product swap
pub fn swap(
    source_amount: u128,
    swap_source_amount: u128,
    swap_destination_amount: u128,
) -> Option<SwapWithoutFeesResult> {
    let invariant = swap_source_amount.checked_mul(swap_destination_amount)?;

    let new_swap_source_amount = swap_source_amount.checked_add(source_amount)?;
    let (new_swap_destination_amount, new_swap_source_amount) =
        invariant.checked_ceil_div(new_swap_source_amount)?;

    let source_amount_swapped = new_swap_source_amount.checked_sub(swap_source_amount)?;
    let destination_amount_swapped =
        map_zero_to_none(swap_destination_amount.checked_sub(new_swap_destination_amount)?)?;

    Some(SwapWithoutFeesResult {
        source_amount_swapped,
        destination_amount_swapped,
    })
}

// A two sided offset swap, where one side offset is dynamically affected
// by the supply of the mirror token in another swap pool
impl LongShortCurve {
    pub fn swap(
        &self,
        source_amount: u128,
        swap_source_amount: u128,
        swap_destination_amount: u128,
        swap_a_anti_amount: u128, // shorted amount of A
        trade_direction: TradeDirection,
    ) -> Option<SwapWithoutFeesResult> {
        let token_b_offset = self.token_b_offset as u128;
        let swap_source_amount = match trade_direction {
            TradeDirection::AToB => swap_source_amount.checked_add(swap_a_anti_amount)?,
            TradeDirection::BToA => swap_source_amount.checked_add(token_b_offset)?,
        };
        let swap_destination_amount = match trade_direction {
            TradeDirection::AToB => swap_destination_amount.checked_add(token_b_offset)?,
            TradeDirection::BToA => swap_destination_amount.checked_add(swap_a_anti_amount)?,
        };
        swap(source_amount, swap_source_amount, swap_destination_amount)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn swap_offset() {
        let swap_source_amount: u128 = 1_000_000;
        let swap_destination_amount: u128 = 0;
        let source_amount: u128 = 100;
        let token_b_offset = 1_000_000;
        let curve = LongShortCurve { token_b_offset };
        let result = curve
            .swap(
                source_amount,
                swap_source_amount,
                swap_destination_amount,
                0,
                TradeDirection::AToB,
            )
            .unwrap();
        assert_eq!(result.source_amount_swapped, source_amount);
        assert_eq!(result.destination_amount_swapped, source_amount - 1);

        let bad_result = curve.swap(
            source_amount,
            swap_source_amount,
            swap_destination_amount,
            0,
            TradeDirection::BToA,
        );
        assert!(bad_result.is_none());
    }

    #[test]
    fn swap_offset_opposite() {
        let swap_source_amount: u128 = 1_000_000;
        let swap_destination_amount: u128 = 0;
        let source_amount: u128 = 100;
        let curve = LongShortCurve { token_b_offset: 0 };
        let a_anti_amount: u128 = 1_000_000;
        let result = curve
            .swap(
                source_amount,
                swap_source_amount,
                swap_destination_amount,
                a_anti_amount,
                TradeDirection::BToA,
            )
            .unwrap();
        assert_eq!(result.source_amount_swapped, source_amount);
        assert_eq!(result.destination_amount_swapped, source_amount - 1);

        let bad_result = curve.swap(
            source_amount,
            swap_source_amount,
            swap_destination_amount,
            a_anti_amount,
            TradeDirection::AToB,
        );
        assert!(bad_result.is_none());
    }

    fn test_truncation(
        curve: &LongShortCurve,
        source_amount: u128,
        swap_source_amount: u128,
        swap_destination_amount: u128,
        swap_opposity_supply: u128,
        expected_source_amount_swapped: u128,
        expected_destination_amount_swapped: u128,
    ) {
        let invariant = swap_source_amount * swap_destination_amount;
        let result = curve
            .swap(
                source_amount,
                swap_source_amount,
                swap_destination_amount,
                swap_opposity_supply,
                TradeDirection::AToB,
            )
            .unwrap();
        assert_eq!(result.source_amount_swapped, expected_source_amount_swapped);
        assert_eq!(
            result.destination_amount_swapped,
            expected_destination_amount_swapped
        );
        let new_invariant = (swap_source_amount + result.source_amount_swapped)
            * (swap_destination_amount - result.destination_amount_swapped);
        assert!(new_invariant >= invariant);
    }

    #[test]
    fn constant_product_swap_rounding() {
        let curve = LongShortCurve { token_b_offset: 0 };

        // much too small
        assert!(curve
            .swap(10, 70_000_000_000, 4_000_000, 0, TradeDirection::AToB)
            .is_none()); // spot: 10 * 4m / 70b = 0

        let tests: &[(u128, u128, u128, u128, u128)] = &[
            (10, 4_000_000, 70_000_000_000, 10, 174_999), // spot: 10 * 70b / ~4m = 174,999.99
            (20, 30_000 - 20, 10_000, 18, 6), // spot: 20 * 1 / 3.000 = 6.6667 (source can be 18 to get 6 dest.)
            (19, 30_000 - 20, 10_000, 18, 6), // spot: 19 * 1 / 2.999 = 6.3334 (source can be 18 to get 6 dest.)
            (18, 30_000 - 20, 10_000, 18, 6), // spot: 18 * 1 / 2.999 = 6.0001
            (10, 20_000, 30_000, 10, 14),     // spot: 10 * 3 / 2.0010 = 14.99
            (10, 20_000 - 9, 30_000, 10, 14), // spot: 10 * 3 / 2.0001 = 14.999
            (10, 20_000 - 10, 30_000, 10, 15), // spot: 10 * 3 / 2.0000 = 15
            (100, 60_000, 30_000, 99, 49), // spot: 100 * 3 / 6.001 = 49.99 (source can be 99 to get 49 dest.)
            (99, 60_000, 30_000, 99, 49),  // spot: 99 * 3 / 6.001 = 49.49
            (98, 60_000, 30_000, 97, 48), // spot: 98 * 3 / 6.001 = 48.99 (source can be 97 to get 48 dest.)
        ];
        for (
            source_amount,
            swap_source_amount,
            swap_destination_amount,
            expected_source_amount,
            expected_destination_amount,
        ) in tests.iter()
        {
            test_truncation(
                &curve,
                *source_amount,
                *swap_source_amount,
                *swap_destination_amount,
                0,
                *expected_source_amount,
                *expected_destination_amount,
            );
        }
    }
}
