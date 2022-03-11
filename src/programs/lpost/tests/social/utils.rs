use ltag::{
    get_tag_program_address, get_tag_record_program_address,
    state::{TagAccount, TagRecordAccount},
};
use shared::content::ContentSource;
use solana_program::{
    borsh::try_from_slice_unchecked, hash::Hash, program_pack::Pack, system_instruction,
};

use lchannel::{
    instruction::create_channel,
    state::{find_channel_program_address, ChannelAccount, ChannelAuthority},
};
use lpost::{
    instruction::{cast_vote, create_post, uncast_vote, CreateVoteConfig, SigningChannelAuthority},
    state::{
        post::{PostAccount, PostContent},
        vote_record::Vote,
    },
};
/* use luser::{
    find_user_account_program_address, instruction::create_user_transaction,
    state::deserialize_user_account,
}; */
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

pub async fn create_tag(
    banks_client: &mut BanksClient,
    payer: &Keypair,
    recent_blockhash: &Hash,
    tag: &str,
) -> Pubkey {
    // Create tag
    banks_client
        .process_transaction(Transaction::new_signed_with_payer(
            &[ltag::instruction::create_tag(
                &ltag::id(),
                tag,
                &payer.pubkey(),
            )],
            Some(&payer.pubkey()),
            &[payer],
            *recent_blockhash,
        ))
        .await
        .unwrap();

    let tag_address = get_tag_program_address(&ltag::id(), tag).0;
    let tag_account_info = banks_client
        .get_account(tag_address)
        .await
        .expect("get_tag")
        .expect("tag not found");
    let tag_account = try_from_slice_unchecked::<TagAccount>(&tag_account_info.data).unwrap();
    assert_eq!(tag_account.tag, tag);
    tag_address
}

pub async fn create_tag_record(
    banks_client: &mut BanksClient,
    payer: &Keypair,
    recent_blockhash: &Hash,
    tag: &Pubkey,
    owner: &Keypair,
    authority: &Keypair,
) -> Pubkey {
    // Create tag record
    banks_client
        .process_transaction(Transaction::new_signed_with_payer(
            &[ltag::instruction::create_tag_record(
                &ltag::id(),
                tag,
                &owner.pubkey(),
                &authority.pubkey(),
                &payer.pubkey(),
            )],
            Some(&payer.pubkey()),
            &[payer, owner, authority],
            *recent_blockhash,
        ))
        .await
        .unwrap();

    let tag_record_address =
        get_tag_record_program_address(&ltag::id(), tag, &owner.pubkey(), &authority.pubkey()).0;
    let tag_record_account_info = banks_client
        .get_account(tag_record_address)
        .await
        .expect("get_tag_record")
        .expect("tag not found");
    let tag_record_account =
        try_from_slice_unchecked::<TagRecordAccount>(&tag_record_account_info.data).unwrap();
    assert_eq!(&tag_record_account.authority, &authority.pubkey());
    assert_eq!(&tag_record_account.owner, &owner.pubkey());
    assert_eq!(&tag_record_account.tag, tag);
    tag_record_address
}

/* pub async fn create_and_verify_user(
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
} */

pub async fn create_and_verify_channel(
    channel_name: &str,
    channel_governance_config: &ChannelAuthority,
    link: Option<ContentSource>,
    banks_client: &mut BanksClient,
    payer: &Keypair,
    recent_blockhash: &Hash,
) -> Result<Pubkey, TransportError> {
    let (channel_address_pda, _bump) =
        find_channel_program_address(&lchannel::id(), channel_name).unwrap();

    let mut transaction_create = Transaction::new_with_payer(
        &[create_channel(
            &lchannel::id(),
            channel_name,
            &payer.pubkey(),
            None,
            link,
            channel_governance_config.clone(),
            &payer.pubkey(),
        )],
        Some(&payer.pubkey()),
    );
    transaction_create.sign(&[payer, payer], *recent_blockhash);
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

/*
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

    /*  pub fn upvote_token_account(&self, post: &TestPost) -> Pubkey {
        get_associated_token_address(&self.payer, &post.upvote_mint)
    }

    pub fn downvote_token_account(&self, post: &TestPost) -> Pubkey {
        get_associated_token_address(&self.payer, &post.downvote_mint)
    } */
    pub fn token_account(&self, token: &TestGovernanceToken) -> Pubkey {
        get_associated_token_address(&self.payer, &token.mint)
    }
} */

pub struct TestPost {
    pub post: Pubkey,
    pub channel: TestChannel,
    pub source: ContentSource,
    pub hash: [u8; 32],
    pub proposal_transactions: Vec<Pubkey>,
}

