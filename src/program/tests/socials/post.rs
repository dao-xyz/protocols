use solana_program::{hash::Hash, program_pack::Pack, rent::Rent};

use s2g::{
    socials::{
        find_user_account_program_address,
        post::{
            find_escrow_program_address, find_post_downvote_mint_program_address,
            find_post_program_address, find_post_upvote_mint_program_address,
            instruction::{
                create_post_transaction, create_post_unvote_transaction,
                create_post_vote_transaction,
            },
            state::{deserialize_post_account, ContentSource, PostType},
            Vote,
        },
    },
    stake_pool::state::Fee,
    tokens::spl_utils::find_utility_mint_program_address,
};
use solana_program_test::*;
use solana_sdk::{
    commitment_config::CommitmentLevel, pubkey::Pubkey, signature::Keypair, signer::Signer,
    transaction::Transaction,
};
use spl_associated_token_account::{create_associated_token_account, get_associated_token_address};

use crate::{
    socials::{channel::create_and_verify_channel, user::create_and_verify_user},
    utils::program_test,
};

pub async fn get_token_balance(banks_client: &mut BanksClient, token: &Pubkey) -> u64 {
    let token_account = banks_client.get_account(*token).await.unwrap().unwrap();
    let account_info: spl_token::state::Account =
        spl_token::state::Account::unpack_from_slice(token_account.data.as_slice()).unwrap();
    account_info.amount
}

pub fn create_content() -> ([u8; 32], ContentSource) {
    (
        Pubkey::new_unique().to_bytes(),
        ContentSource::External {
            url: "ipfs:xyz".into(),
        },
    )
}

async fn assert_token_balance(banks_client: &mut BanksClient, account: &Pubkey, amount: u64) {
    banks_client.get_latest_blockhash().await.unwrap();
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
        .initialize_stake_pool(banks_client, payer, recent_blockhash, 1)
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
            payer,
            recent_blockhash,
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
    hash: [u8; 32],
    source: ContentSource,
    governence_mint: Pubkey,
}

impl SocialAccounts {
    pub fn new(payer: &Pubkey) -> Self {
        let username = "name";
        let (hash, source) = create_content();
        let post = find_post_program_address(&s2g::id(), &hash).0;
        let upvote_mint = find_post_upvote_mint_program_address(&s2g::id(), &post).0;
        let downvote_mint = find_post_downvote_mint_program_address(&s2g::id(), &post).0;
        let (utility_mint_address, __bump) = find_utility_mint_program_address(&s2g::id());

        Self {
            username: username.into(),
            user: find_user_account_program_address(&s2g::id(), username).0,
            user_token_account: get_associated_token_address(
                payer,
                &find_utility_mint_program_address(&s2g::id()).0,
            ),
            post,
            hash,
            source,
            channel: Pubkey::new_unique(),
            upvote_mint,
            downvote_mint,
            upvote_token_account: get_associated_token_address(payer, &upvote_mint),
            downvote_token_account: get_associated_token_address(payer, &downvote_mint),
            governence_mint: utility_mint_address,
        }
    }
    pub async fn initialize(
        &self,
        banks_client: &mut BanksClient,
        payer: &Keypair,
        recent_blockhash: &Hash,
        utility_token_balance: u64,
    ) {
        ensure_utility_token_balance(banks_client, payer, recent_blockhash, utility_token_balance)
            .await;

        let user = create_and_verify_user(
            banks_client,
            payer,
            recent_blockhash,
            self.username.as_ref(),
            "profile",
        )
        .await;

        let channel = create_and_verify_channel(
            banks_client,
            payer,
            recent_blockhash,
            "Channel",
            &user,
            &self.governence_mint,
            None,
        )
        .await
        .unwrap();

        assert_eq!(user, self.user);
    }
    pub async fn assert_vote(&self, banks_client: &mut BanksClient, upvotes: u64, downvotes: u64) {
        let post = deserialize_post_account(
            &*banks_client
                .get_account(self.post)
                .await
                .unwrap()
                .unwrap()
                .data,
        )
        .unwrap();
        match post.post_type {
            PostType::SimplePost(s) => {
                assert_eq!(s.upvotes, upvotes);
                assert_eq!(s.downvotes, downvotes);
            }
        };
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
        )
        .unwrap();

        let mut tx = match vote {
            Vote::UP => Transaction::new_with_payer(
                &[create_post_vote_transaction(
                    &s2g::id(),
                    &payer.pubkey(),
                    &self.post,
                    amount,
                    Vote::UP,
                )],
                Some(&payer.pubkey()),
            ),
            Vote::DOWN => Transaction::new_with_payer(
                &[create_post_vote_transaction(
                    &s2g::id(),
                    &payer.pubkey(),
                    &self.post,
                    amount,
                    Vote::DOWN,
                )],
                Some(&payer.pubkey()),
            ),
        };
        tx.sign(&[payer], banks_client.get_latest_blockhash().await.unwrap());
        banks_client.process_transaction(tx).await.unwrap();
    }

    pub async fn unvote(
        &self,
        banks_client: &mut BanksClient,
        payer: &Keypair,
        vote: Vote,
        amount: u64,
    ) {
        let mut tx = match vote {
            Vote::UP => Transaction::new_with_payer(
                &[create_post_unvote_transaction(
                    &s2g::id(),
                    &payer.pubkey(),
                    &self.post,
                    amount,
                    Vote::UP,
                )],
                Some(&payer.pubkey()),
            ),
            Vote::DOWN => Transaction::new_with_payer(
                &[create_post_unvote_transaction(
                    &s2g::id(),
                    &payer.pubkey(),
                    &self.post,
                    amount,
                    Vote::DOWN,
                )],
                Some(&payer.pubkey()),
            ),
        };
        tx.sign(&[payer], banks_client.get_latest_blockhash().await.unwrap());
        banks_client.process_transaction(tx).await.unwrap();
    }
}

