

use solana_program_test::*;


use westake::processor::Processor;

pub fn program_test() -> ProgramTest {
    ProgramTest::new("westake", westake::id(), processor!(Processor::process))
}
