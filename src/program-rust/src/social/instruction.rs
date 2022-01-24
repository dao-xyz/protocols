use std::io::Result;

use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};
use solana_program::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    system_program,
};
use spl_associated_token_account::get_associated_token_address;

use crate::{
    instruction::CHAT_INSTRUCTION_INDEX,
    shared::io_utils::try_to_vec_prepend,
    social::accounts::{ChannelAccount, Message, UserAccount},
    tokens::spl_utils::{
        find_mint_authority_program_address, find_mint_escrow_program_address,
        find_utility_mint_program_address,
    },
};

use super::{
    accounts::{AMMCurve, Content, Description, MarketMaker, PostAccount},
    find_channel_program_address, find_post_downvote_mint_program_address,
    find_post_mint_authority_program_address, find_post_program_address,
    find_post_upvote_mint_program_address, find_user_account_program_address,
    swap::{self, find_escrow_program_address, longshort::LongShortCurve},
    Vote,
};
/*
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, BorshSchema, PartialEq)]
pub struct SendMessage {
    pub user: Pubkey,
    pub channel: Pubkey,
    pub timestamp: u64,
    pub message: Message,
    pub bump_seed: u8,
}
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, BorshSchema, PartialEq)]
pub struct SubmitMessage {
    pub from: Pubkey,
}*/

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, BorshSchema, PartialEq)]
pub enum AMMCurveSwapSettings {
    Identity {
        escrow_bump_seed: u8,
    },
    OffsetPool {
        swap_bump_seed: u8,
        swap_authority_bump_seed: u8,
        swap_mint_bump_seed: u8,
        swap_fee_token_account_bump_seed: u8,
        token_utility_account_bump_seed: u8,
        token_upvote_account_bump_seed: u8,
        token_downvote_account_bump_seed: u8,
        vote_mint_authority_bump_seed: u8,
    },
    Offset {},
}

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, BorshSchema, PartialEq)]
pub enum MarketMakerSwapSettings {
    AMM(AMMCurveSwapSettings), // order book later
}

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, BorshSchema, PartialEq)]
pub struct OffsetCreateSettings {
    pub swap_bump_seed: u8,
    pub swap_authority_bump_seed: u8,
    pub swap_mint_bump_seed: u8,
    pub swap_fee_token_account_bump_seed: u8,
    pub swap_deposit_token_account_bump_seed: u8,
    pub token_utility_account_bump_seed: u8,
    pub token_target_account_bump_seed: u8,
}

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, BorshSchema, PartialEq)]
pub enum AMMCurveCreateSettings {
    Identity {
        escrow_bump_seed: u8,
    }, // 1 to 1 (risk "free"), unlimited supply
    OffsetLongShortPool {
        // Create two swap pools to simulate price actions for taking long and short positions of underlying assets
        offset: u64, // A common offset for both offset curves
        mint_authority_bump_seed: u8,
        long: OffsetCreateSettings,
        short: OffsetCreateSettings,
    },

    OffsetLongShort {
        // Create two swap pools to simulate price actions for taking long and short positions of underlying assets
        curve: LongShortCurve,
        utility_token_account_bump_seed: u8,
        long_token_account_bump_seed: u8,
        short_token_account_bump_seed: u8,
    },
}

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, BorshSchema, PartialEq)]
pub enum MarketMakerCreateSettings {
    AMM(AMMCurveCreateSettings), // order book later
}

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, BorshSchema, PartialEq)]
pub struct CreatePost {
    pub creator: Pubkey,
    pub channel: Pubkey,
    pub timestamp: u64,
    pub content: Content,
    pub market_maker: MarketMakerCreateSettings,
    pub post_bump_seed: u8,
    pub mint_upvote_bump_seed: u8,
    pub mint_downvote_bump_seed: u8,
    pub mint_authority_bump_seed: u8,
}

/* #[derive(Clone, Debug, BorshSerialize, BorshDeserialize, BorshSchema, PartialEq)]
pub struct CreatePostContent {
    pub message: Message,
    pub bump_seed: u8,
} */

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, BorshSchema, PartialEq)]
pub struct VotePost {
    pub post: Pubkey,
    pub stake: u64,
    pub mint_authority_bump_seed: u8,
    pub market_maker: MarketMakerSwapSettings,
    pub vote: Vote,
}

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, BorshSchema, PartialEq)]
pub enum ChatInstruction {
    // Message builder is user to build a message that later can be submitted with the submitt message instruction
    CreateUser(UserAccount),

