use std::future::Future;

use shared::content::ContentSource;
use solana_program::{
    borsh::try_from_slice_unchecked, hash::Hash, program_pack::Pack, system_instruction,
};

use lchannel::{
    find_channel_program_address, instruction::create_channel_transaction, state::ChannelAccount,
};
use lpost::{
    find_post_downvote_mint_program_address, find_post_upvote_mint_program_address,
    find_treasury_token_account_address,
    instruction::{
        create_create_rule_transaction, create_post_unvote_transaction,
        create_post_vote_transaction,
    },
    state::{
        post::{deserialize_post_account, PostAccount, PostType},
        proposal::proposal_transaction::ProposalTransactionV2,
        rules::rule::{AcceptenceCriteria, RuleTimeConfig, RuleVoteConfig},
        vote_record::Vote,
    },
};
use luser::{
    find_user_account_program_address, instruction::create_user_transaction,
    state::deserialize_user_account,
};
use solana_program_test::*;
use solana_sdk::{
    pubkey::Pubkey, signature::Keypair, signer::Signer, transaction::Transaction,
    transport::TransportError,
};
use spl_associated_token_account::{create_associated_token_account, get_associated_token_address};
pub fn deserialize_channel_account(data: &[u8]) -> std::io::Result<ChannelAccount> {
    let account: ChannelAccount = try_from_slice_unchecked(data)?;
    Ok(account)
}

pub async fn get_token_balance(banks_client: &mut BanksClient, token: &Pubkey) -> u64 {
    let token_account = banks_client.get_account(*token).await.unwrap().unwrap();
    let account_info: spl_token::state::Account =
        spl_token::state::Account::unpack_from_slice(token_account.data.as_slice()).unwrap();
    account_info.amount
}

pub fn create_post_hash() -> (Pubkey, [u8; 32]) {
    let hash = Pubkey::new_unique().to_bytes();
    (Pubkey::find_program_address(&[&hash], &lpost::id()).0, hash)
}

pub async fn create_mint_from_keypair(
    banks_client: &mut BanksClient,
    payer: &Keypair,
    recent_blockhash: &Hash,
    pool_mint: &Keypair,
    pool_mint_authority: &Pubkey,
) -> Result<(), TransportError> {
    let rent = banks_client.get_rent().await.unwrap();
    let mint_rent = rent.minimum_balance(spl_token::state::Mint::LEN);

    let mut transaction = Transaction::new_with_payer(
        &[
            system_instruction::create_account(
                &payer.pubkey(),
                &pool_mint.pubkey(),
                mint_rent,
                spl_token::state::Mint::LEN as u64,
                &spl_token::id(),
            ),
            spl_token::instruction::initialize_mint(
                &spl_token::id(),
                &pool_mint.pubkey(),
                pool_mint_authority,
                None,
                0,
            )
            .unwrap(),
        ],
        Some(&payer.pubkey()),
    );
    transaction.sign(&[payer, pool_mint], *recent_blockhash);
    #[allow(clippy::useless_conversion)] // Remove during upgrade to 1.10
    banks_client
        .process_transaction(transaction)
        .await
        .map_err(|e| e.into())
}

pub async fn create_mint_with_payer_authority(
    banks_client: &mut BanksClient,
    payer: &Keypair,
    recent_blockhash: &Hash,
    _amount: u64,
) -> Keypair {
    let mint = Keypair::new();
    let _mint_pubkey = mint.pubkey();
    create_mint_from_keypair(
        banks_client,
        payer,
        recent_blockhash,
        &mint,
        &payer.pubkey(),
    )
    .await
    .unwrap();
    mint
}

