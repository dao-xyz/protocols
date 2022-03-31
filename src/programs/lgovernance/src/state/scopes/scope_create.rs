use std::io::Result;

use super::scope::get_scope_program_address;
use super::scope::AcceptenceCriteria;
use super::scope::InstructionConditional;
use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};
use shared::{account::MaxSize, content::ContentSource};
use solana_program::pubkey::Pubkey;
pub const MAX_CONTENT_LEN: usize = 32 // hash pubkey
    + 200; // IPFS link (and some padding)

#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, BorshSchema, PartialEq)]
pub struct CreateScope {
    pub id: Pubkey,
    pub vote_mint: Pubkey,
    pub instruction_program_id: Pubkey,
    pub instruction_condition: InstructionConditional,
    pub criteria: AcceptenceCriteria,
    pub name: Option<String>,
    pub info: Option<ContentSource>,
}

impl MaxSize for CreateScope {
    fn get_max_size(&self) -> Option<usize> {
        None
    }
}

#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, BorshSchema, PartialEq)]
pub enum VotingScopeUpdate {
    Delete(Pubkey),
    Create { scope: CreateScope, bump_seed: u8 },
}

impl VotingScopeUpdate {
    pub fn create(scope: CreateScope, program_id: &Pubkey) -> Self {
        let bump_seed = get_scope_program_address(program_id, &scope.id).1;
        Self::Create { scope, bump_seed }
    }
}
