use std::collections::HashSet;

use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};
use shared::content::ContentSource;
use solana_program::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    system_program, sysvar,
};

use crate::state::{
    channel::ChannelSigner,
    delegation::rule_delegation_record_account::get_rule_delegation_account_program_address,
    governance::get_governance_address,
    native_treasury::get_native_treasury_address,
    proposal::{
        get_proposal_address,
        proposal_option::get_proposal_option_program_address,
        proposal_transaction::{get_proposal_transaction_address, ConditionedInstruction},
        VoteType,
    },
    realm::{get_realm_mint_authority_program_address, get_realm_mint_program_address},
    rules::rule::{get_rule_program_address, RuleConfig},
    token_owner_budget_record::get_token_owner_budget_record_address,
    token_owner_record::{
        get_token_owner_delegatee_record_address, get_token_owner_record_address,
    },
    vote_record::get_vote_record_address,
};
use crate::Vote;

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, BorshSchema, PartialEq)]
pub enum CreateProposalOptionType {
    Instruction(String), // label
    Deny,
}
/*
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, BorshSchema, PartialEq)]
pub struct PostVote {
    /* pub stake: u64, */
    pub vote_record_bump_seed: u8,

    /*  pub mint_authority_bump_seed: u8,
    pub escrow_bump_seed: u8, */
} */

pub enum SignedCreateProposal {
    AuthorityTag {
        record: Pubkey,
        owner: Pubkey,
    },
    TokenOwner {
        owner_record: Pubkey,
        governing_token_owner: Pubkey,
    },
}
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, BorshSchema, PartialEq)]
pub enum PostInstruction {
    // Create channel
    CreateProposal {
        vote_type: VoteType,
        rules_count: u8,
        source: ContentSource,
        bump_seed: u8,
    },
    Delegate {
        amount: u64,
        delegation_record_bump_seed: u8,
    },
    Undelegate {
        amount: u64,
    },
    DelegateHistory,
    UndelegateHistory,
    InsertRule,
    Vote {
        vote_record_bump_seed: u8,
    },
    Unvote,
    ExecuteProposal {
        governance_bump_seed: u8,
    },
    CreateGovernance {
        bump_seed: u8,
    },
    CreateRealm {
        bump_seed: u8,
    },
    CreateRule {
        id: Pubkey,
        config: RuleConfig,
        bump_seed: u8,
    },
    CreateProposalOption {
        option_type: CreateProposalOptionType,
        bump_seed: u8,
    },
    CountMaxVoteWeights,
    CountVotes,
    CreateNativeTreasury,
    InsertTransaction {
        option_index: u16,
        instruction_index: u16,
        hold_up_time: u32,
        instructions: Vec<ConditionedInstruction>,
    },
    CreateDelegatee {
        rule: Pubkey,
        governing_token_mint: Pubkey,
        token_owner_record_bump_seed: u8,
    },
    DepositGoverningTokens {
        amount: u64,
        token_owner_record_bump_seed: u8,
    },
    CreateTokenOwnerBudgetRecord {
        rule: Pubkey,
        token_owner_budget_record_bump_seed: u8,
    },

    FinalizeDraft,
}

pub fn create_proposal(
    program_id: &Pubkey,

    // Accounts
    creator: &Pubkey,
    governance: &Pubkey,
    payer: &Pubkey,

    // Args
    proposal_index: u64,
    vote_type: VoteType,
    rules_count: u8,
    source: &ContentSource,
) -> Instruction {
    let (proposal_address, proposal_bump_seed) =
        get_proposal_address(program_id, governance, &proposal_index.to_le_bytes());

    let accounts = vec![
        AccountMeta::new(proposal_address, false),
        AccountMeta::new_readonly(*governance, false),
        AccountMeta::new_readonly(*creator, true),
        AccountMeta::new(*payer, true),
        AccountMeta::new_readonly(system_program::id(), false),
    ];

    Instruction {
        program_id: *program_id,
        data: (PostInstruction::CreateProposal {
            vote_type,
            source: source.clone(),
            rules_count,
            bump_seed: proposal_bump_seed,
        })
        .try_to_vec()
        .unwrap(),
        accounts,
    }
}

