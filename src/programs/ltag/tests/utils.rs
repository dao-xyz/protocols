use solana_program_test::*;

use ltag::processor::Processor;

pub fn program_test() -> ProgramTest {
    ProgramTest::new("ltag", ltag::id(), processor!(Processor::process))
}