#[tokio::test]
async fn success_upvote() {
    let program = program_test();

    let (mut banks_client, payer, recent_blockhash) = program.start().await;
    let utility_amount = 100000;
    let socials = SocialAccounts::new(&payer.pubkey());
    socials
        .initialize(&mut banks_client, &payer, &recent_blockhash, utility_amount)
        .await;
    let mut transaction_post = Transaction::new_with_payer(
        &[create_post_transaction(
            &s2g::id(),
            &payer.pubkey(),
            &socials.user,
            &socials.channel,
            &socials.governence_mint,
            &socials.hash,
            &socials.source,
        )],
        Some(&payer.pubkey()),
    );
    transaction_post.sign(&[&payer], recent_blockhash);
    banks_client
        .process_transaction(transaction_post)
        .await
        .unwrap();

    let (escrow_account_info, _) = find_escrow_program_address(&s2g::id(), &socials.post);

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

    assert_token_balance(
        &mut banks_client,
        &socials.user_token_account,
        utility_amount - stake,
    )
    .await;
    assert_token_balance(&mut banks_client, &socials.upvote_token_account, stake).await;
    assert_token_balance(&mut banks_client, &escrow_account_info, stake).await;
    socials.assert_vote(&mut banks_client, stake, 0).await;

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
    socials.assert_vote(&mut banks_client, stake * 2, 0).await;

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
    socials.assert_vote(&mut banks_client, stake, 0).await;

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
    socials.assert_vote(&mut banks_client, 0, 0).await;
}

#[tokio::test]
async fn success_downvote() {
    let program = program_test();

    let (mut banks_client, payer, recent_blockhash) = program.start().await;
    let utility_amount = 100000;
    let socials = SocialAccounts::new(&payer.pubkey());
    socials
        .initialize(&mut banks_client, &payer, &recent_blockhash, utility_amount)
        .await;
    let mut transaction_post = Transaction::new_with_payer(
        &[create_post_transaction(
            &s2g::id(),
            &payer.pubkey(),
            &socials.user,
            &socials.channel,
            &socials.governence_mint,
            &socials.hash,
            &socials.source,
        )],
        Some(&payer.pubkey()),
    );
    transaction_post.sign(&[&payer], recent_blockhash);
    banks_client
        .process_transaction(transaction_post)
        .await
        .unwrap();

    let (escrow_account_info, _) = find_escrow_program_address(&s2g::id(), &socials.post);
    let rent = Rent::default();
    let stake = 1000;

    // Stake some
    socials
        .vote(&mut banks_client, &payer, Vote::DOWN, stake)
        .await;

    let escrow_account = banks_client
        .get_account(escrow_account_info)
        .await
        .unwrap()
        .unwrap();

    assert!(rent.is_exempt(escrow_account.lamports, escrow_account.data.len()));

    assert_token_balance(
        &mut banks_client,
        &socials.user_token_account,
        utility_amount - stake,
    )
    .await;
    assert_token_balance(&mut banks_client, &socials.downvote_token_account, stake).await;
    assert_token_balance(&mut banks_client, &escrow_account_info, stake).await;
    socials.assert_vote(&mut banks_client, 0, stake).await;

    // Stake more
    socials
        .vote(&mut banks_client, &payer, Vote::DOWN, stake)
        .await;

    assert_token_balance(
        &mut banks_client,
        &socials.user_token_account,
        utility_amount - stake * 2,
    )
    .await;

    assert_token_balance(
        &mut banks_client,
        &socials.downvote_token_account,
        stake * 2,
    )
    .await;
    assert_token_balance(&mut banks_client, &escrow_account_info, stake * 2).await;
    socials.assert_vote(&mut banks_client, 0, stake * 2);

    // Unstake
    socials
        .unvote(&mut banks_client, &payer, Vote::DOWN, stake)
        .await;

    assert_token_balance(
        &mut banks_client,
        &socials.user_token_account,
        utility_amount - stake,
    )
    .await;
    assert_token_balance(&mut banks_client, &socials.downvote_token_account, stake).await;
    assert_token_balance(&mut banks_client, &escrow_account_info, stake).await;
    socials.assert_vote(&mut banks_client, 0, stake);

    // Unstake, same amount (we should now 0 token accounts)
    socials
        .unvote(&mut banks_client, &payer, Vote::DOWN, stake)
        .await;

    assert_token_balance(
        &mut banks_client,
        &socials.user_token_account,
        utility_amount,
    )
    .await;
    assert_token_balance(&mut banks_client, &socials.downvote_token_account, 0).await;
    assert_token_balance(&mut banks_client, &escrow_account_info, 0).await;
    socials.assert_vote(&mut banks_client, 0, 0);
}
