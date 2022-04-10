use solana_program_test::*;

pub fn program_test() -> ProgramTest {
    let mut program = ProgramTest::new(
        "lsocial",
        lsocial::id(),
        processor!(lsocial::processor::Processor::process),
    );

    program.add_program(
        "ltag",
        ltag::id(),
        processor!(lsocial::processor::Processor::process),
    );

    program
}
