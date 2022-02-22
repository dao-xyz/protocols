use solana_program_test::*;

use luser::processor::Processor;

pub fn program_test() -> ProgramTest {
    ProgramTest::new("luser", luser::id(), processor!(Processor::process))
}
