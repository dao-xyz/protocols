use solana_program::{
    account_info::{next_account_info, AccountInfo},
    borsh::{get_instance_packed_len, get_packed_len, try_from_slice_unchecked},
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    pubkey::Pubkey,
    rent::Rent,
    sysvar::Sysvar,
};
use spl_associated_token_account::create_associated_token_account;
use spl_token::instruction::burn;

use crate::social::{
    accounts::{deserialize_user_account, AccountContainer},
    create_post_mint_escrow_program_address_seeds,
    instruction::ChatInstruction,
};
use crate::{
    shared::account::{
        create_and_serialize_account_signed, create_and_serialize_account_signed_verify_with_bump,
    },
    social::accounts::deserialize_post_account,
    tokens::spl_utils::{create_program_token_account, spl_mint_to, token_transfer},
};

use super::{
    accounts::{AMMCurve, ChannelAccount, MarketMaker, PostAccount, UserAccount},
    create_channel_account_program_address_seeds, create_post_mint_authority_program_address_seeds,
    create_post_mint_program_account, create_user_account_program_address_seeds,
    instruction::{CreatePost, VotePost},
    Vote,
};

pub static NULL_KEY: Pubkey = Pubkey::new_from_array([0_u8; 32]);

pub static MESSAGE_TRANSACTION_MAX_SIZE: usize = 1200;

pub struct Processor {}
impl Processor {
    // Program entrypoint's implementation

    pub fn process_create_user(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        user: UserAccount,
    ) -> ProgramResult {
        let accounts_iter = &mut accounts.iter();
        let payer_account = next_account_info(accounts_iter)?;

        if user.name.is_empty() {
            return Err(ProgramError::InvalidArgument);
        }
        // check if leading or trailing spaces, if so name is invalid
        let mut chars = user.name.chars();
        if chars.next().unwrap().is_whitespace() || chars.last().unwrap_or('_').is_whitespace() {
            return Err(ProgramError::InvalidArgument);
        }

        if &user.owner != payer_account.key {
            return Err(ProgramError::IllegalOwner); // requires payer as owner (for now)
        }

        let user_acount_info = next_account_info(accounts_iter)?;
        let system_account = next_account_info(accounts_iter)?;

        let rent = Rent::get()?;
        let seeds = create_user_account_program_address_seeds(&user.name);
        let seed_slice = &seeds.iter().map(|x| &x[..]).collect::<Vec<&[u8]>>()[..];
        create_and_serialize_account_signed(
            payer_account,
            user_acount_info,
            &AccountContainer::UserAccount(user),
            seed_slice,
            program_id,
            system_account,
            &rent,
        )?;
        Ok(())
    }

    pub fn process_create_channel(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        channel: ChannelAccount,
    ) -> ProgramResult {
        let accounts_iter = &mut accounts.iter();
        let payer_account = next_account_info(accounts_iter)?;
        let user_account_info = next_account_info(accounts_iter)?;
        let user = deserialize_user_account(user_account_info.data.borrow().as_ref());
        if &user.owner != payer_account.key {
            return Err(ProgramError::IllegalOwner); // requires payer as owner (for now)
        }

        let channel_account_info = next_account_info(accounts_iter)?;
        let system_account = next_account_info(accounts_iter)?;

        let rent = Rent::get()?;
        let seeds = create_channel_account_program_address_seeds(&channel.name);
        let seed_slice = &seeds.iter().map(|x| &x[..]).collect::<Vec<&[u8]>>()[..];
        create_and_serialize_account_signed(
            payer_account,
            channel_account_info,
            &AccountContainer::ChannelAccount(channel),
            seed_slice,
            program_id,
            system_account,
            &rent,
        )?;
        Ok(())
    }

