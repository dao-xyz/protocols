use solana_program::pubkey::Pubkey;
use solana_program_test::ProgramTest;

use solana_program_test::*;
use solvei::{process_instruction};

pub fn program_test(program_id: Pubkey) -> ProgramTest {
    ProgramTest::new(
        "solvei",
        program_id,
        processor!(process_instruction),
    )
}