pub fn create_governance(
    program_id: &Pubkey,

    // Accounts
    channel: &Pubkey,
    channel_authority: &Pubkey,
    payer: &Pubkey,
) -> Instruction {
    let (governance_address, governance_bump_seed) = get_governance_address(program_id, channel);
    let accounts = vec![
        AccountMeta::new(governance_address, false),
        AccountMeta::new_readonly(*channel, false),
        AccountMeta::new_readonly(*channel_authority, true),
        AccountMeta::new(*payer, true),
        AccountMeta::new_readonly(system_program::id(), false),
    ];

    Instruction {
        program_id: *program_id,
        data: (PostInstruction::CreateGovernance {
            bump_seed: governance_bump_seed,
        })
        .try_to_vec()
        .unwrap(),
        accounts,
    }
}

pub fn deposit_governing_tokens(
    program_id: &Pubkey,
    // Accounts
    governing_token_source: &Pubkey,
    governing_token_owner: &Pubkey,
    governing_token_transfer_authority: &Pubkey,
    payer: &Pubkey,
    // Args
    amount: u64,
    governing_token_mint: &Pubkey,
) -> Instruction {
    let (token_owner_record_address, token_owner_record_bump_seed) =
        get_token_owner_record_address(program_id, governing_token_mint, governing_token_owner);

    let governing_token_holding_address =
        get_realm_mint_program_address(program_id, governing_token_mint).0;

    let accounts = vec![
        AccountMeta::new(governing_token_holding_address, false),
        AccountMeta::new(*governing_token_source, false),
        AccountMeta::new_readonly(*governing_token_owner, true),
        AccountMeta::new_readonly(*governing_token_transfer_authority, true),
        AccountMeta::new(token_owner_record_address, false),
        AccountMeta::new(*payer, true),
        AccountMeta::new_readonly(system_program::id(), false),
        AccountMeta::new_readonly(spl_token::id(), false),
    ];

    let instruction = PostInstruction::DepositGoverningTokens {
        amount,
        token_owner_record_bump_seed,
    };

    Instruction {
        program_id: *program_id,
        accounts,
        data: instruction.try_to_vec().unwrap(),
    }
}

pub fn create_delegatee(
    program_id: &Pubkey,
    // Accounts
    governing_token_owner: &Pubkey,
    payer: &Pubkey,
    // Args
    rule: &Pubkey,
    governing_token_mint: &Pubkey,
) -> Instruction {
    let (token_owner_record_address, token_owner_record_bump_seed) =
        get_token_owner_delegatee_record_address(
            program_id,
            governing_token_mint,
            governing_token_owner,
            rule,
        );

    let accounts = vec![
        AccountMeta::new_readonly(*governing_token_owner, true),
        AccountMeta::new(token_owner_record_address, false),
        AccountMeta::new(*payer, true),
        AccountMeta::new_readonly(system_program::id(), false),
    ];

    let instruction = PostInstruction::CreateDelegatee {
        token_owner_record_bump_seed,
        governing_token_mint: *governing_token_mint,
        rule: *rule,
    };

    Instruction {
        program_id: *program_id,
        accounts,
        data: instruction.try_to_vec().unwrap(),
    }
}

pub fn insert_transaction(
    program_id: &Pubkey,
    payer: &Pubkey,
    creator: &Pubkey,
    proposal: &Pubkey,
    option_index: u16,
    instruction_index: u16,
    hold_up_time: u32,
    instructions: Vec<ConditionedInstruction>,
) -> Instruction {
    let proposal_transaction_address = get_proposal_transaction_address(
        program_id,
        proposal,
        &option_index.to_le_bytes(),
        &instruction_index.to_le_bytes(),
    );
    let option_address =
        get_proposal_option_program_address(program_id, proposal, &option_index.to_le_bytes()).0;
    let mut accounts = vec![
        AccountMeta::new(*proposal, false),
        AccountMeta::new_readonly(*creator, true),
        AccountMeta::new(proposal_transaction_address, false),
        AccountMeta::new(option_address, false),
        AccountMeta::new(*payer, true), //  voter token owner record
        AccountMeta::new(system_program::id(), false),
        AccountMeta::new_readonly(solana_program::sysvar::rent::id(), false),
    ];
    let unique_rules = instructions
        .iter()
        .map(|i| i.rule)
        .collect::<HashSet<Pubkey>>();
    unique_rules
        .iter()
        .for_each(|rule| accounts.push(AccountMeta::new_readonly(*rule, false)));

    Instruction {
        program_id: *program_id,
        data: PostInstruction::InsertTransaction {
            hold_up_time,
            instruction_index,
            instructions,
            option_index,
        }
        .try_to_vec()
        .unwrap(),
        accounts,
    }
}

