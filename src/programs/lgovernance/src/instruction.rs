use std::collections::HashSet;

use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};
use shared::content::ContentSource;
use solana_program::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    system_program,
};
use spl_associated_token_account::get_associated_token_address;

use crate::state::{
    post::PostAccount,
    proposal::{
        proposal_option::find_proposal_option_program_address,
        proposal_transaction::{get_proposal_transaction_address, ConditionedInstruction},
        VoteType,
    },
    realm::find_realm_mint_program_address,
    rules::rule::{find_create_rule_associated_program_address, RuleTimeConfig, RuleVoteConfig},
    token_owner_record::{
        get_token_owner_delegatee_record_address, get_token_owner_record_address,
    },
};
use crate::{
    find_escrow_program_address, find_post_downvote_mint_program_address,
    find_post_mint_authority_program_address, find_post_program_address,
    find_post_upvote_mint_program_address, Vote,
};

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, BorshSchema, PartialEq)]
pub enum CreatePostType {
    InformationPost,
    Proposal { vote_type: VoteType },
}

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, BorshSchema, PartialEq)]
pub enum CreateProposalOptionType {
    Instruction(String), // label
    Deny,
}

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, BorshSchema, PartialEq)]
pub struct CreatePost {
    pub hash: [u8; 32],
    pub source: ContentSource,
    pub post_type: CreatePostType,
    pub rules: Vec<Pubkey>,
    pub post_bump_seed: u8,
}
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, BorshSchema, PartialEq)]
pub struct PostVote {
    /* pub stake: u64, */
    pub vote: Vote,
    /*  pub mint_authority_bump_seed: u8,
    pub escrow_bump_seed: u8, */
}

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, BorshSchema, PartialEq)]
pub enum PostInstruction {
    // Create channel
    CreatePost(CreatePost),
    Vote(PostVote),
    Unvote(PostVote),
    ExecutePost,
    CreateRule {
        rule_id: Pubkey,
        vote_config: RuleVoteConfig,
        time_config: RuleTimeConfig,
        rule_bump_seed: u8,
    },
    CreateProposalOption(CreateProposalOptionType),
    InsertTransaction {
        option_index: u16,
        instruction_index: u16,
        hold_up_time: u32,
        instructions: Vec<ConditionedInstruction>,
    },
    DepositGoverningTokens {
        amount: u64,
    },
}

