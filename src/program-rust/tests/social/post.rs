use solana_program::{
    hash::Hash,
    instruction::{AccountMeta, Instruction, InstructionError},
    program_error::ProgramError,
    program_pack::Pack,
    rent::Rent,
};

use solana_program_test::*;
use solana_sdk::{
    commitment_config::CommitmentLevel, pubkey::Pubkey, signature::Keypair, signer::Signer,
    transaction::Transaction,
};
use spl_associated_token_account::{create_associated_token_account, get_associated_token_address};
use westake::{
    social::{
        accounts::{
            deserialize_post_account, AMMCurve, Content, ContentSource, MarketMaker, OffsetCurve,
        },
        find_post_downvote_mint_program_address, find_post_program_address,
        find_post_upvote_mint_program_address, find_user_account_program_address,
        instruction::{
            create_post_transaction, create_post_unvote_transaction, create_post_vote_transaction,
        },
        swap::{self, find_escrow_program_address},
        Vote,
    },
    stake_pool::state::Fee,
    tokens::spl_utils::find_utility_mint_program_address,
};

use crate::utils::program_test;
use westake::id;

pub async fn get_token_balance(banks_client: &mut BanksClient, token: &Pubkey) -> u64 {
    let token_account = banks_client.get_account(*token).await.unwrap().unwrap();
    let account_info: spl_token::state::Account =
        spl_token::state::Account::unpack_from_slice(token_account.data.as_slice()).unwrap();
    account_info.amount
}

pub fn create_content() -> Content {
    Content {
        hash: Pubkey::new_unique().to_bytes(),
        source: ContentSource::External {
            url: "ipfs:xyz".into(),
        },
    }
}

async fn assert_token_balance(banks_client: &mut BanksClient, account: &Pubkey, amount: u64) {
    assert_eq!(
        spl_token::state::Account::unpack(
            &*banks_client
                .get_account_with_commitment(*account, CommitmentLevel::Finalized)
                .await
                .unwrap()
                .unwrap()
                .data
        )
        .unwrap()
        .amount,
        amount
    );
}
pub async fn ensure_utility_token_balance(
    banks_client: &mut BanksClient,
    payer: &Keypair,
    recent_blockhash: &Hash,
    expected_balance: u64,
) {
    let mut stake_pool_accounts = super::super::stake_pool::helpers::StakePoolAccounts::new();
    stake_pool_accounts.sol_deposit_fee = Fee {
        numerator: 0,
        denominator: 1,
    };

    stake_pool_accounts
        .initialize_stake_pool(banks_client, &payer, &recent_blockhash, 1)
        .await
        .unwrap();

    // Create token account to hold utility tokens
    let mut transaction = Transaction::new_with_payer(
        &[create_associated_token_account(
            &payer.pubkey(),
            &payer.pubkey(),
            &stake_pool_accounts.pool_mint,
        )],
        Some(&payer.pubkey()),
    );
    transaction.sign(&[payer], *recent_blockhash);
    banks_client.process_transaction(transaction).await.unwrap();

    let associated_token_address =
        get_associated_token_address(&payer.pubkey(), &stake_pool_accounts.pool_mint);

    // Make deposit to stake pool to create utility tokens
    assert!(stake_pool_accounts
        .deposit_sol(
            banks_client,
            &payer,
            &recent_blockhash,
            &associated_token_address,
            expected_balance,
            None,
        )
        .await
        .is_none());
    let associated_account = banks_client
        .get_account(associated_token_address)
        .await
        .expect("get_account")
        .expect("associated_account not none");
    assert_eq!(
        associated_account.data.len(),
        spl_token::state::Account::LEN
    );
    let balance = spl_token::state::Account::unpack(&*associated_account.data)
        .unwrap()
        .amount;

    assert_eq!(expected_balance, balance);
}

struct SocialAccounts {
    username: String,
    user: Pubkey,
    user_token_account: Pubkey,
    post: Pubkey,
    upvote_mint: Pubkey,
    upvote_token_account: Pubkey,
    downvote_mint: Pubkey,
    downvote_token_account: Pubkey,
    channel: Pubkey,
    content: Content,
    mm: MarketMaker,
}

