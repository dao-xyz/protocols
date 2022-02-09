

use solana_program_test::*;


use s2g::processor::Processor;

pub fn program_test() -> ProgramTest {
    ProgramTest::new("s2g", s2g::id(), processor!(Processor::process))
}
