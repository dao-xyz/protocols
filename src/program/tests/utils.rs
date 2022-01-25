

use solana_program_test::*;

use solana_sdk::pubkey::Pubkey;
use westake::processor::Processor;

pub fn program_test() -> ProgramTest {
    ProgramTest::new("westake", westake::id(), processor!(Processor::process))
}