impl SocialAccounts {
    pub fn new(payer: &Pubkey, curve: AMMCurve) -> Self {
        let username = "name";
        let content = create_content();
        let post = find_post_program_address(&id(), &content.hash).0;
        let upvote_mint = find_post_upvote_mint_program_address(&id(), &post).0;
        let downvote_mint = find_post_downvote_mint_program_address(&id(), &post).0;
        Self {
            username: username.into(),
            user: find_user_account_program_address(&id(), username).0,
            user_token_account: get_associated_token_address(
                payer,
                &find_utility_mint_program_address(&id()).0,
            ),
            post,
            content,
            channel: Pubkey::new_unique(),
            mm: MarketMaker::AMM(curve),
            upvote_mint,
            downvote_mint,
            upvote_token_account: get_associated_token_address(&payer, &upvote_mint),
            downvote_token_account: get_associated_token_address(&payer, &downvote_mint),
        }
    }
    pub async fn initialize(
        &self,
        banks_client: &mut BanksClient,
        payer: &Keypair,
        recent_blockhash: &Hash,
        utility_token_balance: u64,
    ) {
        ensure_utility_token_balance(
            banks_client,
            &payer,
            &recent_blockhash,
            utility_token_balance,
        )
        .await;

        let user = crate::utils::create_and_verify_user(
            banks_client,
            &payer,
            &recent_blockhash,
            self.username.as_ref(),
        )
        .await;
        assert_eq!(user, self.user);
        let mut transaction_post = Transaction::new_with_payer(
            &[create_post_transaction(
                &id(),
                &payer.pubkey(),
                &self.user,
                &self.channel,
                123,
                &self.content,
                &self.mm,
            )],
            Some(&payer.pubkey()),
        );
        transaction_post.sign(&[payer], *recent_blockhash);
        banks_client
            .process_transaction(transaction_post)
            .await
            .unwrap();
    }

    pub async fn vote(
        &self,
        banks_client: &mut BanksClient,
        payer: &Keypair,
        vote: Vote,
        amount: u64,
    ) {
        let post_account = deserialize_post_account(
            &*banks_client
                .get_account(self.post)
                .await
                .unwrap()
                .unwrap()
                .data,
        );

        let mut tx = match vote {
            Vote::UP => Transaction::new_with_payer(
                &[create_post_vote_transaction(
                    &id(),
                    &payer.pubkey(),
                    &self.post,
                    &post_account,
                    amount,
                    Vote::UP,
                )],
                Some(&payer.pubkey()),
            ),
            Vote::DOWN => Transaction::new_with_payer(
                &[create_post_vote_transaction(
                    &id(),
                    &payer.pubkey(),
                    &self.post,
                    &post_account,
                    amount,
                    Vote::DOWN,
                )],
                Some(&payer.pubkey()),
            ),
        };
        tx.sign(&[payer], banks_client.get_recent_blockhash().await.unwrap());
        banks_client.process_transaction(tx).await.unwrap();
    }

    pub async fn unvote(
        &self,
        banks_client: &mut BanksClient,
        payer: &Keypair,
        vote: Vote,
        amount: u64,
    ) {
        let post_account = deserialize_post_account(
            &*banks_client
                .get_account(self.post)
                .await
                .unwrap()
                .unwrap()
                .data,
        );

        let mut tx = match vote {
            Vote::UP => Transaction::new_with_payer(
                &[create_post_unvote_transaction(
                    &id(),
                    &payer.pubkey(),
                    &self.post,
                    &post_account,
                    amount,
                    Vote::UP,
                )],
                Some(&payer.pubkey()),
            ),
            Vote::DOWN => Transaction::new_with_payer(
                &[create_post_unvote_transaction(
                    &id(),
                    &payer.pubkey(),
                    &self.post,
                    &post_account,
                    amount,
                    Vote::DOWN,
                )],
                Some(&payer.pubkey()),
            ),
        };
        tx.sign(&[payer], banks_client.get_recent_blockhash().await.unwrap());
        banks_client.process_transaction(tx).await.unwrap();
    }
}

