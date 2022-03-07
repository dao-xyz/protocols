use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};
use shared::content::ContentSource;
use solana_program::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    system_program, sysvar,
};
use spl_associated_token_account::get_associated_token_address;

use crate::state::{
    post::{PostAccount, PostType, VotingRuleUpdate},
    proposal::{
        proposal_option::{find_proposal_option_program_address, ProposalOptionType},
        VoteType,
    },
    rule_delegation_account::find_rule_delegation_account_program_address,
    rules::rule::{find_create_rule_associated_program_address, RuleTimeConfig, RuleVoteConfig},
    token_owner_record::{
        get_token_owner_delegatee_record_address, get_token_owner_record_address,
    },
};
use crate::tokens::spl_utils::find_authority_program_address;
use crate::{
    find_escrow_program_address, find_post_downvote_mint_program_address,
    find_post_mint_authority_program_address, find_post_program_address,
    find_post_upvote_mint_program_address, find_treasury_token_account_address, Vote,
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
}

pub fn create_post_proposal_transaction(
    program_id: &Pubkey,
    payer: &Pubkey,
    user: &Pubkey,
    channel: &Pubkey,
    hash: &[u8; 32],
    vote_type: VoteType,
    rules: Vec<Pubkey>,
    use_deny_option: bool,
    token_owner_record: &Pubkey,
    source: &ContentSource,
) -> Instruction {
    let (post_address, post_bump_seed) = find_post_program_address(program_id, &hash);

    /*   let (user_post_token_account, user_post_token_account_bump_seed) =
    find_user_post_token_program_address(program_id, &post_address, user); */
    let mut accounts = vec![
        AccountMeta::new(*payer, true),
        AccountMeta::new(*user, false),
        AccountMeta::new(post_address, false),
        AccountMeta::new(*channel, false),
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

// "Stake" solvei tokens
pub fn create_post_vote_transaction(
    program_id: &Pubkey,
    payer: &Pubkey,
    post: &Pubkey,
    vote: Vote,
    governing_token_mint: &Pubkey,
    rules: &[Pubkey],
    delegated_rule: Option<Pubkey>,
) -> Instruction {
    /*
      let post_account_info = next_account_info(accounts_iter)?;
        let proposal_transaction_info = next_account_info(accounts_iter)?;
        let vote_record_info = next_account_info(accounts_iter)?;
        let token_owner_record_info = next_account_info(accounts_iter)?;
        let voter_token_owner_record_info = next_account_info(accounts_iter)?;
        let payer_info = next_account_info(accounts_iter)?;
        let system_info = next_account_info(accounts_iter)?;

    */
    let record_address = match delegated_rule {
        Some(rule) => {
            get_token_owner_delegatee_record_address(program_id, governing_token_mint, payer)
        }
        None => get_token_owner_record_address(program_id, governing_token_mint, payer),
    };

    let mut accounts = vec![
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

// "Unstake" solvei tokens
pub fn create_post_unvote_transaction(
    program_id: &Pubkey,
    payer: &Pubkey,
    post: &Pubkey,
    governence_mint: &Pubkey,
    stake: u64,
    vote: Vote,
) -> Instruction {
    let (mint_upvote_address, _) = find_post_upvote_mint_program_address(program_id, post);
    let (mint_downvote_address, _) = find_post_downvote_mint_program_address(program_id, post);

    let payer_utility_token_address = get_associated_token_address(payer, governence_mint);

    let (mint_authority_address, mint_authority_bump_seed) =
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

    let (escrow_address, escrow_bump_seed) = find_escrow_program_address(program_id, post);
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

pub fn create_create_rule_transaction(
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
    payer: &Pubkey,
    post: &Pubkey,
    post_account: &PostAccount,
    governence_mint: &Pubkey,
) -> Instruction {
    let mut accounts = vec![
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