    pub fn process_create_post(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        post: CreatePost,
    ) -> ProgramResult {
        let accounts_iter = &mut accounts.iter();
        let payer_account = next_account_info(accounts_iter)?;
        let user_account_info = next_account_info(accounts_iter)?;
        let user = deserialize_user_account(user_account_info.data.borrow().as_ref());
        if &user.owner != payer_account.key {
            // Can not create a post for another user
            return Err(ProgramError::InvalidArgument);
        }

        let post_account_info = next_account_info(accounts_iter)?;
        let mint_upvote_account_info = next_account_info(accounts_iter)?;
        let mint_downvote_account_info = next_account_info(accounts_iter)?;
        let mint_authority_account_info = next_account_info(accounts_iter)?;
        let escrow_utility_token_account_info = next_account_info(accounts_iter)?;
        let utility_mint_account_info = next_account_info(accounts_iter)?;
        let system_account = next_account_info(accounts_iter)?;
        let rent_info = next_account_info(accounts_iter)?;
        let token_program_info = next_account_info(accounts_iter)?;

        let rent = Rent::get()?;

        let content_hash = post.content.hash.clone();
        create_and_serialize_account_signed_verify_with_bump(
            payer_account,
            post_account_info,
            &AccountContainer::PostAccount(PostAccount {
                channel: post.channel,
                content: post.content,
                market_maker: post.market_maker.clone(),
                timestamp: post.timestamp,
                creator: *user_account_info.key,
            }),
            &[&content_hash],
            program_id,
            system_account,
            &rent,
            post.post_bump_seed,
        )?;

        // Upvote tokens
        create_post_mint_program_account(
            post_account_info.key,
            Vote::UP,
            mint_upvote_account_info,
            post.mint_upvote_bump_seed,
            mint_authority_account_info,
            payer_account,
            rent_info,
            token_program_info,
            system_account,
            program_id,
        )?;

        // Downvote tokens
        create_post_mint_program_account(
            post_account_info.key,
            Vote::DOWN,
            mint_downvote_account_info,
            post.mint_downvote_bump_seed,
            mint_authority_account_info,
            payer_account,
            rent_info,
            token_program_info,
            system_account,
            program_id,
        )?;

        // create empty escrow account
        let escrow_bump_seeds = &[post.escrow_bump_seed];
        let escrow_account_seeds = create_post_mint_escrow_program_address_seeds(
            &post_account_info.key,
            escrow_bump_seeds,
        );
        let expected_escrow_address =
            Pubkey::create_program_address(&escrow_account_seeds, program_id).unwrap();

        if escrow_utility_token_account_info.key != &expected_escrow_address {
            msg!(
                "Create account with PDA: {:?} was requested while PDA: {:?} was expected",
                escrow_utility_token_account_info.key,
                expected_escrow_address
            );
            return Err(ProgramError::InvalidSeeds);
        }

        let escrow_bump_seeds = &[post.escrow_bump_seed];
        let escrow_seeds =
            create_post_mint_escrow_program_address_seeds(post_account_info.key, escrow_bump_seeds);

        create_program_token_account(
            escrow_utility_token_account_info,
            &escrow_seeds,
            utility_mint_account_info,
            mint_authority_account_info,
            payer_account,
            rent_info,
            token_program_info,
            system_account,
            program_id,
        )?;

        // MM
        match post.market_maker {
            MarketMaker::AMM(amm_curve) => match amm_curve {
                AMMCurve::Identity => {}
                AMMCurve::Offset(offset) => {
                    let curve = spl_token_swap::curve::base::SwapCurve {
                        curve_type: spl_token_swap::curve::base::CurveType::Offset,
                        calculator: Box::new(spl_token_swap::curve::offset::OffsetCurve {
                            token_b_offset: offset,
                        }),
                    };

                    let swap_account_info = next_account_info(accounts_iter)?;
                    let swap_authority_info = next_account_info(accounts_iter)?;
                    let token_a_account = next_account_info(accounts_iter)?;
                    let token_b_account = next_account_info(accounts_iter)?;
                    let swap_pool_mint = next_account_info(accounts_iter)?;
                    let swap_pool_token_account = next_account_info(accounts_iter)?;
                    let swap_initial_token_account = next_account_info(accounts_iter)?;

                    spl_token_swap::instruction::initialize(
                        program_id,
                        token_program_info.key,
                        swap_account_info.key,
                        swap_authority_info.key,
                        token_a_account.key,
                        token_b_account.key,
                        swap_pool_mint.key,
                        swap_pool_token_account.key,
                        swap_initial_token_account.key,
                        0, // bump seed
                        spl_token_swap::curve::fees::Fees {
                            // 0 fees for now
                            trade_fee_numerator: 0,
                            trade_fee_denominator: 1,
                            owner_trade_fee_numerator: 0,
                            owner_trade_fee_denominator: 1,
                            owner_withdraw_fee_numerator: 0,
                            owner_withdraw_fee_denominator: 1,
                            host_fee_numerator: 0,
                            host_fee_denominator: 1,
                        },
                        curve,
                    )?;
                }
            },
        }

        Ok(())
    }

