use solana_program::{hash::Hash, program_pack::Pack, rent::Rent};

use s2g::{
    socials::{
        channel::{
            find_channel_program_address,
            state::{deserialize_channel_account, ChannelAccount},
        },
        find_user_account_program_address,
        post::{
            find_escrow_program_address, find_post_downvote_mint_program_address,
            find_post_program_address, find_post_upvote_mint_program_address,
            find_treasury_token_account_address,
            instruction::{
                create_post_transaction, create_post_unvote_transaction,
                create_post_vote_transaction, CreatePostType,
            },
            state::{deserialize_post_account, ActionStatus, ContentSource, PostAccount, PostType},
            Vote,
        },
    },
    stake_pool::state::Fee,
    tokens::spl_utils::find_platform_mint_program_address,
};
use solana_program_test::*;
use solana_sdk::{
    commitment_config::CommitmentLevel, pubkey::Pubkey, signature::Keypair, signer::Signer,
    transaction::Transaction,
};
use spl_associated_token_account::{create_associated_token_account, get_associated_token_address};

use crate::{
    social::{channel::create_and_verify_channel, user::create_and_verify_user},
    utils::program_test,
};

pub async fn get_token_balance(banks_client: &mut BanksClient, token: &Pubkey) -> u64 {
    let token_account = banks_client.get_account(*token).await.unwrap().unwrap();
    let account_info: spl_token::state::Account =
        spl_token::state::Account::unpack_from_slice(token_account.data.as_slice()).unwrap();
    account_info.amount
}

pub fn create_post_hash() -> (Pubkey, [u8; 32]) {
    let hash = Pubkey::new_unique().to_bytes();
    (Pubkey::find_program_address(&[&hash], &s2g::id()).0, hash)
}