#[tokio::test]
async fn success_identity() {
    let program = program_test();

    let (mut banks_client, payer, recent_blockhash) = program.start().await;
    let utility_amount = 100000;
    let socials = SocialAccounts::new(&payer.pubkey(), AMMCurve::Identity);
    socials
        .initialize(&mut banks_client, &payer, &recent_blockhash, utility_amount)
        .await;

    let (escrow_account_info, _) = find_escrow_program_address(&id(), &socials.post);

    let rent = Rent::default();
    let stake = 1000;

    // Stake some
    socials
        .vote(&mut banks_client, &payer, Vote::UP, stake)
        .await;

    let escrow_account = banks_client
        .get_account(escrow_account_info)
        .await
        .unwrap()
        .unwrap();

    assert!(rent.is_exempt(escrow_account.lamports, escrow_account.data.len()));
    println!("ABC");
    assert_token_balance(
        &mut banks_client,
        &socials.user_token_account,
        utility_amount - stake,
    )
    .await;
    assert_token_balance(&mut banks_client, &socials.upvote_token_account, stake).await;
    assert_token_balance(&mut banks_client, &escrow_account_info, stake).await;
    println!("DEF");

    // Stake more
    socials
        .vote(&mut banks_client, &payer, Vote::UP, stake)
        .await;

    assert_token_balance(
        &mut banks_client,
        &socials.user_token_account,
        utility_amount - stake * 2,
    )
    .await;

    assert_token_balance(&mut banks_client, &socials.upvote_token_account, stake * 2).await;
    assert_token_balance(&mut banks_client, &escrow_account_info, stake * 2).await;

    // Unstake
    socials
        .unvote(&mut banks_client, &payer, Vote::UP, stake)
        .await;

    assert_token_balance(
        &mut banks_client,
        &socials.user_token_account,
        utility_amount - stake,
    )
    .await;
    assert_token_balance(&mut banks_client, &socials.upvote_token_account, stake).await;
    assert_token_balance(&mut banks_client, &escrow_account_info, stake).await;

    // Unstake, same amount (we should now 0 token accounts)
    socials
        .unvote(&mut banks_client, &payer, Vote::UP, stake)
        .await;

    assert_token_balance(
        &mut banks_client,
        &socials.user_token_account,
        utility_amount,
    )
    .await;
    assert_token_balance(&mut banks_client, &socials.upvote_token_account, 0).await;
    assert_token_balance(&mut banks_client, &escrow_account_info, 0).await;
}

#[tokio::test]
async fn success_offset() {
    let program = program_test();

    let (mut banks_client, payer, recent_blockhash) = program.start().await;

    let utility_amount = 100000;
    let initial_supply = 100000;
    let socials = SocialAccounts::new(
        &payer.pubkey(),
        AMMCurve::Offset(OffsetCurve {
            offset: initial_supply,
        }),
    );
    socials
        .initialize(&mut banks_client, &payer, &recent_blockhash, utility_amount)
        .await;

    let (upvote_source_account_info, _) =
        swap::offset::find_swap_token_account_program_address(&id(), &socials.upvote_mint);
    let (downvote_source_account_info, _) =
        swap::offset::find_swap_token_account_program_address(&id(), &socials.downvote_mint);
    let (utility_source_account_info, _) = find_escrow_program_address(&id(), &socials.post);

    let stake = 1000;
    /*
    // Stake some
    socials
        .vote(&mut banks_client, &payer, Vote::UP, stake)
        .await;

    assert_token_balance(
        &mut banks_client,
        &socials.user_token_account,
        utility_amount - stake,
    )
    .await;
    assert_token_balance(&mut banks_client, &upvote_source_account_info, 990100).await;
    assert_token_balance(&mut banks_client, &utility_source_account_info, stake).await;
    assert_token_balance(&mut banks_client, &socials.upvote_token_account, 9900).await;

    // Stake more
    socials
        .vote(&mut banks_client, &payer, Vote::UP, stake)
        .await;

    assert_token_balance(
        &mut banks_client,
        &socials.user_token_account,
        utility_amount - stake * 2,
    )
    .await;
    assert_token_balance(&mut banks_client, &upvote_source_account_info, 980394).await;
    assert_token_balance(&mut banks_client, &utility_source_account_info, stake * 2).await;
    assert_token_balance(&mut banks_client, &socials.upvote_token_account, 19606).await; */

    /*
    // Unstake
    socials
        .unvote(&mut banks_client, &payer, Vote::UP, stake)
        .await;

    socials
        .assert_post_upvote_token_balances(&mut banks_client, utility_amount - stake, stake, stake)
        .await;

    // Unstake, same amount (we should now 0 token accounts)
    socials
        .unvote(&mut banks_client, &payer, Vote::UP, stake)
        .await;

    socials
        .assert_post_upvote_token_balances(&mut banks_client, utility_amount, 0, 0)
        .await; */
}