impl TestPost {
    pub async fn new(
        channel: &TestChannel,
        content: &PostContent,
        owner: &Keypair,
        payer: &Keypair,
        banks_client: &mut BanksClient,
        recent_blockhash: &Hash,
    ) -> Self {
        let (post, hash) = create_post_hash();
        let authority_config = channel
            .get_authority_config(banks_client, &owner.pubkey())
            .await;
        let mut transaction_create = Transaction::new_with_payer(
            &[create_post(
                &lpost::id(),
                &payer.pubkey(),
                &channel.channel,
                &hash,
                content,
                &CreateVoteConfig::Simple,
                &authority_config,
            )],
            Some(&payer.pubkey()),
        );
        transaction_create.sign(&[payer, owner], *recent_blockhash);
        banks_client
            .process_transaction(transaction_create)
            .await
            .unwrap();

        // Verify channel name
        let post_account_info = banks_client
            .get_account(post)
            .await
            .expect("get_account")
            .expect("channel_account not found");
        let post_account =
            try_from_slice_unchecked::<PostAccount>(&post_account_info.data).unwrap();

        assert_eq!(post_account.hash, hash);

        Self {
            post,
            hash,
            source: ContentSource::External {
                url: "whatever".into(),
            },
            channel: channel.clone(),
            proposal_transactions: Vec::new(),
        }
    }

    /* pub async fn get_proposal_transactions(
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
    } */
    /* pub async fn get_proposal_used_rules(&self, banks_client: &mut BanksClient) {
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
    } */

    pub async fn vote(
        &self,
        vote: Vote,
        owner: &Keypair,
        banks_client: &mut BanksClient,
        payer: &Keypair,
        recent_blockhash: &Hash,
    ) -> Result<(), TransportError> {
        let authority_config = self
            .channel
            .get_authority_config(banks_client, &owner.pubkey())
            .await;

        let mut tx = Transaction::new_with_payer(
            &[cast_vote(
                &lpost::id(),
                &payer.pubkey(),
                &self.post,
                &self.channel.channel,
                &owner.pubkey(),
                &authority_config,
                vote,
            )],
            Some(&payer.pubkey()),
        );
        tx.sign(&[payer, owner], *recent_blockhash);
        banks_client.process_transaction(tx).await
    }

    pub async fn unvote(
        &self,
        owner: &Keypair,
        banks_client: &mut BanksClient,
        payer: &Keypair,
        recent_blockhash: &Hash,
    ) -> Result<(), TransportError> {
        let mut tx = Transaction::new_with_payer(
            &[uncast_vote(
                &lpost::id(),
                &self.post,
                &owner.pubkey(),
                &payer.pubkey(),
            )],
            Some(&payer.pubkey()),
        );
        tx.sign(&[payer, owner], *recent_blockhash);
        banks_client.process_transaction(tx).await
    }

    pub async fn get_post_account(&self, banks_client: &mut BanksClient) -> PostAccount {
        try_from_slice_unchecked::<PostAccount>(
            &*banks_client
                .get_account(self.post)
                .await
                .unwrap()
                .unwrap()
                .data,
        )
        .unwrap()
    }

    pub async fn assert_votes(&self, banks_client: &mut BanksClient, upvotes: u64, downvotes: u64) {
        let post_account = self.get_post_account(banks_client).await;
        match post_account.vote_config {
            lpost::state::post::VoteConfig::Simple { downvote, upvote } => {
                assert_eq!(upvote, upvotes);
                assert_eq!(downvote, downvotes);
            }
        };
    }

    /*
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
            PostType::Proposal(s) => {
                assert_eq!(s.upvotes, upvotes);
                assert_eq!(s.downvotes, downvotes);
            }
        };
    } */
}

pub struct TestGovernanceToken {
    pub mint: Pubkey,
}
impl TestGovernanceToken {
    pub async fn new(
        banks_client: &mut BanksClient,
        authority_payer: &Keypair,
        recent_blockhash: &Hash,
    ) -> Self {
        let mint = Keypair::new();
        create_mint_from_keypair(
            banks_client,
            authority_payer,
            recent_blockhash,
            &mint,
            &authority_payer.pubkey(), // Payer is auth
        )
        .await
        .unwrap();
        Self {
            mint: mint.pubkey(),
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
}

#[derive(Clone, Debug)]
pub struct TestChannel {
    pub channel: Pubkey,
}

impl TestChannel {
    pub async fn new(
        channel_authority: &ChannelAuthority,
        banks_client: &mut BanksClient,
        authority_payer: &Keypair,
        recent_blockhash: &Hash,
    ) -> Self {
        let channel_name = Pubkey::new_unique().to_string();
        let channel = create_and_verify_channel(
            channel_name.as_ref(),
            channel_authority,
            None,
            banks_client,
            authority_payer,
            recent_blockhash,
        )
        .await
        .unwrap();

        Self { channel }
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

    pub async fn get_authority_config(
        &self,
        banks_client: &mut BanksClient,
        owner: &Pubkey,
    ) -> SigningChannelAuthority {
        let account = self.get_channel_account(banks_client).await;
        match account.channel_authority_config {
            ChannelAuthority::AuthorityByTag { authority, tag } => {
                SigningChannelAuthority::AuthorityByTag {
                    authority,
                    owner: *owner,
                    tag,
                }
            }
        }
    }
}