    // Create channel
    CreateChannel(ChannelAccount),

    // Update channel (the tail message)
    UpdateChannel(ChannelAccount),

    // Message builder is user to build a message that later can be submitted with the submitt message instruction
    // SendMessage(SendMessage),
    // Add message to message builder
    //BuildMessagePart(String),

    // Submit message from BuildMessage invocations
    //SubmitMessage,
    CreatePost(CreatePost),

    /// Must be included in the same transaction as the CreatePost transaction
    //CreatePostContent(CreatePostContent),

    // "Like" or "Dislike" the post with an amount of Solvei tokens
    VotePost(VotePost),
    UnvotePost(VotePost),
}

impl ChatInstruction {
    /**
     * Prepends global instruction index
     */
    pub fn try_to_vec(&self) -> Result<Vec<u8>> {
        try_to_vec_prepend(CHAT_INSTRUCTION_INDEX, self)
    }
}

/// Creates a create user transction
pub fn create_user_transaction(program_id: &Pubkey, username: &str, payer: &Pubkey) -> Instruction {
    let (user_address_pda, _) = find_user_account_program_address(program_id, username);
    Instruction {
        program_id: *program_id,
        data: ChatInstruction::CreateUser(UserAccount {
            name: username.into(),
            owner: *payer,
        })
        .try_to_vec()
        .unwrap(),
        accounts: vec![
            AccountMeta::new(*payer, true),
            AccountMeta::new(user_address_pda, false),
            AccountMeta::new(system_program::id(), false),
        ],
    }
}

/// Creates a create user transction
pub fn create_channel_transaction(
    program_id: &Pubkey,
    channel_name: &str,
    payer: &Pubkey,
    user: &Pubkey,
) -> Instruction {
    let (channel, _) = find_channel_program_address(program_id, channel_name);

    Instruction {
        program_id: *program_id,
        data: ChatInstruction::CreateChannel(ChannelAccount {
            name: channel_name.into(),
            description: Description::String("This channel lets you channel channels".into()),
            owner: *payer,
        })
        .try_to_vec()
        .unwrap(),
        accounts: vec![
            AccountMeta::new(*payer, true),
            AccountMeta::new(*user, false),
            AccountMeta::new(channel, false),
            AccountMeta::new(system_program::id(), false),
        ],
    }
}

fn build_offset_curve_instruction(
    program_id: &Pubkey,
    post: &Pubkey,
    target_mint: &Pubkey,
    accounts: &mut Vec<AccountMeta>,
    vote: &Vote,
) -> OffsetCreateSettings {
    let (swap, swap_bump_seed) = swap::offset::find_swap_program_address(program_id, &post, vote);
    let (swap_authority, swap_authority_bump_seed) =
        swap::offset::find_swap_authority_program_address(&swap);
    let (swap_token_account_utility, swap_token_account_utility_bump_seed) =
        swap::offset::find_utility_account_program_address(program_id, &post, vote);

    let (swap_token_account_target, swap_token_account_target_bump_seed) =
        swap::offset::find_swap_token_account_program_address(program_id, &target_mint);
    let (swap_mint, swap_mint_bump_seed) =
        swap::offset::find_swap_mint_program_address(program_id, &swap);
    let (swap_fee_token_account, swap_fee_token_account_bump_seed) =
        swap::offset::find_swap_token_fee_account_program_address(program_id, &swap);
    let (swap_deposit_token_account, swap_deposit_token_account_bump_seed) =
        swap::offset::find_swap_token_deposit_account_program_address(program_id, &swap);

    accounts.push(AccountMeta::new(swap, false));
    accounts.push(AccountMeta::new_readonly(swap_authority, false));
    accounts.push(AccountMeta::new(swap_token_account_utility, false));
    accounts.push(AccountMeta::new(swap_token_account_target, false));
    accounts.push(AccountMeta::new(swap_mint, false));
    accounts.push(AccountMeta::new(swap_fee_token_account, false));
    accounts.push(AccountMeta::new(swap_deposit_token_account, false));

    OffsetCreateSettings {
        swap_bump_seed,
        swap_authority_bump_seed,
        swap_mint_bump_seed,
        swap_deposit_token_account_bump_seed,
        swap_fee_token_account_bump_seed,
        token_target_account_bump_seed: swap_token_account_target_bump_seed,
        token_utility_account_bump_seed: swap_token_account_utility_bump_seed,
    }
}