    pub fn process_create_post_vote(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        stake: VotePost,
    ) -> ProgramResult {
        let accounts_iter = &mut accounts.iter();
        let payer_account = next_account_info(accounts_iter)?;
        let payer_utility_token_account = next_account_info(accounts_iter)?;
        let post_account_info = next_account_info(accounts_iter)?;
        let post_account = deserialize_post_account(post_account_info.data.borrow().as_ref());
        let mint_account_info = next_account_info(accounts_iter)?;
        let mint_authority_account_info = next_account_info(accounts_iter)?;
        let mint_associated_token_account = next_account_info(accounts_iter)?;
        let escrow_token_account_info = next_account_info(accounts_iter)?;
        let system_account = next_account_info(accounts_iter)?;
        let rent_info = next_account_info(accounts_iter)?;
        let token_program_info = next_account_info(accounts_iter)?;
        let spl_associated_token_acount_program_info = next_account_info(accounts_iter)?;

        // Verify escrow account is correct
        let escrow_bump_seeds = &[stake.mint_escrow_bump_seed];
        let escrow_account_seeds = create_post_mint_escrow_program_address_seeds(
            &post_account_info.key,
            escrow_bump_seeds,
        );
        msg!("A");

        let expected_escrow_address =
            Pubkey::create_program_address(&escrow_account_seeds, program_id).unwrap();

        if escrow_token_account_info.key != &expected_escrow_address {
            msg!(
                "Create account with PDA: {:?} was requested while PDA: {:?} was expected",
                escrow_token_account_info.key,
                expected_escrow_address
            );
            return Err(ProgramError::InvalidSeeds);
        }

        if mint_associated_token_account.data.borrow().is_empty() {
            // Unitialized token account
            // this will cost some sol, but we assume we don't have to mint tokens for this

            invoke(
                &create_associated_token_account(
                    payer_account.key,
                    payer_account.key,
                    mint_account_info.key,
                ),
                &[
                    payer_account.clone(),
                    mint_associated_token_account.clone(),
                    payer_account.clone(),
                    mint_account_info.clone(),
                    system_account.clone(),
                    token_program_info.clone(),
                    rent_info.clone(),
                    spl_associated_token_acount_program_info.clone(),
                ],
            )?;
        }
        match post_account.market_maker {
            MarketMaker::AMM(curve) => {
                //transfer_to(payer_account, mint_escrow_account_info, stake.stake)?;
                //spl_burn(solvei_associated_token_account,solvei_mint_info,solvei_mint_authority_info,create_spl)

                match curve {
                    AMMCurve::Identity => {
                        token_transfer(
                            token_program_info.clone(),
                            payer_utility_token_account.clone(),
                            escrow_token_account_info.clone(),
                            payer_account.clone(),
                            stake.stake,
                        )?;

                        // for some tokens (Upvotes or downvotes depending on the mint info)
                        spl_mint_to(
                            mint_associated_token_account,
                            mint_account_info,
                            mint_authority_account_info,
                            &create_post_mint_authority_program_address_seeds(
                                post_account_info.key,
                                &[stake.mint_authority_bump_seed],
                            ),
                            stake.stake,
                            program_id,
                        )?;
                    }
                    _ => panic!("Not supported"),
                }
            }
        }

        Ok(())
    }