pub fn insert_rule(
    program_id: &Pubkey,

    // Accounts
    rule: &Pubkey,
    proposal: &Pubkey,
    creator: &Pubkey,
    payer: &Pubkey,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new_readonly(*rule, false),
        AccountMeta::new_readonly(*proposal, false),
        AccountMeta::new_readonly(*creator, true),
        AccountMeta::new_readonly(*payer, true),
    ];
    Instruction {
        program_id: *program_id,
        data: PostInstruction::InsertRule.try_to_vec().unwrap(),
        accounts,
    }
}

pub fn create_proposal_option(
    program_id: &Pubkey,
    // Accounts
    creator: &Pubkey,
    payer: &Pubkey,
    proposal: &Pubkey,

    // Args
    proposal_option_type: &CreateProposalOptionType,
    option_index: u16,
) -> Instruction {
    let (proposal_option_address, proposal_option_bump_seed) =
        get_proposal_option_program_address(program_id, proposal, &option_index.to_le_bytes());
    let accounts = vec![
        AccountMeta::new(proposal_option_address, false),
        AccountMeta::new(*proposal, false),
        AccountMeta::new_readonly(*creator, true),
        AccountMeta::new(*payer, true),
        AccountMeta::new(system_program::id(), false),
    ];
    Instruction {
        program_id: *program_id,
        data: PostInstruction::CreateProposalOption {
            option_type: proposal_option_type.clone(),
            bump_seed: proposal_option_bump_seed,
        }
        .try_to_vec()
        .unwrap(),
        accounts,
    }
}

pub fn finalize_draft(
    program_id: &Pubkey,
    // Accounts
    creator: &Pubkey,
    proposal: &Pubkey,
    governance: &Pubkey,
    rules: &Vec<(Pubkey, SignedCreateProposal)>,
) -> Instruction {
    let mut accounts = vec![
        AccountMeta::new(*proposal, false),
        AccountMeta::new(*governance, false),
        AccountMeta::new_readonly(*creator, true),
    ];
    for (rule_address, signed_create_proposal) in rules {
        accounts.push(AccountMeta::new_readonly(*rule_address, false));
        match signed_create_proposal {
            SignedCreateProposal::AuthorityTag { owner, record } => {
                accounts.push(AccountMeta::new_readonly(*record, false));
                accounts.push(AccountMeta::new_readonly(*owner, true));
            }
            SignedCreateProposal::TokenOwner {
                governing_token_owner,
                owner_record,
            } => {
                accounts.push(AccountMeta::new_readonly(*owner_record, false));
                accounts.push(AccountMeta::new_readonly(*governing_token_owner, true));
            }
        }
    }

    Instruction {
        program_id: *program_id,
        data: PostInstruction::FinalizeDraft.try_to_vec().unwrap(),
        accounts,
    }
}

pub fn cast_vote(
    program_id: &Pubkey,
    payer: &Pubkey,
    proposal: &Pubkey,
    token_record: &Pubkey,
    governing_token_owner: &Pubkey,
    rule: &Pubkey,
    options: &Vec<Pubkey>,
    last_vote_record: Option<&Pubkey>,
    delegated: bool,
) -> Instruction {
    let (vote_record, vote_record_bump_seed) =
        get_vote_record_address(program_id, proposal, &token_record, &rule);

    let mut accounts = vec![
        AccountMeta::new(*proposal, false),
        AccountMeta::new(vote_record, false),
        AccountMeta::new(*token_record, false),
        AccountMeta::new_readonly(*governing_token_owner, true),
        AccountMeta::new_readonly(*rule, false),
        AccountMeta::new(*payer, true),
        AccountMeta::new(system_program::id(), false),
    ];
    match delegated {
        false => {
            // If not delegated, we include the budget
            // this because delegated token owner records
            // does not have budgets (i.e. delegated again)
            let (budget_record_address, _) =
                get_token_owner_budget_record_address(program_id, token_record, rule);
            accounts.push(AccountMeta::new_readonly(budget_record_address, false));
        }
        true => {}
    };

    if let Some(last_vote) = last_vote_record {
        accounts.push(AccountMeta::new(*last_vote, false));
    }

    for option in options {
        accounts.push(AccountMeta::new(*option, false))
    }

    Instruction {
        program_id: *program_id,
        data: (PostInstruction::Vote {
            vote_record_bump_seed,
        })
        .try_to_vec()
        .unwrap(),
        accounts,
    }
}

