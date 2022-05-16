use solana_program_test::*;

pub fn program_test() -> ProgramTest {
    let mut program = ProgramTest::new(
        "lgovernance",
        lgovernance::id(),
        processor!(lgovernance::processor::Processor::process),
    );
    /*  program.add_program(
        "lchannel",
        lchannel::id(),
        processor!(ltag::processor::Processor::process),
    ); */
    program.add_program(
        "ltag",
        ltag::id(),
        processor!(ltag::processor::Processor::process),
    );
    program
}
