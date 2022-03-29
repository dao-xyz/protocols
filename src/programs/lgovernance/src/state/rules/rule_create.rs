use std::io::Result;

use super::rule::get_rule_program_address;
use super::rule::AcceptenceCriteria;
use super::rule::InstructionConditional;
use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};
use shared::{account::MaxSize, content::ContentSource};
use solana_program::pubkey::Pubkey;
pub const MAX_CONTENT_LEN: usize = 32 // hash pubkey
    + 200; // IPFS link (and some padding)

#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, BorshSchema, PartialEq)]
pub struct CreateRule {
    pub id: Pubkey,
    pub vote_mint: Pubkey,
    pub instruction_program_id: Pubkey,
    pub instruction_condition: InstructionConditional,
    pub criteria: AcceptenceCriteria,
    pub name: Option<String>,
    pub info: Option<ContentSource>,
}

impl MaxSize for CreateRule {
    fn get_max_size(&self) -> Option<usize> {
        None
    }
}

#[derive(Clone, Debug, BorshDeserialize, BorshSerialize, BorshSchema, PartialEq)]
pub enum VotingRuleUpdate {
    Delete(Pubkey),
    Create { rule: CreateRule, bump_seed: u8 },
}

impl VotingRuleUpdate {
    pub fn create(rule: CreateRule, program_id: &Pubkey) -> Self {
        let bump_seed = get_rule_program_address(program_id, &rule.id).1;
        Self::Create { rule, bump_seed }
    }
}