pub fn uncast_vote(
    program_id: &Pubkey,

    // Accounts
    proposal: &Pubkey,
    token_record: &Pubkey,
    governing_token_owner: &Pubkey,
    beneficiary: &Pubkey,

    // Args
    rule: &Pubkey,
    options: &Vec<Pubkey>,
) -> Instruction {
    let (vote_record, _) = get_vote_record_address(program_id, proposal, &token_record, &rule);

    let mut accounts = vec![
        AccountMeta::new(*proposal, false),
        AccountMeta::new(vote_record, false),
        AccountMeta::new(*token_record, false),
        AccountMeta::new_readonly(*governing_token_owner, true),
        AccountMeta::new_readonly(*rule, false),
        AccountMeta::new(*beneficiary, false),
    ];

    for option in options {
        accounts.push(AccountMeta::new(*option, false))
    }
    Instruction {
        program_id: *program_id,
        data: (PostInstruction::Unvote).try_to_vec().unwrap(),
        accounts,
    }
}

pub fn create_token_owner_budget_record(
    program_id: &Pubkey,
    payer: &Pubkey,
    token_record: &Pubkey,
    rule: &Pubkey,
) -> Instruction {
    let (token_owner_budget_record, token_owner_budget_record_bump_seed) =
        get_token_owner_budget_record_address(program_id, token_record, rule);

    let accounts = vec![
        AccountMeta::new_readonly(*token_record, false),
        AccountMeta::new(token_owner_budget_record, false),
        AccountMeta::new(*payer, true),
        AccountMeta::new(system_program::id(), false),
    ];

    Instruction {
        program_id: *program_id,
        data: (PostInstruction::CreateTokenOwnerBudgetRecord {
            rule: *rule,
            token_owner_budget_record_bump_seed,
        })
        .try_to_vec()
        .unwrap(),
        accounts,
    }
}

pub fn count_vote_max_weights(
    program_id: &Pubkey,
    proposal: &Pubkey,
    rule_mints: &Vec<(Pubkey, Vec<Pubkey>)>,
) -> Instruction {
    let mut accounts = vec![AccountMeta::new(*proposal, false)];
    for (rule, mints) in rule_mints {
        accounts.push(AccountMeta::new_readonly(*rule, false));
        for mint in mints {
            accounts.push(AccountMeta::new_readonly(*mint, false))
        }
    }
    Instruction {
        program_id: *program_id,
        data: (PostInstruction::CountMaxVoteWeights).try_to_vec().unwrap(),
        accounts,
    }
}

pub fn count_votes(
    program_id: &Pubkey,
    proposal: &Pubkey,
    option: &Pubkey,
    deny_option: Option<&Pubkey>,
    rules: &Vec<Pubkey>,
) -> Instruction {
    let mut accounts = vec![
        AccountMeta::new(*proposal, false),
        AccountMeta::new(*option, false),
    ];
    if let Some(key) = deny_option {
        accounts.push(AccountMeta::new_readonly(*key, false));
    }
    for rule in rules {
        accounts.push(AccountMeta::new_readonly(*rule, false));
    }
    Instruction {
        program_id: *program_id,
        data: (PostInstruction::CountVotes).try_to_vec().unwrap(),
        accounts,
    }
}