    pub fn process_create_post_unvote(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        stake: VotePost,
    ) -> ProgramResult {
        let accounts_iter = &mut accounts.iter();
        let payer = next_account_info(accounts_iter)?;
        let payer_utility_token_account = next_account_info(accounts_iter)?;
        let post_account_info = next_account_info(accounts_iter)?;
        let post_account = deserialize_post_account(post_account_info.data.borrow().as_ref());
        let mint_account_info = next_account_info(accounts_iter)?;
        let mint_authority_account_info = next_account_info(accounts_iter)?;
        let mint_associated_token_account = next_account_info(accounts_iter)?;
        let escrow_token_account_info = next_account_info(accounts_iter)?;
        let token_program_info = next_account_info(accounts_iter)?;

        // Verify escrow account is correct
        let escrow_bump_seeds = &[stake.mint_escrow_bump_seed];
        let escrow_account_seeds = create_post_mint_escrow_program_address_seeds(
            &post_account_info.key,
            escrow_bump_seeds,
        );

        let expected_escrow_address =
            Pubkey::create_program_address(&escrow_account_seeds, program_id).unwrap();

        if escrow_token_account_info.key != &expected_escrow_address {
            msg!(
                "Create account with PDA: {:?} was requested while PDA: {:?} was expected",
                escrow_token_account_info.key,
                expected_escrow_address
            );
            return Err(ProgramError::InvalidSeeds);
        }

        match post_account.market_maker {
            MarketMaker::AMM(curve) => {
                //transfer_to(payer_account, mint_escrow_account_info, stake.stake)?;
                //spl_burn(solvei_associated_token_account,solvei_mint_info,solvei_mint_authority_info,create_spl)

                match curve {
                    AMMCurve::Identity => {
                        let bump_seeds = &[stake.mint_authority_bump_seed];
                        let seeds = create_post_mint_authority_program_address_seeds(
                            post_account_info.key,
                            bump_seeds,
                        );

                        invoke_signed(
                            &spl_token::instruction::transfer(
                                token_program_info.key,
                                escrow_token_account_info.key,
                                payer_utility_token_account.key,
                                &mint_authority_account_info.key,
                                &[],
                                stake.stake,
                            )?,
                            &[
                                escrow_token_account_info.clone(),
                                payer_utility_token_account.clone(),
                                mint_authority_account_info.clone(),
                                token_program_info.clone(),
                            ],
                            &[&seeds],
                        )?;

                        invoke(
                            &burn(
                                &token_program_info.key,
                                mint_associated_token_account.key,
                                mint_account_info.key,
                                &payer.key,
                                &[],
                                stake.stake,
                            )?,
                            &[
                                mint_associated_token_account.clone(),
                                mint_account_info.clone(),
                                payer.clone(),
                                token_program_info.clone(),
                            ],
                        )?;
                    }
                    _ => panic!("Not supported"),
                }
            }
        }

        Ok(())
    }

