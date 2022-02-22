



use solana_program_test::*;


pub fn program_test() -> ProgramTest {
    let mut program = ProgramTest::new(
        "lpost",
        lpost::id(),
        processor!(lpost::processor::Processor::process),
    );
    program.add_program(
        "lchannel",
        lchannel::id(),
        processor!(lchannel::processor::Processor::process),
    );

    program.add_program(
        "luser",
        luser::id(),
        processor!(luser::processor::Processor::process),
    );
    program
}
