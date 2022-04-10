use ltag::{
    get_tag_program_address, get_tag_record_factory_program_address, get_tag_record_program_address,
};
use shared::content::ContentSource;
use solana_program::{
    borsh::try_from_slice_unchecked, hash::Hash, program_error::ProgramError, program_pack::Pack,
    system_instruction,
};

use lsocial::{
    instruction::{
        cast_vote, create_post, delete_channel_authority, uncast_vote, update_info,
        CreateVoteConfig, SignedAuthority, SignedAuthorityCondition,
    },
    state::{
        channel_authority::{
            get_channel_authority_address, AuthorityCondition, AuthorityType, ChannelAuthority,
        },
        post::{get_post_program_address, PostAccount, PostContent},
        vote_record::Vote,
    },
};
use lsocial::{
    instruction::{create_channel, create_channel_authority},
    state::channel::{get_channel_program_address, ChannelAccount},
};

use solana_program_test::*;
use solana_sdk::{
    pubkey::Pubkey, signature::Keypair, signer::Signer, transaction::Transaction,
    transport::TransportError,
};
use spl_associated_token_account::{create_associated_token_account, get_associated_token_address};

use crate::bench::{clone_keypair, ProgramTestBench};
pub fn deserialize_channel_account(data: &[u8]) -> std::io::Result<ChannelAccount> {
    let account = try_from_slice_unchecked::<ChannelAccount>(data)?;
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
    (get_post_program_address(&lsocial::id(), &hash).0, hash)
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
pub struct TestTagRecordFactory {
    pub tag: Pubkey,
    pub factory: Pubkey,
    pub authority: Keypair,
}

impl TestTagRecordFactory {
    pub async fn new(bench: &mut ProgramTestBench) -> Self {
        let authority = Keypair::new();
        let tag = Pubkey::new_unique().to_string();
        bench
            .process_transaction(
                &[ltag::instruction::create_tag(
                    &ltag::id(),
                    tag.as_ref(),
                    None,
                    &authority.pubkey(),
                    &bench.payer.pubkey(),
                )],
                Some(&[&authority]),
            )
            .await
            .unwrap();

        let tag_address = get_tag_program_address(&ltag::id(), tag.as_ref()).0;
        bench
            .process_transaction(
                &[ltag::instruction::create_tag_record_factory(
                    &ltag::id(),
                    &tag_address,
                    &authority.pubkey(),
                    &bench.payer.pubkey(),
                )],
                Some(&[&authority]),
            )
            .await
            .unwrap();

        let tag_record_factory_address =
            get_tag_record_factory_program_address(&ltag::id(), &tag_address, &authority.pubkey())
                .0;
        Self {
            tag: tag_address,
            factory: tag_record_factory_address,
            authority,
        }
    }

    pub async fn new_record(&self, bench: &mut ProgramTestBench, owner: &TestUser) {
        bench
            .process_transaction(
                &[ltag::instruction::create_tag_record(
                    &ltag::id(),
                    &self.tag,
                    &owner.keypair.pubkey(),
                    &self.factory,
                    &self.authority.pubkey(),
                    &bench.payer.pubkey(),
                )],
                Some(&[&self.authority]),
            )
            .await
            .unwrap();
    }
}

pub struct TestAuthority {
    pub key: Pubkey,
}
pub struct TestSignedAuthority<'a> {
    pub signer: &'a Keypair,
    pub authority: SignedAuthority,
}
impl TestAuthority {
    pub async fn new<'a>(
        bench: &mut ProgramTestBench,
        channel: &TestChannel,
        authority_types: &Vec<AuthorityType>,
        condition: &AuthorityCondition,
        signing_authority: &TestSignedAuthority<'a>,
    ) -> Self {
        let seed = Pubkey::new_unique();
        bench
            .process_transaction(
                &[create_channel_authority(
                    &lsocial::id(),
                    &channel.channel,
                    &bench.payer.pubkey(),
                    authority_types,
                    condition,
                    &seed,
                    &signing_authority.authority,
                )],
                Some(&[signing_authority.signer]),
            )
            .await
            .unwrap();

        Self {
            key: get_channel_authority_address(&lsocial::id(), &channel.channel, &seed).0,
        }
    }

    pub async fn delete<'a>(
        &self,
        bench: &mut ProgramTestBench,
        signing_authority: &TestSignedAuthority<'a>,
    ) {
        let account = self.get_channel_authority_account(bench).await.unwrap();
        bench
            .process_transaction(
                &[delete_channel_authority(
                    &lsocial::id(),
                    &self.key,
                    &account.channel,
                    &bench.payer.pubkey(),
                    &signing_authority.authority,
                )],
                Some(&[signing_authority.signer]),
            )
            .await
            .unwrap();

        assert!(self.get_channel_authority_account(bench).await.is_none());
    }

    pub async fn get_channel_authority_account(
        &self,
        bench: &mut ProgramTestBench,
    ) -> Option<ChannelAuthority> {
        let account = bench.get_account(&self.key).await;
        if let Some(account) = account {
            Some(try_from_slice_unchecked::<ChannelAuthority>(&account.data).unwrap())
        } else {
            None
        }
    }
    pub async fn get_signing_authority<'a>(
        &self,
        bench: &mut ProgramTestBench,
        user: &'a TestUser,
    ) -> TestSignedAuthority<'a> {
        let account = self.get_channel_authority_account(bench).await.unwrap();
        let condition = match account.condition {
            AuthorityCondition::None => SignedAuthorityCondition::None,
            AuthorityCondition::Pubkey(key) => SignedAuthorityCondition::Pubkey(key),
            AuthorityCondition::Tag { record_factory } => {
                let record = user.get_tag_record_address(&record_factory);
                SignedAuthorityCondition::Tag {
                    record_factory,
                    record,
                    owner: user.keypair.pubkey(),
                }
            }
        };
        TestSignedAuthority {
            signer: &user.keypair,
            authority: SignedAuthority {
                channel_authority: self.key,
                condition,
            },
        }
    }
}