    pub fn process(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction: ChatInstruction,
    ) -> ProgramResult {
        // Iterating accounts is safer then indexing

        match instruction {
            ChatInstruction::CreateUser(user) => {
                msg!("Create user: {}", user.name);
                Self::process_create_user(program_id, accounts, user)
            }

            ChatInstruction::CreateChannel(channel) => {
                msg!("Create channel: {}", channel.name);
                Self::process_create_channel(program_id, accounts, channel)
            }

            ChatInstruction::UpdateChannel(_) => {
                /*  let channel_account_info = next_account_info(accounts_iter)?;

                // Don't allow channel name to be updated, since it would require us to resize the account size
                // This would also mean that the PDA would change!
                channel.serialize(&mut *channel_account_info.data.borrow_mut())? */
                Ok(())
            }

            ChatInstruction::CreatePost(post) => {
                msg!("Create post");
                Self::process_create_post(program_id, accounts, post)
            }
            /* ChatInstruction::CreatePostContent(content) => {
                msg!("Create post content");
                Self::process_create_post_content(program_id, accounts, content)
            } */
            ChatInstruction::VotePost(stake) => {
                //let token_account_info = next_account_info(accounts_iter)?;
                msg!("Create vote");
                Self::process_create_post_vote(program_id, accounts, stake)
            }

            ChatInstruction::UnvotePost(stake) => {
                //let token_account_info = next_account_info(accounts_iter)?;
                msg!("Create unvote");
                Self::process_create_post_unvote(program_id, accounts, stake)
            }
        }
    }
}
/*

fn token_mint_to<'a>(
    stake_pool: &Pubkey,
    token_program: AccountInfo<'a>,
    mint: AccountInfo<'a>,
    destination: AccountInfo<'a>,
    authority: AccountInfo<'a>,
    authority_type: &[u8],
    bump_seed: u8,
    amount: u64,
) -> Result<(), ProgramError> {
    let me_bytes = stake_pool.to_bytes();
    let authority_signature_seeds = [&me_bytes[..32], authority_type, &[bump_seed]];
    let signers = &[&authority_signature_seeds[..]];

    let ix = spl_token::instruction::mint_to(
        token_program.key,
        mint.key,
        destination.key,
        authority.key,
        &[],
        amount,
    )?;

    invoke_signed(&ix, &[mint, destination, authority, token_program], signers)
}
fn check_account_owner(
    account_info: &AccountInfo,
    program_id: &Pubkey,
) -> Result<(), ProgramError> {
    if *program_id != *account_info.owner {
        msg!(
            "Expected account to be owned by program {}, received {}",
            program_id,
            account_info.owner
        );
        Err(ProgramError::IncorrectProgramId)
    } else {
        Ok(())
    }
} */
/* ChatInstruction::InitializeToken(initialize) => {
    // initialize multisig owner mint with escrow
    let owner_account = next_account_info(accounts_iter)?;
    let escrow_account_info = next_account_info(accounts_iter)?;
    let mint_account_info = next_account_info(accounts_iter)?;
    //let multisig_account_info = next_account_info(accounts_iter)?;
    let mint_authority_account_info = next_account_info(accounts_iter)?;
    let owner_token_account = next_account_info(accounts_iter)?;
    let rent_info = next_account_info(accounts_iter)?;
    let token_program_info = next_account_info(accounts_iter)?;
    let rent = Rent::get()?;

    assert_is_signing_program_owner(owner_account, owner_token_account)?;
    let mint_authority_account_address =
        Pubkey::find_program_address(&[b"token_mint_authority"], program_id).0;
    if mint_authority_account_address != *mint_authority_account_info.key {
        return Err(ProgramError::InvalidAccountData);
    }
    /* create_payer_program_multisig_account(
        multisig_account_info,
        initialize.multisig_bump_seed,
        payer_account,
        owner_account,
        rent_info,
        token_program_info,
        program_account,
        system_account,
    )?; */

    create_program_account_mint_account(
        program_id,
        mint_account_info,
        initialize.mint_bump_seed,
        mint_authority_account_info,
        payer_account,
        rent_info,
        token_program_info,
        system_account,
        program_id,
    )?;

    // create empty escrow account
    let escrow_bump_seeds = &[initialize.escrow_bump_seed];
    let escrow_account_seeds =
        create_escrow_account_bump_seeds(&program_id, escrow_bump_seeds);
    let expected_escrow_address =
        Pubkey::create_program_address(&escrow_account_seeds, program_id).unwrap();

    if escrow_account_info.key != &expected_escrow_address {
        msg!(
            "Create account with PDA: {:?} was requested while PDA: {:?} was expected",
            escrow_account_info.key,
            expected_escrow_address
        );
        return Err(ProgramError::InvalidSeeds);
    }
    let minimum_balance_as_stake = rent.minimum_balance(0);
    let create_account_instruction = create_account(
        payer_account.key,
        escrow_account_info.key,
        minimum_balance_as_stake,
        0 as u64,
        program_id,
    );
    invoke_signed(
        &create_account_instruction,
        &[
            payer_account.clone(),
            escrow_account_info.clone(),
            system_account.clone(),
        ],
        &[&escrow_account_seeds],
    )?;
} */

/* ChatInstruction::SendMessage(send_message) => {
    // Initializes an account for us that lets us build an message
    let user_account_info = next_account_info(accounts_iter)?;
    let channel_account_info = next_account_info(accounts_iter)?;
    let message_account = MessageAccount::new(
        send_message.user,
        send_message.channel,
        send_message.timestamp,
        send_message.message,
    );
    let message_account_info = next_account_info(accounts_iter)?;
    let rent = Rent::get()?;
    let user = deserialize_user_account(user_account_info.data.borrow().as_ref());
    if &user.owner != payer_account.key {
        return Err(ProgramError::IllegalOwner); // requires payer as owner (for now)
    }
    create_and_serialize_account_signed_verify_with_bump(
        payer_account,
        message_account_info,
        &AccountContainer::MessageAccount(message_account),
        &[
            &user_account_info.key.to_bytes(),
            &channel_account_info.key.to_bytes(),
            &send_message.timestamp.to_le_bytes(),
        ],
        program_id,
        system_account,
        &rent,
        send_message.bump_seed,
    )?;
} */