/// Creates a create post transction
pub fn create_post_transaction(
    program_id: &Pubkey,
    payer: &Pubkey,
    user: &Pubkey,
    channel: &Pubkey,
    timestamp: u64,
    content: &Content,
    market_maker: &MarketMaker,
) -> Instruction {
    let (post_address, post_bump_seed) = find_post_program_address(program_id, &content.hash);
    let (mint_upvote_address, mint_upvote_bump_seed) =
        find_post_upvote_mint_program_address(program_id, &post_address);

    let (mint_downvote_address, mint_downvote_bump_seed) =
        find_post_downvote_mint_program_address(program_id, &post_address);

    let (mint_authority_address, mint_authority_bump_seed) =
        find_post_mint_authority_program_address(program_id, &post_address);

    let (utility_mint_address, _) = find_utility_mint_program_address(program_id);
    /*   let (user_post_token_account, user_post_token_account_bump_seed) =
    find_user_post_token_program_address(program_id, &post_address, user); */
    let mut accounts = vec![
        AccountMeta::new(*payer, true),
        AccountMeta::new(*user, false),
        AccountMeta::new(post_address, false),
        AccountMeta::new(mint_upvote_address, false),
        AccountMeta::new(mint_downvote_address, false),
        AccountMeta::new(mint_authority_address, false),
        AccountMeta::new(utility_mint_address, false),
        AccountMeta::new_readonly(system_program::id(), false),
        AccountMeta::new_readonly(solana_program::sysvar::rent::id(), false),
        AccountMeta::new_readonly(spl_token::id(), false),
    ];

    // For offset SwapCurve we need aditional accounts
    let create_market_maker = match market_maker {
        MarketMaker::AMM(curve) => match curve {
            AMMCurve::Identity => MarketMakerCreateSettings::AMM({
                let (escrow_address, escrow_bump_seed) =
                    find_escrow_program_address(program_id, &post_address);
                accounts.push(AccountMeta::new(escrow_address, false));
                AMMCurveCreateSettings::Identity {
                    escrow_bump_seed: escrow_bump_seed,
                }
            }),
            AMMCurve::Offset(offset) => {
                /* let long = build_offset_curve_instruction(
                    program_id,
                    &post_address,
                    &mint_upvote_address,
                    &mut accounts,
                    &Vote::UP,
                );
                let short = build_offset_curve_instruction(
                    program_id,
                    &post_address,
                    &mint_downvote_address,
                    &mut accounts,
                    &Vote::DOWN,
                );
                accounts.push(AccountMeta::new_readonly(spl_token_swap::id(), false));
                MarketMakerCreateSettings::AMM(AMMCurveCreateSettings::OffsetLongShortPool {
                    offset: offset.offset,
                    mint_authority_bump_seed,
                    long,
                    short,
                }) */

                let (utility_token_account, utility_token_account_bump_seed) =
                    swap::longshort::find_post_mint_token_account(
                        program_id,
                        &post_address,
                        &utility_mint_address,
                    );

                let (long_token_account, long_token_account_bump_seed) =
                    swap::longshort::find_post_mint_token_account(
                        program_id,
                        &post_address,
                        &mint_upvote_address,
                    );
                let (short_token_account, short_token_account_bump_seed) =
                    swap::longshort::find_post_mint_token_account(
                        program_id,
                        &post_address,
                        &mint_downvote_address,
                    );

                accounts.push(AccountMeta::new(utility_token_account, false));
                accounts.push(AccountMeta::new(long_token_account, false));
                accounts.push(AccountMeta::new(short_token_account, false));

                MarketMakerCreateSettings::AMM(AMMCurveCreateSettings::OffsetLongShort {
                    curve: LongShortCurve {
                        token_b_offset: offset.offset,
                    },
                    long_token_account_bump_seed,
                    short_token_account_bump_seed,
                    utility_token_account_bump_seed,
                })
            }
        },
    };

    Instruction {
        program_id: *program_id,
        data: ChatInstruction::CreatePost(CreatePost {
            creator: *user,
            channel: *channel,
            mint_upvote_bump_seed,
            mint_downvote_bump_seed,
            mint_authority_bump_seed,
            market_maker: create_market_maker,
            timestamp,
            content: content.clone(),
            post_bump_seed,
        })
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
    post_account: &PostAccount,
    stake: u64,
    vote: Vote,
) -> Instruction {
    let (mint_address_upvote, _) = find_post_upvote_mint_program_address(program_id, post);
    let (mint_address_downvote, _) = find_post_downvote_mint_program_address(program_id, post);

    let payer_utility_token_address =
        get_associated_token_address(payer, &find_utility_mint_program_address(program_id).0);

    let (mint_authority_address, mint_authority_bump_seed) =
        find_post_mint_authority_program_address(program_id, post);

    let associated_token_address = match vote {
        Vote::UP => get_associated_token_address(payer, &mint_address_upvote),
        Vote::DOWN => get_associated_token_address(payer, &mint_address_downvote),
    };

    let mut accounts = vec![
        AccountMeta::new(*payer, true),
        AccountMeta::new(payer_utility_token_address, false),
        AccountMeta::new(*post, false),
        AccountMeta::new(mint_address_upvote, false),
        AccountMeta::new(mint_address_downvote, false),
        AccountMeta::new(mint_authority_address, false),
        AccountMeta::new(associated_token_address, false),
        AccountMeta::new(system_program::id(), false),
        AccountMeta::new(solana_program::sysvar::rent::id(), false),
        AccountMeta::new_readonly(spl_token::id(), false),
        AccountMeta::new_readonly(spl_associated_token_account::id(), false),
    ];

    let market_maker_settings = match &post_account.market_maker {
        MarketMaker::AMM(curve) => match curve {
            AMMCurve::Identity => {
                let (escrow_address, mint_escrow_bump_seed) =
                    find_escrow_program_address(program_id, post);

                accounts.push(AccountMeta::new(escrow_address, false));
                MarketMakerSwapSettings::AMM(AMMCurveSwapSettings::Identity {
                    escrow_bump_seed: mint_escrow_bump_seed,
                })
            }
            AMMCurve::Offset(offset) => {
                let vote = &Vote::UP;
                let (utility_mint_address, _) = find_utility_mint_program_address(program_id);
                let (swap, swap_bump_seed) =
                    swap::offset::find_swap_program_address(program_id, post, vote);
                let (swap_authority, swap_authority_bump_seed) =
                    swap::offset::find_swap_authority_program_address(&swap);
                let (swap_token_account_utility, swap_token_account_utility_bump_seed) =
                    swap::offset::find_utility_account_program_address(program_id, &post, vote);
                let (swap_token_account_upvote, swap_token_account_upvote_bump_seed) =
                    swap::offset::find_swap_token_account_program_address(
                        program_id,
                        &mint_address_upvote,
                    );
                let (swap_token_account_downvote, swap_token_account_downvote_bump_seed) =
                    swap::offset::find_swap_token_account_program_address(
                        program_id,
                        &mint_address_downvote,
                    );
                let (swap_mint, swap_mint_bump_seed) =
                    swap::offset::find_swap_mint_program_address(program_id, &swap);
                let (swap_fee_token_account, swap_fee_token_account_bump_seed) =
                    swap::offset::find_swap_token_fee_account_program_address(program_id, &swap);
                accounts.push(AccountMeta::new(swap, false));
                accounts.push(AccountMeta::new_readonly(swap_authority, false));
                accounts.push(AccountMeta::new(swap_token_account_utility, false));
                accounts.push(AccountMeta::new(swap_token_account_upvote, false));
                accounts.push(AccountMeta::new(swap_token_account_downvote, false));
                accounts.push(AccountMeta::new(swap_mint, false));
                accounts.push(AccountMeta::new(swap_fee_token_account, false));
                accounts.push(AccountMeta::new_readonly(spl_token_swap::id(), false));

                MarketMakerSwapSettings::AMM(AMMCurveSwapSettings::OffsetPool {
                    swap_bump_seed,
                    swap_authority_bump_seed,
                    swap_mint_bump_seed,
                    swap_fee_token_account_bump_seed,
                    token_upvote_account_bump_seed: swap_token_account_upvote_bump_seed,
                    token_downvote_account_bump_seed: swap_token_account_downvote_bump_seed,
                    token_utility_account_bump_seed: swap_token_account_utility_bump_seed,
                    vote_mint_authority_bump_seed: mint_authority_bump_seed,
                })
            }
        },
    };

    Instruction {
        program_id: *program_id,
        data: ChatInstruction::VotePost(VotePost {
            mint_authority_bump_seed,
            stake,
            post: *post,
            market_maker: market_maker_settings,
            vote,
        })
        .try_to_vec()
        .unwrap(),
        accounts: accounts,
    }
}

// "Unstake" solvei tokens
pub fn create_post_unvote_transaction(
    program_id: &Pubkey,
    payer: &Pubkey,
    post: &Pubkey,
    post_account: &PostAccount,
    stake: u64,
    vote: Vote,
) -> Instruction {
    let (mint_address_upvote, _) = find_post_upvote_mint_program_address(program_id, post);
    let (mint_address_downvote, _) = find_post_downvote_mint_program_address(program_id, post);

    let payer_utility_token_address =
        get_associated_token_address(payer, &find_utility_mint_program_address(program_id).0);

    let (mint_authority_address, mint_authority_bump_seed) =
        find_post_mint_authority_program_address(program_id, post);

    let associated_token_address = match vote {
        Vote::UP => get_associated_token_address(payer, &mint_address_upvote),
        Vote::DOWN => get_associated_token_address(payer, &mint_address_downvote),
    };

    let mut accounts = vec![
        AccountMeta::new(*payer, true),
        AccountMeta::new(payer_utility_token_address, false),
        AccountMeta::new(*post, false),
        AccountMeta::new(mint_address_upvote, false),
        AccountMeta::new(mint_address_downvote, false),
        AccountMeta::new(mint_authority_address, false),
        AccountMeta::new(associated_token_address, false),
        AccountMeta::new_readonly(spl_token::id(), false),
    ];

    let market_maker_settings = match &post_account.market_maker {
        MarketMaker::AMM(curve) => match curve {
            AMMCurve::Identity => {
                let (escrow_address, mint_escrow_bump_seed) =
                    find_escrow_program_address(program_id, post);
                accounts.push(AccountMeta::new(escrow_address, false));
                MarketMakerSwapSettings::AMM(AMMCurveSwapSettings::Identity {
                    escrow_bump_seed: mint_escrow_bump_seed,
                })
            }
            AMMCurve::Offset(offset) => {
                let (utility_mint_address, _) = find_utility_mint_program_address(program_id);
                let (swap, swap_bump_seed) =
                    swap::offset::find_swap_program_address(program_id, &post, &Vote::UP);
                let (swap_authority, swap_authority_bump_seed) =
                    swap::offset::find_swap_authority_program_address(&swap);
                let (swap_token_account_utility, swap_token_account_utility_bump_seed) =
                    swap::offset::find_swap_token_account_program_address(
                        program_id,
                        &utility_mint_address,
                    );
                let (swap_token_account_upvote, swap_token_account_upvote_bump_seed) =
                    swap::offset::find_swap_token_account_program_address(
                        program_id,
                        &mint_address_upvote,
                    );
                let (swap_token_account_downvote, swap_token_account_downvote_bump_seed) =
                    swap::offset::find_swap_token_account_program_address(
                        program_id,
                        &mint_address_downvote,
                    );
                let (swap_mint, swap_mint_bump_seed) =
                    swap::offset::find_swap_mint_program_address(program_id, &swap);
                let (swap_fee_token_account, swap_fee_token_account_bump_seed) =
                    swap::offset::find_swap_token_fee_account_program_address(program_id, &swap);
                accounts.push(AccountMeta::new(swap, false));
                accounts.push(AccountMeta::new_readonly(swap_authority, false));
                accounts.push(AccountMeta::new(swap_token_account_utility, false));
                accounts.push(AccountMeta::new(swap_token_account_upvote, false));
                accounts.push(AccountMeta::new(swap_token_account_downvote, false));
                accounts.push(AccountMeta::new(swap_mint, false));
                accounts.push(AccountMeta::new(swap_fee_token_account, false));
                accounts.push(AccountMeta::new_readonly(spl_token_swap::id(), false));

                MarketMakerSwapSettings::AMM(AMMCurveSwapSettings::OffsetPool {
                    swap_bump_seed,
                    swap_authority_bump_seed,
                    swap_mint_bump_seed,
                    swap_fee_token_account_bump_seed,
                    token_upvote_account_bump_seed: swap_token_account_upvote_bump_seed,
                    token_downvote_account_bump_seed: swap_token_account_downvote_bump_seed,
                    token_utility_account_bump_seed: swap_token_account_utility_bump_seed,
                    vote_mint_authority_bump_seed: mint_authority_bump_seed,
                })
            }
        },
    };

    Instruction {
        program_id: *program_id,
        data: ChatInstruction::UnvotePost(VotePost {
            mint_authority_bump_seed,
            stake,
            post: *post,
            market_maker: market_maker_settings,
            vote,
        })
        .try_to_vec()
        .unwrap(),
        accounts,
    }
}