pub fn create_post_proposal(
    program_id: &Pubkey,
    payer: &Pubkey,
    channel: &Pubkey,
    hash: &[u8; 32],
    vote_type: VoteType,
    rules: Vec<Pubkey>,
    token_owner_record: &Pubkey,
    source: &ContentSource,
) -> Instruction {
    let (post_address, post_bump_seed) = find_post_program_address(program_id, hash);
    let accounts = vec![
        AccountMeta::new(post_address, false),
        AccountMeta::new(*channel, false),
        AccountMeta::new(*payer, true),
        AccountMeta::new_readonly(system_program::id(), false),
        AccountMeta::new_readonly(solana_program::sysvar::rent::id(), false),
        AccountMeta::new_readonly(spl_token::id(), false),
        // proposal specific
        AccountMeta::new_readonly(*token_owner_record, false),
    ];

    Instruction {
        program_id: *program_id,
        data: (PostInstruction::CreatePost(CreatePost {
            /*        creator: *user,
            channel: *channel,
            vote_mint_address: *vote_mint_address, */
            hash: *hash,
            post_type: CreatePostType::Proposal { vote_type },
            source: source.clone(),
            post_bump_seed,
            rules,
        }))
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
    let token_owner_record_address =
        get_token_owner_record_address(program_id, governing_token_mint, governing_token_owner);

    let governing_token_holding_address =
        find_realm_mint_program_address(program_id, payer, governing_token_mint).0;

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

    let instruction = PostInstruction::DepositGoverningTokens { amount };

    Instruction {
        program_id: *program_id,
        accounts,
        data: instruction.try_to_vec().unwrap(),
    }
}

pub fn insert_transaction(
    program_id: &Pubkey,
    payer: &Pubkey,
    post: &Pubkey,
    channel: &Pubkey,
    token_owner_record: &Pubkey,
    governance_authority_info: &Pubkey,
    option_index: u16,
    instruction_index: u16,
    hold_up_time: u32,
    instructions: Vec<ConditionedInstruction>,
) -> Instruction {
    let proposal_transaction_address = get_proposal_transaction_address(
        program_id,
        post,
        &option_index.to_le_bytes(),
        &instruction_index.to_le_bytes(),
    );
    let mut accounts = vec![
        AccountMeta::new_readonly(*channel, false),
        AccountMeta::new(*post, false),
        AccountMeta::new_readonly(*token_owner_record, false),
        AccountMeta::new_readonly(*governance_authority_info, true),
        AccountMeta::new(proposal_transaction_address, false),
        AccountMeta::new(*payer, false), //  voter token owner record
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

pub fn create_proposal_option(
    program_id: &Pubkey,
    payer: &Pubkey,
    post: &Pubkey,
    proposal_option_type: CreateProposalOptionType,
    option_index: u16,
) -> Instruction {
    let proposal_option_address =
        find_proposal_option_program_address(program_id, post, &option_index.to_le_bytes()).0;
    let accounts = vec![
        AccountMeta::new(proposal_option_address, false),
        AccountMeta::new(*post, false),
        AccountMeta::new(*payer, false), //  voter token owner record
        AccountMeta::new(system_program::id(), false),
        AccountMeta::new_readonly(solana_program::sysvar::rent::id(), false),
    ];
    Instruction {
        program_id: *program_id,
        data: PostInstruction::CreateProposalOption(proposal_option_type)
            .try_to_vec()
            .unwrap(),
        accounts,
    }
}

pub fn cast_vote(
    program_id: &Pubkey,
    payer: &Pubkey,
    post: &Pubkey,
    vote: Vote,
    governing_token_mint: &Pubkey,
    delegated_rule: Option<Pubkey>,
) -> Instruction {
    /// CAST  VOTE BY RULE?
    let record_address = match delegated_rule {
        Some(_rule) => {
            get_token_owner_delegatee_record_address(program_id, governing_token_mint, payer)
        }
        None => get_token_owner_record_address(program_id, governing_token_mint, payer),
    };

    let accounts = vec![
        AccountMeta::new(*post, false),
        AccountMeta::new(record_address, false),
        AccountMeta::new(*payer, false), //  voter token owner record
        AccountMeta::new(*payer, true),
        AccountMeta::new(system_program::id(), false),
        AccountMeta::new_readonly(spl_token::id(), false),
    ];

    Instruction {
        program_id: *program_id,
        data: (PostInstruction::Vote(PostVote {
            /*   mint_authority_bump_seed,
            stake,
            escrow_bump_seed, */
            vote,
        }))
        .try_to_vec()
        .unwrap(),
        accounts,
    }
}

pub fn uncast_vote(
    program_id: &Pubkey,
    payer: &Pubkey,
    post: &Pubkey,
    governence_mint: &Pubkey,
    _stake: u64,
    vote: Vote,
) -> Instruction {
    let (mint_upvote_address, _) = find_post_upvote_mint_program_address(program_id, post);
    let (mint_downvote_address, _) = find_post_downvote_mint_program_address(program_id, post);

    let payer_utility_token_address = get_associated_token_address(payer, governence_mint);

    let (mint_authority_address, _mint_authority_bump_seed) =
        find_post_mint_authority_program_address(program_id, post);

    /*  let associated_token_address = match vote {
        Vote::Up => get_associated_token_address(payer, &mint_upvote_address),
        Vote::Down => get_associated_token_address(payer, &mint_downvote_address),
    }; */

    let mut accounts = vec![
        AccountMeta::new(*payer, true),
        AccountMeta::new(payer_utility_token_address, false),
        AccountMeta::new(*post, false),
        AccountMeta::new(mint_upvote_address, false),
        AccountMeta::new(mint_downvote_address, false),
        AccountMeta::new(mint_authority_address, false),
        /* AccountMeta::new(associated_token_address, false), */
        AccountMeta::new_readonly(spl_token::id(), false),
    ];

    let (escrow_address, _escrow_bump_seed) = find_escrow_program_address(program_id, post);
    accounts.push(AccountMeta::new(escrow_address, false));
    Instruction {
        program_id: *program_id,
        data: (PostInstruction::Unvote(PostVote {
            /*  mint_authority_bump_seed,
            stake,
            escrow_bump_seed, */
            vote,
        }))
        .try_to_vec()
        .unwrap(),
        accounts,
    }
}

pub fn create_rule(
    program_id: &Pubkey,
    payer: &Pubkey,
    vote_config: &RuleVoteConfig,
    time_config: &RuleTimeConfig,
) -> Instruction {
    let id = Pubkey::new_unique();
    let (create_rule_address, create_rule_address_bump_seed) =
        find_create_rule_associated_program_address(program_id, &id);
    let accounts = vec![
        AccountMeta::new(create_rule_address, false),
        AccountMeta::new(*payer, true),
        AccountMeta::new(system_program::id(), false),
    ];

    Instruction {
        program_id: *program_id,
        data: (PostInstruction::CreateRule {
            rule_id: id,
            time_config: time_config.clone(),
            vote_config: vote_config.clone(),
            rule_bump_seed: create_rule_address_bump_seed,
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
pub fn create_post_execution_transaction(
    program_id: &Pubkey,
    _payer: &Pubkey,
    post: &Pubkey,
    post_account: &PostAccount,
    governence_mint: &Pubkey,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(*post, false),
        AccountMeta::new(post_account.channel, false),
        AccountMeta::new(*governence_mint, false),
    ];

    /* match &post_account.post_type {
           PostType::ActionPost(action) => match &action.action {
               Action::CustomEvent { event_type, .. } => {
                   accounts.push(AccountMeta::new(
                       find_create_rule_associated_program_address(
                           program_id,
                           &ActionType::CustomEvent(*event_type),
                           &post_account.channel,
                       )
                       .0,
                       false,
                   ));
               }
               Action::DeletePost(key) => {
                   accounts.push(AccountMeta::new(
                       find_create_rule_associated_program_address(
                           program_id,
                           &ActionType::DeletePost,
                           &post_account.channel,
                       )
                       .0,
                       false,
                   ));
                   accounts.push(AccountMeta::new(*key, false));
               }
               Action::ManageRule(update) => match update {
                   VotingRuleUpdate::Create { rule, bump_seed } => {
                       accounts.push(AccountMeta::new(
                           find_create_rule_associated_program_address(
                               program_id,
                               &ActionType::ManageRule(RuleUpdateType::Create),
                               &post_account.channel,
                           )
                           .0,
                           false,
                       ));

                       accounts.push(AccountMeta::new(*payer, true));
                       let (new_rule, new_rule_bump_seed) =
                           find_create_rule_associated_program_address(
                               program_id,
                               &rule.action,
                               &rule.channel,
                           );

                       if &new_rule_bump_seed != bump_seed {
                           panic!("Unexpected");
                       }

                       accounts.push(AccountMeta::new(new_rule, false));
                       accounts.push(AccountMeta::new_readonly(system_program::id(), false));
                   }
                   VotingRuleUpdate::Delete(key) => {
                       accounts.push(AccountMeta::new(
                           find_create_rule_associated_program_address(
                               program_id,
                               &ActionType::ManageRule(RuleUpdateType::Delete),
                               &post_account.channel,
                           )
                           .0,
                           false,
                       ));
                       accounts.push(AccountMeta::new(*key, false));
                   }
               },
               Action::Treasury(treasury_action) => match treasury_action {
                   TreasuryAction::Create { mint } => {
                       accounts.push(AccountMeta::new(
                           find_create_rule_associated_program_address(
                               program_id,
                               &ActionType::Treasury(TreasuryActionType::Create),
                               &post_account.channel,
                           )
                           .0,
                           false,
                       ));

                       accounts.push(AccountMeta::new(*payer, true));
                       accounts.push(AccountMeta::new(*mint, false));
                       let treasury_address = find_treasury_token_account_address(
                           &post_account.channel,
                           mint,
                           &spl_token::id(),
                           program_id,
                       )
                       .0;
                       accounts.push(AccountMeta::new(treasury_address, false));
                       accounts.push(AccountMeta::new(
                           find_authority_program_address(program_id, &treasury_address).0,
                           false,
                       ));
                       accounts.push(AccountMeta::new(system_program::id(), false));
                       accounts.push(AccountMeta::new_readonly(spl_token::id(), false));
                       accounts.push(AccountMeta::new_readonly(sysvar::rent::id(), false));
                   }
                   TreasuryAction::Transfer {
                       from,
                       to,
                       bump_seed,
                       ..
                   } => {
                       accounts.push(AccountMeta::new(
                           find_create_rule_associated_program_address(
                               program_id,
                               &ActionType::Treasury(TreasuryActionType::Transfer {
                                   from: Some(*from),
                                   to: Some(*to),
                               }),
                               &post_account.channel,
                           )
                           .0,
                           false,
                       ));
                       accounts.push(AccountMeta::new(*from, false));
                       accounts.push(AccountMeta::new(*to, false));
                       accounts.push(AccountMeta::new(
                           find_authority_program_address(program_id, from).0,
                           false,
                       ));

                       //   accounts.push(AccountMeta::new(, false));
                       accounts.push(AccountMeta::new_readonly(spl_token::id(), false));
                   }
               },
           },
           _ => {
               panic!("Unexpected");
           }
       };
    */
    Instruction {
        program_id: *program_id,
        data: (PostInstruction::ExecutePost).try_to_vec().unwrap(),
        accounts,
    }
}