pub struct TestChannel {
    pub channel: Pubkey,
}

impl TestChannel {
    pub async fn new<'a>(
        bench: &mut ProgramTestBench,
        admin: &TestUser,
        info: Option<ContentSource>,
        parent: Option<&TestChannel>,
        activity_authority: Option<&TestSignedAuthority<'a>>,
    ) -> (Self, TestAuthority) {
        let channel_name = Pubkey::new_unique().to_string();
        let (channel_address_pda, _bump) =
            get_channel_program_address(&lsocial::id(), channel_name.as_ref()).unwrap();
        let channel_authority_seed = Pubkey::new_unique();
        let mut signers = vec![clone_keypair(&admin.keypair)];
        if let Some(activity_authority) = activity_authority {
            signers.push(clone_keypair(activity_authority.signer));
        }

        bench
            .process_transaction(
                &[create_channel(
                    &lsocial::id(),
                    parent.map(|p| p.channel),
                    &admin.keypair.pubkey(),
                    &bench.payer.pubkey(),
                    channel_name.as_ref(),
                    info,
                    &channel_authority_seed,
                    activity_authority.map(|x| &x.authority),
                )],
                Some(&signers.iter().collect::<Vec<&Keypair>>()),
            )
            .await
            .unwrap();

        (
            Self {
                channel: channel_address_pda,
            },
            TestAuthority {
                key: get_channel_authority_address(
                    &lsocial::id(),
                    &channel_address_pda,
                    &channel_authority_seed,
                )
                .0,
            },
        )
    }

    pub async fn get_channel_account(&self, bench: &mut ProgramTestBench) -> ChannelAccount {
        let channel = try_from_slice_unchecked::<ChannelAccount>(
            &*bench.get_account(&self.channel).await.unwrap().data,
        )
        .unwrap();
        channel
    }

    pub async fn update_info<'a>(
        &self,
        bench: &mut ProgramTestBench,
        new_info: Option<ContentSource>,
        activity_authority: &TestSignedAuthority<'a>,
    ) {
        bench
            .process_transaction(
                &[update_info(
                    &lsocial::id(),
                    &self.channel,
                    new_info,
                    &activity_authority.authority,
                )],
                Some(&[activity_authority.signer]),
            )
            .await
            .unwrap();
    }

    /* pub async fn get_authority_config(
        &self,
        bench: &mut ProgramTestBench,
        owner: &Pubkey,
    ) -> SigningActivityAuthority {
        let account = self.get_channel_account(bench).await;
        match account.activity_authority {
            ActivityAuthority::Tag { record_factory } => SigningActivityAuthority::Tag {
                record_factory,
                owner: *owner,
            },
            ActivityAuthority::None => SigningActivityAuthority::None,
        }
    } */
}