/* ChatInstruction::SetupStakePool(initialize) => {
    let rent = Rent::get()?;
    let program_owner_info = next_account_info(accounts_iter)?;
    let program_owner_token_info = next_account_info(accounts_iter)?;
    let stake_pool_info = next_account_info(accounts_iter)?;
    let manager_info = next_account_info(accounts_iter)?;
    let staker_info = next_account_info(accounts_iter)?;
    let withdraw_authority_info = next_account_info(accounts_iter)?;
    let validator_list_info = next_account_info(accounts_iter)?;
    let reserve_stake_info = next_account_info(accounts_iter)?;
    let pool_mint_info = next_account_info(accounts_iter)?;
    let manager_fee_info = next_account_info(accounts_iter)?;
    let rent_info = next_account_info(accounts_iter)?;
    let token_program_info = next_account_info(accounts_iter)?;
    let stake_program_info = next_account_info(accounts_iter)?;

    assert_is_signing_program_owner(program_owner_info, program_owner_token_info)?;

    let max_validators = 10;
    create_pool_mint(
        stake_pool_info.key,
        pool_mint_info,
        initialize.pool_mint_bump_seed,
        withdraw_authority_info.key,
        payer_account,
        rent_info,
        token_program_info,
        system_account,
        program_id,
    )?;

    // Reserve stake account
    create_independent_reserve_stake_account(
        reserve_stake_info,
        initialize.reserve_stake_bump_seed,
        stake_pool_info.key,
        payer_account,
        withdraw_authority_info.key,
        stake_program_info,
        rent_info,
        program_id,
    )?;

    // Manager fee account
    create_program_associated_token_account(
        manager_fee_info,
        initialize.manager_fee_account_bump_seed,
        pool_mint_info,
        payer_account,
        manager_info,
        rent_info,
        token_program_info,
        system_account,
        program_id,
    )?;
}

ChatInstruction::InitializeStakePool(initialize) => {
    let rent = Rent::get()?;
    let program_owner_info = next_account_info(accounts_iter)?;
    let program_owner_token_info = next_account_info(accounts_iter)?;
    let stake_pool_info = next_account_info(accounts_iter)?;
    let manager_info = next_account_info(accounts_iter)?;
    let staker_info = next_account_info(accounts_iter)?;
    let withdraw_authority_info = next_account_info(accounts_iter)?;
    let validator_list_info = next_account_info(accounts_iter)?;
    let reserve_stake_info = next_account_info(accounts_iter)?;
    let pool_mint_info = next_account_info(accounts_iter)?;
    let manager_fee_info = next_account_info(accounts_iter)?;
    let rent_info = next_account_info(accounts_iter)?;
    let token_program_info = next_account_info(accounts_iter)?;
    /* let ss = next_account_info(accounts_iter)?;
     */
    let ss2 = next_account_info(accounts_iter)?;

    let max_validators = 10; */
/* assert_is_signing_program_owner(program_owner_info, program_owner_token_info)?;


create_pool_mint(
    stake_pool_info.key,
    pool_mint_info,
    initialize.pool_mint_bump_seed,
    withdraw_authority_info.key,
    payer_account,
    rent_info,
    token_program_info,
    system_account,
    program_id,
)?;

// Reserve stake account
create_independent_reserve_stake_account(
    reserve_stake_info,
    initialize.reserve_stake_bump_seed,
    stake_pool_info.key,
    payer_account,
    withdraw_authority_info.key,
    stake_program_info,
    rent_info,
    program_id,
)?;

// Manager fee account
create_program_associated_token_account(
    manager_fee_info,
    initialize.manager_fee_account_bump_seed,
    pool_mint_info,
    payer_account,
    manager_info,
    rent_info,
    token_program_info,
    system_account,
    program_id,
)?;
*/
/* let manager_account_bump_seeds = [
    "manager".as_bytes(),
    &stake_pool_info.key.to_bytes(),
    &[initialize.manager_bump_seed],
];
let manager_account_address =
    Pubkey::create_program_address(&manager_account_bump_seeds, program_id).unwrap();

if manager_info.key != &manager_account_address {
    msg!(
        "Create account with PDA: {:?} was requested while PDA: {:?} was expected",
        manager_info.key,
        manager_account_address
    );
    return Err(ProgramError::InvalidSeeds);
} */

