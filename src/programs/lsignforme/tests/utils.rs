use solana_program_test::*;

use lsignforme::processor::Processor;

pub fn program_test() -> ProgramTest {
    ProgramTest::new(
        "lsignforme",
        lsignforme::id(),
        processor!(Processor::process),
    )
}
