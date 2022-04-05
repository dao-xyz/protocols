use crate::state::{
    proposal::{
        get_proposal_data,
    },
    scopes::{
        scope::{get_scope_data},
        scope_weight::ScopeWeight,
    },
};

use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    pubkey::Pubkey,
};

pub fn process_insert_scope(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let scope_info = next_account_info(account_info_iter)?; // 1
    let proposal_info = next_account_info(account_info_iter)?; // 1
    let creator_info = next_account_info(account_info_iter)?; // 0
    let mut proposal_data = get_proposal_data(program_id, proposal_info)?;
    proposal_data.assert_can_edit_scopes(creator_info)?;

    let _scope_data = get_scope_data(program_id, scope_info)?;

    proposal_data.scopes_max_vote_weight.push(ScopeWeight {
        scope: *scope_info.key,
        weight: 0, // max vote weight is calculated later
    });

    /*    proposal_data
    .common_scope_config
    .set_strictest(&scope_data.config.vote_config.vote_tipping); */

    proposal_data.serialize(&mut *proposal_info.data.borrow_mut())?;
    Ok(())
}