pub async fn assert_token_balance(banks_client: &mut BanksClient, account: &Pubkey, amount: u64) {
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
    if expected_balance == 0 {
        return;
    }

    let mut stake_pool_accounts =
        super::super::super::stake_pool::helpers::StakePoolAccounts::new();
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
pub struct TestUser {
    pub upvote_token_account: Pubkey,
    pub downvote_token_account: Pubkey,
    pub payer: Pubkey,
}

impl TestUser {
    pub fn new(payer: &Pubkey, test_post: &TestPost) -> Self {
        Self {
            payer: *payer,
            upvote_token_account: get_associated_token_address(&payer, &test_post.upvote_mint),
            downvote_token_account: get_associated_token_address(&payer, &test_post.downvote_mint),
        }
    }

    pub async fn get_token_account(
        &self,
        banks_client: &mut BanksClient,
        mint: &Pubkey,
    ) -> spl_token::state::Account {
        spl_token::state::Account::unpack(
            banks_client
                .get_account(get_associated_token_address(&self.payer, mint))
                .await
                .unwrap()
                .unwrap()
                .data
                .as_slice(),
        )
        .unwrap()
    }
}

pub struct TestPost {
    pub post: Pubkey,
    pub channel: Pubkey,
    pub source: ContentSource,
    pub hash: [u8; 32],
    pub upvote_mint: Pubkey,
    pub downvote_mint: Pubkey,
}

impl TestPost {
    pub fn new(channel: &Pubkey) -> Self {
        let (post, hash) = create_post_hash();
        let upvote_mint = find_post_upvote_mint_program_address(&s2g::id(), &post).0;
        let downvote_mint = find_post_downvote_mint_program_address(&s2g::id(), &post).0;
        Self {
            post,
            hash,
            source: ContentSource::External {
                url: "whatever".into(),
            },
            channel: *channel,
            upvote_mint,
            downvote_mint,
        }
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
        let channel_account = deserialize_channel_account(
            &*banks_client
                .get_account(self.channel)
                .await
                .unwrap()
                .unwrap()
                .data,
        )
        .unwrap();
        let mut tx = match vote {
            Vote::Up => Transaction::new_with_payer(
                &[create_post_vote_transaction(
                    &s2g::id(),
                    &payer.pubkey(),
                    &self.post,
                    &channel_account.governence_mint,
                    amount,
                    Vote::Up,
                )],
                Some(&payer.pubkey()),
            ),
            Vote::Down => Transaction::new_with_payer(
                &[create_post_vote_transaction(
                    &s2g::id(),
                    &payer.pubkey(),
                    &self.post,
                    &channel_account.governence_mint,
                    amount,
                    Vote::Down,
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
        let channel_account = deserialize_channel_account(
            &*banks_client
                .get_account(self.channel)
                .await
                .unwrap()
                .unwrap()
                .data,
        )
        .unwrap();
        let mut tx = match vote {
            Vote::Up => Transaction::new_with_payer(
                &[create_post_unvote_transaction(
                    &s2g::id(),
                    &payer.pubkey(),
                    &self.post,
                    &channel_account.governence_mint,
                    amount,
                    Vote::Up,
                )],
                Some(&payer.pubkey()),
            ),
            Vote::Down => Transaction::new_with_payer(
                &[create_post_unvote_transaction(
                    &s2g::id(),
                    &payer.pubkey(),
                    &self.post,
                    &channel_account.governence_mint,
                    amount,
                    Vote::Down,
                )],
                Some(&payer.pubkey()),
            ),
        };
        tx.sign(&[payer], banks_client.get_latest_blockhash().await.unwrap());
        banks_client.process_transaction(tx).await.unwrap();
    }

    pub async fn get_post_account(&self, banks_client: &mut BanksClient) -> PostAccount {
        deserialize_post_account(
            &*banks_client
                .get_account(self.post)
                .await
                .unwrap()
                .unwrap()
                .data,
        )
        .unwrap()
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
            PostType::InformalPost(s) => {
                assert_eq!(s.upvotes, upvotes);
                assert_eq!(s.downvotes, downvotes);
            }
            PostType::ActionPost(s) => {
                assert_eq!(s.upvotes, upvotes);
                assert_eq!(s.downvotes, downvotes);
            }
        };
    }
}

pub struct SocialAccounts {
    pub username: String,
    pub user: Pubkey,
    pub user_token_account: Pubkey,
    pub channel: Pubkey,
    pub governence_mint: Pubkey,
}

impl SocialAccounts {
    pub fn new(payer: &Pubkey) -> Self {
        let (utility_mint_address, __bump) = find_platform_mint_program_address(&s2g::id());
        return SocialAccounts::new_with_org(payer, &utility_mint_address);
    }
    pub fn new_with_org(payer: &Pubkey, governence_mint: &Pubkey) -> Self {
        let username = "name";
        let (post, hash) = create_post_hash();
        let (channel, ___bump) = find_channel_program_address(&s2g::id(), "Channel").unwrap();

        Self {
            username: username.into(),
            user: find_user_account_program_address(&s2g::id(), username).0,
            user_token_account: get_associated_token_address(
                payer,
                &find_platform_mint_program_address(&s2g::id()).0,
            ),
            channel,
            governence_mint: *governence_mint,
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
        assert_eq!(channel, self.channel);
    }

    pub async fn get_channel_account(&self, banks_client: &mut BanksClient) -> ChannelAccount {
        deserialize_channel_account(
            &*banks_client
                .get_account(self.channel)
                .await
                .unwrap()
                .unwrap()
                .data,
        )
        .unwrap()
    }

    pub fn get_treasury_address(&self, mint: &Pubkey) -> Pubkey {
        find_treasury_token_account_address(&self.channel, mint, &spl_token::id(), &s2g::id()).0
    }

    pub async fn get_treasury_account(
        &self,
        banks_client: &mut BanksClient,
        mint: &Pubkey,
    ) -> spl_token::state::Account {
        spl_token::state::Account::unpack(
            banks_client
                .get_account(self.get_treasury_address(mint))
                .await
                .unwrap()
                .unwrap()
                .data
                .as_slice(),
        )
        .unwrap()
    }
}