pub async fn assert_token_balance(banks_client: &mut BanksClient, account: &Pubkey, amount: u64) {
    banks_client.get_latest_blockhash().await.unwrap();
    assert_eq!(
        spl_token::state::Account::unpack(
            &*banks_client
                .get_account(*account)
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

pub async fn create_and_verify_user(
    banks_client: &mut BanksClient,
    payer: &Keypair,
    recent_blockhash: &Hash,
    username: &str,
    profile: &str,
) -> Pubkey {
    // Create user
    banks_client
        .process_transaction(Transaction::new_signed_with_payer(
            &[create_user_transaction(
                &luser::id(),
                username,
                Some(profile.into()),
                &payer.pubkey(),
            )],
            Some(&payer.pubkey()),
            &[payer],
            *recent_blockhash,
        ))
        .await
        .unwrap();

    // Verify username name
    let user_account_address = find_user_account_program_address(&luser::id(), username).0;
    let user_account_info = banks_client
        .get_account(user_account_address)
        .await
        .expect("get_user")
        .expect("user not found");
    let user = deserialize_user_account(&user_account_info.data).unwrap();
    assert_eq!(user.name, username);
    user_account_address
}

pub async fn create_and_verify_channel(
    banks_client: &mut BanksClient,
    payer: &Keypair,
    recent_blockhash: &Hash,
    channel_name: &str,
    channel_creator_user: &Pubkey,
    utility_mint_address: &Pubkey,
    link: Option<ContentSource>,
) -> Result<Pubkey, TransportError> {
    let (channel_address_pda, _bump) =
        find_channel_program_address(&lchannel::id(), channel_name).unwrap();

    let mut transaction_create = Transaction::new_with_payer(
        &[
            create_channel_transaction(
                &lchannel::id(),
                channel_name,
                channel_creator_user,
                None,
                link,
                &payer.pubkey(),
            ),
            create_create_rule_transaction(
                &lpost::id(),
                &payer.pubkey(),
                &RuleVoteConfig {
                    criteria: AcceptenceCriteria::default(),
                    rule_condition: None,
                    info: None,
                    name: None,
                    vote_tipping: lpost::state::enums::VoteTipping::Disabled,
                },
                &RuleTimeConfig {
                    max_voting_time: 100000,
                    min_transaction_hold_up_time: 0,
                    proposal_cool_off_time: 0,
                },
            ),
        ],
        Some(&payer.pubkey()),
    );
    transaction_create.sign(&[payer], *recent_blockhash);
    banks_client.process_transaction(transaction_create).await?;

    // Verify channel name
    let channel_account_info = banks_client
        .get_account(channel_address_pda)
        .await
        .expect("get_account")
        .expect("channel_account not found");
    let channel_account = deserialize_channel_account(&channel_account_info.data).unwrap();

    assert_eq!(channel_account.name.as_str(), channel_name);
    Ok(channel_address_pda)
}

/* pub async fn ensure_utility_token_balance(
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
} */
pub struct TestUser {
    pub payer: Pubkey,
    pub user: Pubkey,
}

impl TestUser {
    pub async fn new(
        banks_client: &mut BanksClient,
        payer: &Keypair,
        recent_blockhash: &Hash,
    ) -> Self {
        let username = Pubkey::new_unique().to_string();
        let user = create_and_verify_user(
            banks_client,
            payer,
            recent_blockhash,
            username.as_str(),
            "profile",
        )
        .await;

        Self {
            payer: payer.pubkey(),
            user,
        }
    }

    pub fn upvote_token_account(&self, post: &TestPost) -> Pubkey {
        get_associated_token_address(&self.payer, &post.upvote_mint)
    }

    pub fn downvote_token_account(&self, post: &TestPost) -> Pubkey {
        get_associated_token_address(&self.payer, &post.downvote_mint)
    }
    pub fn token_account(&self, channel: &TestChannel) -> Pubkey {
        get_associated_token_address(&self.payer, &channel.mint)
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
    pub proposal_transactions: Vec<Pubkey>,
}

impl TestPost {
    pub async fn new(channel: &Pubkey) -> Self {
        let (post, hash) = create_post_hash();
        let upvote_mint = find_post_upvote_mint_program_address(&lpost::id(), &post).0;
        let downvote_mint = find_post_downvote_mint_program_address(&lpost::id(), &post).0;

        Self {
            post,
            hash,
            source: ContentSource::External {
                url: "whatever".into(),
            },
            channel: *channel,
            proposal_transactions: Vec::new(),
        }
    }

    pub async fn get_proposal_transactions(
        &self,
        banks_client: &mut BanksClient,
    ) -> Vec<Box<dyn Future<Output = std::io::Result<ProposalTransactionV2>>>> {
        let result = self
            .proposal_transactions
            .iter()
            .map(|id| async {
                let transaction: ProposalTransactionV2 = try_from_slice_unchecked(
                    &banks_client.get_account(*id).await.unwrap().unwrap().data,
                )
                .unwrap();
                return transaction;
            })
            .collect::<Vec<_>>();
        return result;
    }
    pub async fn get_proposal_used_rules(&self, banks_client: &mut BanksClient) {
        let x = self.proposal_transactions.iter().map(|id| async {
            let transaction: ProposalTransactionV2 = try_from_slice_unchecked(
                &banks_client.get_account(*id).await.unwrap().unwrap().data,
            )
            .unwrap();
            return transaction.get_used_rules();
        });

        if let PostType::Proposal(proposal) = post.post_type {
            banks_client.get_account(address)
        }
        return None;
    }

    pub async fn vote(&self, banks_client: &mut BanksClient, owner_payer: &Keypair, vote: Vote) {
        /*  let _post_account = deserialize_post_account(
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
                    &lpost::id(),
                    &user_payer.pubkey(),
                    &self.post,
                    &channel_account.governence_mint,
                    amount,
                    Vote::Up,
                )],
                Some(&user_payer.pubkey()),
            ),
            Vote::Down => Transaction::new_with_payer(
                &[create_post_vote_transaction(
                    &lpost::id(),
                    &user_payer.pubkey(),
                    &self.post,
                    &channel_account.governence_mint,
                    amount,
                    Vote::Down,
                )],
                Some(&user_payer.pubkey()),
            ),
        }; */
        // let rules =
        let mut tx = Transaction::new_with_payer(
            &[create_post_vote_transaction(
                &lpost::id(),
                &owner_payer.pubkey(),
                &self.post,
                vote,
            )],
            Some(&owner_payer.pubkey()),
        );
        tx.sign(
            &[owner_payer],
            banks_client.get_latest_blockhash().await.unwrap(),
        );
        banks_client.process_transaction(tx).await.unwrap();
    }

    pub async fn unvote(
        &self,
        banks_client: &mut BanksClient,
        user_payer: &Keypair,
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
                    &lpost::id(),
                    &user_payer.pubkey(),
                    &self.post,
                    &channel_account.governence_mint,
                    amount,
                    Vote::Up,
                )],
                Some(&user_payer.pubkey()),
            ),
            Vote::Down => Transaction::new_with_payer(
                &[create_post_unvote_transaction(
                    &lpost::id(),
                    &user_payer.pubkey(),
                    &self.post,
                    &channel_account.governence_mint,
                    amount,
                    Vote::Down,
                )],
                Some(&user_payer.pubkey()),
            ),
        };
        tx.sign(
            &[user_payer],
            banks_client.get_latest_blockhash().await.unwrap(),
        );
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
            PostType::InformationPost(s) => {
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

pub struct TestChannel {
    /*     pub username: String,
    pub user: Pubkey,
    pub user_token_account: Pubkey, */
    pub channel: Pubkey,
    pub mint: Pubkey,
}

impl TestChannel {
    pub async fn new(
        creator_user: &TestUser,
        banks_client: &mut BanksClient,
        payer: &Keypair,
        recent_blockhash: &Hash,
    ) -> Self {
        let mint = Keypair::new();
        create_mint_from_keypair(
            banks_client,
            payer,
            recent_blockhash,
            &mint,
            &payer.pubkey(), // Payer is auth
        )
        .await
        .unwrap();

        return TestChannel::new_with_mint(
            creator_user,
            &mint.pubkey(),
            banks_client,
            payer,
            recent_blockhash,
        )
        .await;
    }
    pub async fn new_with_mint(
        creator_user: &TestUser,
        mint: &Pubkey,
        banks_client: &mut BanksClient,
        payer: &Keypair,
        recent_blockhash: &Hash,
    ) -> Self {
        /*   let username = "name";
        let (post, hash) = create_post_hash(); */
        let channel_name = Pubkey::new_unique().to_string();
        let channel = create_and_verify_channel(
            banks_client,
            payer,
            recent_blockhash,
            channel_name.as_ref(),
            &creator_user.user,
            mint,
            None,
        )
        .await
        .unwrap();

        Self {
            channel,
            mint: *mint,
        }
    }

    pub async fn mint_to(
        &self,
        amount: u64,
        to: &Pubkey,
        banks_client: &mut BanksClient,
        payer_authority: &Keypair,
        recent_blockhash: &Hash,
    ) {
        let mut create_associated_token_account_transaction = Transaction::new_with_payer(
            &[create_associated_token_account(
                &payer_authority.pubkey(),
                to,
                &self.mint,
            )],
            Some(&payer_authority.pubkey()),
        );

        create_associated_token_account_transaction.sign(&[payer_authority], *recent_blockhash);
        banks_client
            .process_transaction(create_associated_token_account_transaction)
            .await
            .unwrap();

        let mut token_mint_transaction = Transaction::new_with_payer(
            &[spl_token::instruction::mint_to(
                &spl_token::id(),
                &self.mint,
                &get_associated_token_address(to, &self.mint),
                to,
                &[&payer_authority.pubkey()],
                amount,
            )
            .unwrap()],
            Some(&payer_authority.pubkey()),
        );

        token_mint_transaction.sign(&[payer_authority], *recent_blockhash);
        banks_client
            .process_transaction(token_mint_transaction)
            .await
            .unwrap();
    }

    /* pub async fn initialize(
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
    */
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
        find_treasury_token_account_address(&self.channel, mint, &spl_token::id(), &lpost::id()).0
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
