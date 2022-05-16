use solana_program_test::*;

use lpool::processor::Processor;

pub fn program_test() -> ProgramTest {
    ProgramTest::new("lpool", lpool::id(), processor!(Processor::process))
}