pub fn create_realm(
    program_id: &Pubkey,

    // Accounts
    mint: &Pubkey,
    payer: &Pubkey,
) -> Instruction {
    let (token_holding_info, token_holding_bump_seed) =
        get_realm_mint_program_address(program_id, mint);
    let authority = get_realm_mint_authority_program_address(program_id, mint).0;
    let accounts = vec![
        AccountMeta::new(token_holding_info, false),
        AccountMeta::new_readonly(authority, false),
        AccountMeta::new(*mint, false),
        AccountMeta::new(*payer, true),
        AccountMeta::new(system_program::id(), false),
        AccountMeta::new_readonly(spl_token::id(), false),
        AccountMeta::new_readonly(sysvar::rent::id(), false),
    ];

    Instruction {
        program_id: *program_id,
        data: (PostInstruction::CreateRealm {
            bump_seed: token_holding_bump_seed,
        })
        .try_to_vec()
        .unwrap(),
        accounts,
    }
}
pub fn create_rule(
    program_id: &Pubkey,
    // Accounts
    id: &Pubkey,
    governance: &Pubkey,
    payer: &Pubkey,

    channel_signer: &Option<ChannelSigner>,
    // Args
    config: &RuleConfig,
) -> Instruction {
    let (create_rule_address, create_rule_address_bump_seed) =
        get_rule_program_address(program_id, &id);
    let mut accounts = vec![
        AccountMeta::new(create_rule_address, false),
        AccountMeta::new_readonly(*governance, !channel_signer.is_some()),
        AccountMeta::new(*payer, true),
        AccountMeta::new(system_program::id(), false),
    ];

    if let Some(signer) = channel_signer {
        accounts.push(AccountMeta::new_readonly(signer.authority, true));
        for channel in &signer.channel_path {
            accounts.push(AccountMeta::new_readonly(*channel, false));
        }
    }

    Instruction {
        program_id: *program_id,
        data: (PostInstruction::CreateRule {
            id: *id,
            config: config.clone(),
            bump_seed: create_rule_address_bump_seed,
        })
        .try_to_vec()
        .unwrap(),
        accounts,
    }
}
/**
 * Execute post with most stringent rules
 *  (i.e. if execution treasury transfer, it will assume there exist a rule that defines exactly how that transaction can be performed)
 *
 */
pub fn execute_transaction(
    program_id: &Pubkey,

    // Accounts
    channel: &Pubkey,
    proposal: &Pubkey,
    proposal_transaction: &Pubkey,
    proposal_option: &Pubkey,
    instruction_accounts: &[AccountMeta],
) -> Instruction {
    let (governance, governance_bump_seed) = get_governance_address(program_id, channel);

    let mut accounts = vec![
        AccountMeta::new_readonly(governance, false),
        AccountMeta::new(*proposal, false),
        AccountMeta::new(*proposal_transaction, false),
        AccountMeta::new(*proposal_option, false),
    ];

    accounts.extend_from_slice(instruction_accounts);

    let instruction = PostInstruction::ExecuteProposal {
        governance_bump_seed,
    };

    Instruction {
        program_id: *program_id,
        accounts,
        data: instruction.try_to_vec().unwrap(),
    }
}

pub fn delegate(
    program_id: &Pubkey,

    // Accounts
    token_owner_record: &Pubkey,
    token_owner_budget_record: &Pubkey,
    governing_token_owner: &Pubkey,
    delegatee_token_owner_record: &Pubkey,
    delegatee_governing_token_owner: &Pubkey,
    payer: &Pubkey,

    // Args
    amount: &u64,
    rule: &Pubkey,
) -> Instruction {
    let (delegation_record, delegation_record_bump_seed) =
        get_rule_delegation_account_program_address(
            program_id,
            token_owner_record,
            delegatee_token_owner_record,
            rule,
        );

    let accounts = vec![
        AccountMeta::new(delegation_record, false),
        AccountMeta::new_readonly(*token_owner_record, false),
        AccountMeta::new(*token_owner_budget_record, false),
        AccountMeta::new_readonly(*governing_token_owner, true),
        AccountMeta::new(*delegatee_token_owner_record, false),
        AccountMeta::new_readonly(*delegatee_governing_token_owner, false),
        AccountMeta::new(*payer, true),
        AccountMeta::new(system_program::id(), false),
    ];

    let instruction = PostInstruction::Delegate {
        amount: *amount,
        delegation_record_bump_seed,
    };

    Instruction {
        program_id: *program_id,
        accounts,
        data: instruction.try_to_vec().unwrap(),
    }
}

