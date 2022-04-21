pub mod accounts;
pub mod entrypoint;
pub mod error;
pub mod instruction;
pub mod processor;
pub mod shared;
pub mod state;
solana_program::declare_id!("8jZtsr1pruCDNZeMRMMeS21NDJYj4dUmxJJ1J2jzi9Wa");
use state::vote_record::Vote;

/*
#[derive(Copy, Clone, Debug, BorshSerialize, BorshDeserialize, BorshSchema, PartialEq)]
pub enum Vote {
    Up = 0,
    Down = 1,
} */

/* pub fn find_escrow_program_address(program_id: &Pubkey, post: &Pubkey) -> (Pubkey, u8) {
    find_mint_escrow_program_address(program_id, post)
}

pub fn create_escrow_program_address_seeds<'a>(
    post: &'a Pubkey,
    bump_seed: &'a [u8],
) -> [&'a [u8]; 3] {
    create_mint_escrow_program_address_seeds(post, bump_seed)
} */
/*
pub fn create_post_mint_program_account<'a>(
    post: &Pubkey,
    vote: Vote,
    mint_info: &AccountInfo<'a>,
    mint_bump_seed: u8,
    mint_authority_info: &AccountInfo<'a>,
    payer_info: &AccountInfo<'a>,
    rent_info: &AccountInfo<'a>,
    token_program_info: &AccountInfo<'a>,
    system_info: &AccountInfo<'a>,
    program_id: &Pubkey,
) -> ProgramResult {
    let rent = Rent::get()?;
    let mint_rent = rent.minimum_balance(Mint::LEN);
    let decimals = spl_token::native_mint::DECIMALS; // for now

    let mint_bump_seed = &[mint_bump_seed];
    let mint_account_seeds = match vote {
        Vote::Up => create_post_upvote_mint_program_address_seeds(post, mint_bump_seed),
        Vote::Down => create_post_downvote_mint_program_address_seeds(post, mint_bump_seed),
    };

    let address = Pubkey::create_program_address(&mint_account_seeds, program_id).unwrap();
    if mint_info.key != &address {
        msg!(
            "Create account with PDA: {:?} was requested while PDA: {:?} was expected",
            mint_info.key,
            address
        );
        return Err(ProgramError::InvalidSeeds);
    }
    invoke_signed(
        &system_instruction::create_account(
            payer_info.key,
            mint_info.key,
            mint_rent,
            Mint::LEN as u64,
            &spl_token::id(),
        ),
        &[
            payer_info.clone(),
            mint_info.clone(),
            system_info.clone(),
            token_program_info.clone(),
        ],
        &[&mint_account_seeds],
    )?;

    invoke(
        &initialize_mint(
            &spl_token::id(),
            mint_info.key,
            mint_authority_info.key,
            None,
            decimals,
        )?,
        &[mint_info.clone(), rent_info.clone()],
    )?;
    Ok(())
}
 */
/* pub fn find_post_program_address(program_id: &Pubkey, hash: &[u8; 32]) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[hash], program_id)
}

/// Find address for the token upvote mint for the post account
pub fn find_post_upvote_mint_program_address(program_id: &Pubkey, post: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[MINT_SEED, UPVOTE, &post.to_bytes()], program_id)
} */
/*
/// Create post mint upvote program address
pub fn create_post_upvote_mint_program_address_seeds<'a>(
    post: &'a Pubkey,
    bump_seed: &'a [u8],
) -> [&'a [u8]; 4] {
    [MINT_SEED, UPVOTE, post.as_ref(), bump_seed]
}

/// Find post stats account address
pub fn find_post_stats_program_address(program_id: &Pubkey, post: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[STATS, &post.to_bytes()], program_id)
}

/// Create post stats acount address
pub fn create_post_stats_program_address_seeds<'a>(
    post: &'a Pubkey,
    bump_seed: &'a [u8],
) -> [&'a [u8]; 3] {
    [STATS, post.as_ref(), bump_seed]
}

/// Find address for the token downvote mint for the post account
pub fn find_post_downvote_mint_program_address(program_id: &Pubkey, post: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[MINT_SEED, DOWNVOTE, &post.to_bytes()], program_id)
}

/// Create post mint downvote program address
pub fn create_post_downvote_mint_program_address_seeds<'a>(
    post: &'a Pubkey,
    bump_seed: &'a [u8],
) -> [&'a [u8]; 4] {
    [MINT_SEED, DOWNVOTE, post.as_ref(), bump_seed]
} */

/*
/// Find rule account address
pub fn find_create_rule_associated_program_address(
    program_id: &Pubkey,
    action_type: &ActionType,
    channel: &Pubkey,
) -> (Pubkey, u8) {
    match action_type {
        ActionType::DeletePost => {
            Pubkey::find_program_address(&[RULE, b"delete", channel.as_ref()], program_id)
        }
        ActionType::CustomEvent(event_type) => {
            Pubkey::find_program_address(&[RULE, event_type.as_ref(), channel.as_ref()], program_id)
        }
        ActionType::ManageRule(manage_rule) => match manage_rule {
            RuleUpdateType::Create => {
                Pubkey::find_program_address(&[RULE, b"rule_create", channel.as_ref()], program_id)
            }
            RuleUpdateType::Delete => {
                Pubkey::find_program_address(&[RULE, b"rule_delete", channel.as_ref()], program_id)
            }
        },
        ActionType::Treasury(treasury_action) => match treasury_action {
            TreasuryActionType::Transfer { from, to } => Pubkey::find_program_address(
                &[
                    from.as_ref().map_or(b"treasury_from", |key| key.as_ref()),
                    to.as_ref().map_or(b"treasury_to", |key| key.as_ref()),
                    channel.as_ref(),
                ],
                program_id,
            ),
            TreasuryActionType::Create => Pubkey::find_program_address(
                &[RULE, b"treasury_create", channel.as_ref()],
                program_id,
            ),
        },
    }
}

/// Create rule account address
pub fn create_rule_associated_program_address_seeds<'a>(
    channel: &'a Pubkey,
    action_type: &'a ActionType,
    bump_seed: &'a [u8],
) -> [&'a [u8]; 4] {
    match action_type {
        ActionType::CustomEvent(key) => [RULE, key.as_ref(), channel.as_ref(), bump_seed],
        ActionType::DeletePost => [RULE, b"delete", channel.as_ref(), bump_seed],
        ActionType::ManageRule(manage_rule) => match manage_rule {
            RuleUpdateType::Create => [RULE, b"rule_create", channel.as_ref(), bump_seed],
            RuleUpdateType::Delete => [RULE, b"rule_delete", channel.as_ref(), bump_seed],
        },
        ActionType::Treasury(treasury_action) => match treasury_action {
            TreasuryActionType::Transfer { from, to } => [
                from.as_ref().map_or(b"treasury_from", |key| key.as_ref()),
                to.as_ref().map_or(b"treasury_to", |key| key.as_ref()),
                channel.as_ref(),
                bump_seed,
            ],
            TreasuryActionType::Create => [RULE, b"treasury_create", channel.as_ref(), bump_seed],
        },
    }
} */
/*
/// Find address for the token mint authority for the post account
pub fn find_post_mint_authority_program_address(
    program_id: &Pubkey,
    post: &Pubkey,
) -> (Pubkey, u8) {
    find_authority_program_address(program_id, post)
}

/// Create post mint authority program address
pub fn create_post_mint_authority_program_address_seeds<'a>(
    post: &'a Pubkey,
    bump_seed: &'a [u8],
) -> [&'a [u8]; 3] {
    create_authority_program_address_seeds(post, bump_seed)
}

 */