pub struct TestUser {
    pub keypair: Keypair,
}

impl TestUser {
    pub fn new() -> Self {
        Self {
            keypair: Keypair::new(),
        }
    }

    pub fn get_tag_record_address(&self, tag_record_factory: &Pubkey) -> Pubkey {
        get_tag_record_program_address(&ltag::id(), tag_record_factory, &self.keypair.pubkey()).0
    }
}

pub struct TestPost<'a> {
    pub post: Pubkey,
    pub channel: &'a TestChannel,
    pub source: ContentSource,
    pub hash: [u8; 32],
    pub proposal_transactions: Vec<Pubkey>,
}

impl<'a> TestPost<'a> {
    pub async fn new(
        bench: &mut ProgramTestBench,
        channel: &'a TestChannel,
        owner: &TestUser,
        content: &PostContent,
        parent: Option<&'a TestPost<'a>>,
        signing_authority: &TestSignedAuthority<'a>,
    ) -> TestPost<'a> {
        let (post, hash) = create_post_hash();
        bench
            .process_transaction(
                &[create_post(
                    &lsocial::id(),
                    &channel.channel,
                    &owner.keypair.pubkey(),
                    &bench.payer.pubkey(),
                    content.clone(),
                    &hash,
                    parent.map(|p| p.post),
                    &CreateVoteConfig::Simple,
                    &signing_authority.authority,
                )],
                Some(&[signing_authority.signer]),
            )
            .await
            .unwrap();

        let post_account_info = bench
            .get_account(&post)
            .await
            .expect("post_account not found");
        let post_account =
            try_from_slice_unchecked::<PostAccount>(&post_account_info.data).unwrap();

        assert_eq!(post_account.hash, hash);

        if parent.is_some() {
            assert_eq!(post_account.parent, parent.unwrap().post);
        } else {
            assert_eq!(post_account.parent, post_account.channel);
        }

        assert!(post_account.deleted_at_timestamp.is_none());

        Self {
            post,
            hash,
            source: ContentSource::External {
                url: "whatever".into(),
            },
            channel,
            proposal_transactions: Vec::new(),
        }
    }

    pub async fn vote(
        &self,
        bench: &mut ProgramTestBench,
        vote: Vote,
        owner: &TestUser,
        signing_authority: &TestSignedAuthority<'a>,
    ) -> Result<(), ProgramError> {
        bench
            .process_transaction(
                &[cast_vote(
                    &lsocial::id(),
                    &self.post,
                    &self.channel.channel,
                    &owner.keypair.pubkey(),
                    &bench.payer.pubkey(),
                    vote,
                    &signing_authority.authority,
                )],
                Some(&[&owner.keypair, signing_authority.signer]),
            )
            .await?;

        Ok(())
    }

    pub async fn unvote(
        &self,
        bench: &mut ProgramTestBench,
        owner: &TestUser,
        signing_authority: &TestSignedAuthority<'a>,
    ) -> Result<(), ProgramError> {
        let pre_balance = bench
            .get_account(&bench.payer.pubkey())
            .await
            .unwrap()
            .lamports;
        bench
            .process_transaction(
                &[uncast_vote(
                    &lsocial::id(),
                    &self.post,
                    &self.channel.channel,
                    &owner.keypair.pubkey(),
                    &bench.payer.pubkey(),
                    &signing_authority.authority,
                )],
                Some(&[&owner.keypair, signing_authority.signer]),
            )
            .await?;

        let post_balance = bench
            .get_account(&bench.payer.pubkey())
            .await
            .unwrap()
            .lamports;
        // We got some balance bak since we are removing the vote record
        assert!(pre_balance < post_balance);
        Ok(())
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
            lsocial::state::post::VoteConfig::Simple { downvote, upvote } => {
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
