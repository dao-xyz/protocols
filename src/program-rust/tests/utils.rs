use solana_program::{
    account_info::{AccountInfo, IntoAccountInfo},
    entrypoint::ProgramResult,
    hash::Hash,
    instruction::{AccountMeta, Instruction},
    program_error::PrintProgramError,
    program_option::COption,
    program_pack::Pack,
    rent::Rent,
    system_instruction, system_program,
    sysvar::Sysvar,
};
use spl_associated_token_account::{create_associated_token_account, get_associated_token_address};
use std::str::FromStr;

use solana_program_test::*;

use solana_sdk::{
    account::Account, pubkey::Pubkey, signature::Keypair, signer::Signer, transaction::Transaction,
};
use spl_token::{
    instruction::{initialize_mint, mint_to},
    state::Mint,
};
use westake::{id, processor::Processor};

fn swap_pool_process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    //Processor::process(program_id, accounts, instruction_data)
    if let Err(error) =
        spl_token_swap::processor::Processor::process(program_id, accounts, instruction_data)
    {
        // catch the error so we can print it
        error.print::<spl_token_swap::error::SwapError>();
        Err(error)
    } else {
        Ok(())
    }
}

pub fn program_test() -> ProgramTest {
    let mut program = ProgramTest::new("westake", westake::id(), processor!(Processor::process));
    program
}