pub fn undelegate(
    program_id: &Pubkey,

    // Accounts
    delegation_record: &Pubkey,
    token_owner_record: &Pubkey,
    token_owner_budget_record: &Pubkey,
    governing_token_owner: &Pubkey,
    delegatee_token_owner_record: &Pubkey,
    delegatee_governing_token_owner: &Pubkey,
    beneficiary: &Pubkey,

    // Args
    amount: &u64,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(*delegation_record, false),
        AccountMeta::new_readonly(*token_owner_record, false),
        AccountMeta::new(*token_owner_budget_record, false),
        AccountMeta::new_readonly(*governing_token_owner, true),
        AccountMeta::new(*delegatee_token_owner_record, false),
        AccountMeta::new_readonly(*delegatee_governing_token_owner, false),
        AccountMeta::new(*beneficiary, false),
    ];

    let instruction = PostInstruction::Undelegate { amount: *amount };

    Instruction {
        program_id: *program_id,
        accounts,
        data: instruction.try_to_vec().unwrap(),
    }
}
/*
pub fn sync_delegation(
    program_id: &Pubkey,

    // Accounts
    proposal: &Pubkey,
    vote_record: &Pubkey,
    delegator_token_owner_record: &Pubkey,
    governing_token_owner_record: &Pubkey,
    delegatee_token_owner_record: &Pubkey,
    rule: &Pubkey,
    rule_delegation_record: &Pubkey,
    options: &Vec<Pubkey>,

    // Args
    sync: &bool,
) -> Instruction {
    let mut accounts = vec![
        AccountMeta::new(*proposal, false),
        AccountMeta::new(*vote_record, false),
        AccountMeta::new(*rule_delegation_record, false),
        AccountMeta::new_readonly(*delegator_token_owner_record, false),
        AccountMeta::new_readonly(*governing_token_owner_record, true),
        AccountMeta::new_readonly(*rule, false),
    ];

    for option in options {
        accounts.push(AccountMeta::new(*option, false))
    }

    let instruction = PostInstruction::SyncDelegation {
        sync: *sync,
        delegatee_token_owner_record: *delegatee_token_owner_record,
    };

    Instruction {
        program_id: *program_id,
        accounts,
        data: instruction.try_to_vec().unwrap(),
    }
} */

pub fn delegate_history(
    program_id: &Pubkey,

    // Accounts
    vote_record: &Pubkey,
    previous_vote_record: &Pubkey,
    previous_proposal: &Pubkey,
    previous_vote_options: &Vec<Pubkey>,
    rule_delegation_record: &Pubkey,
    delegator_token_owner_record: &Pubkey,
    delegator_governing_token_owner_record: &Pubkey,
    rule: &Pubkey,
) -> Instruction {
    let mut accounts = vec![
        AccountMeta::new_readonly(*vote_record, false),
        AccountMeta::new(*previous_vote_record, false),
        AccountMeta::new(*previous_proposal, false),
        AccountMeta::new(*rule_delegation_record, false),
        AccountMeta::new_readonly(*delegator_token_owner_record, false),
        AccountMeta::new_readonly(*delegator_governing_token_owner_record, true),
        AccountMeta::new_readonly(*rule, false),
    ];

    for option in previous_vote_options {
        accounts.push(AccountMeta::new(*option, false))
    }

    Instruction {
        program_id: *program_id,
        accounts,
        data: PostInstruction::DelegateHistory.try_to_vec().unwrap(),
    }
}

pub fn undelegate_history(
    program_id: &Pubkey,

    // Accounts
    vote_record: &Pubkey,
    proposal: &Pubkey,
    options: &Vec<Pubkey>,
    rule_delegation_record: &Pubkey,
    delegator_token_owner_record: &Pubkey,
    delegator_governing_token_owner_record: &Pubkey,
    rule: &Pubkey,
) -> Instruction {
    let mut accounts = vec![
        AccountMeta::new(*vote_record, false),
        AccountMeta::new(*proposal, false),
        AccountMeta::new(*rule_delegation_record, false),
        AccountMeta::new_readonly(*delegator_token_owner_record, false),
        AccountMeta::new_readonly(*delegator_governing_token_owner_record, true),
        AccountMeta::new_readonly(*rule, false),
    ];

    for option in options {
        accounts.push(AccountMeta::new(*option, false))
    }

    Instruction {
        program_id: *program_id,
        accounts,
        data: PostInstruction::UndelegateHistory.try_to_vec().unwrap(),
    }
}

/// Creates CreateNativeTreasury instruction
pub fn create_native_treasury(
    program_id: &Pubkey,
    // Accounts
    governance: &Pubkey,
    payer: &Pubkey,
) -> Instruction {
    let native_treasury_address = get_native_treasury_address(program_id, governance);

    let accounts = vec![
        AccountMeta::new_readonly(*governance, false),
        AccountMeta::new(native_treasury_address, false),
        AccountMeta::new(*payer, true),
        AccountMeta::new_readonly(system_program::id(), false),
    ];

    let instruction = PostInstruction::CreateNativeTreasury {};

    Instruction {
        program_id: *program_id,
        accounts,
        data: instruction.try_to_vec().unwrap(),
    }
}