// -------- create stake pool ----------
// Validator list account
/* let empty_validator_list = ValidatorList::new(max_validators);
let validator_list_size = get_instance_packed_len(&empty_validator_list)?;
invoke_signed(
    &create_account(
        payer_account.key,
        validator_list_info.key,
        rent.minimum_balance(validator_list_size),
        validator_list_size as u64,
        &stake_pool::id(),
    ),
    &[payer_account.clone(), validator_list_info.clone()],
    &[&[
        "validator_list".as_bytes(),
        &stake_pool_info.key.to_bytes(),
        &[initialize.validator_list_bump_seed],
    ]],
)?;

// Stake ppol account
let xs = ["stake_pool".as_bytes(), &[initialize.stake_pool_bump_seed]];
let x = Pubkey::create_program_address(&xs, &program_id).unwrap();

if stake_pool_info.key != &x {
    msg!(
        "Create account with PDA: {:?} was requested while PDA: {:?} was expected",
        stake_pool_info.key,
        x
    );
    return Err(ProgramError::InvalidSeeds);
}
invoke_signed(
    &create_account(
        payer_account.key,
        stake_pool_info.key,
        rent.minimum_balance(initialize.stake_pool_packed_len as usize), // assume 8 bytes
        initialize.stake_pool_packed_len,
        /* rent.minimum_balance(get_packed_len::<stake_pool::state::StakePool>()),
        get_packed_len::<stake_pool::state::StakePool>() as u64, */
        &stake_pool::id(),
    ),
    &[payer_account.clone(), stake_pool_info.clone()],
    &[&["stake_pool".as_bytes(), &[initialize.stake_pool_bump_seed]]],
)?;

msg!("INIT");
msg!("???");
let instruction = stake_pool::instruction::initialize(
    &stake_pool::id(),
    stake_pool_info.key,
    manager_info.key,
    staker_info.key,
    validator_list_info.key,
    reserve_stake_info.key,
    pool_mint_info.key,
    manager_fee_info.key,
    token_program_info.key,
    None,
    stake_pool::state::Fee {
        numerator: 5, // 5%
        denominator: 100,
    },
    stake_pool::state::Fee {
        numerator: 0,
        denominator: 1,
    },
    stake_pool::state::Fee {
        numerator: 0,
        denominator: 1,
    },
    0,
    10,
);

invoke(
    &instruction,
    &[
        stake_pool_info.clone(),
        manager_info.clone(),
        staker_info.clone(),
        withdraw_authority_info.clone(),
        validator_list_info.clone(),
        reserve_stake_info.clone(),
        pool_mint_info.clone(),
        manager_fee_info.clone(),
        token_program_info.clone(),
        ss2.clone(),
        rent_info.clone(),
        program_account.clone(),
        system_account.clone(),
        payer_account.clone(),
    ],
)?; */

/* rent_info.clone(),
program_account.clone(),
system_account.clone(), */
/* process_initialize(
    &stake_pool::id(),
    &[
        stake_pool_info.clone(),
        manager_info.clone(),
        staker_info.clone(),
        withdraw_authority_info.clone(),
        validator_list_info.clone(),
        reserve_stake_info.clone(),
        pool_mint_info.clone(),
        manager_fee_info.clone(),
        token_program_info.clone(),
    ],
    stake_pool::state::Fee {
        numerator: 5, // 5%
        denominator: 100,
    },
    stake_pool::state::Fee {
        numerator: 0,
        denominator: 1,
    },
    stake_pool::state::Fee {
        numerator: 0,
        denominator: 1,
    },
    0,
    10,
)?; */
/*  } */

/*
 instruction::set_fee(
    &id(),
    &stake_pool.pubkey(),
    &manager.pubkey(),
    FeeType::SolDeposit(*sol_deposit_fee),
),
instruction::set_fee(
    &id(),
    &stake_pool.pubkey(),
    &manager.pubkey(),
    FeeType::SolReferral(sol_referral_fee),
),
*/

/*  invoke_signed(
    &stake_pool::instruction::set_fee(
        &stake_pool::id(),
        &stake_pool_info.key,
        &manager_info.key,
        stake_pool::state::FeeType::SolDeposit(*sol_deposit_fee),
    ),
    &[
        stake_pool_info.clone(),
        manager_info.clone(),
        staker_info.clone(),
        validator_list_info.clone(),
        reserve_stake_info.clone(),
        pool_mint_info.clone(),
        manager_fee_info.clone(),
        token_program_info.clone(),
    ],
    &[&manager_account_bump_seeds],
)?; */
