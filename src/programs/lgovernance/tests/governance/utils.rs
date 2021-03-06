use std::collections::{HashMap, HashSet};

use super::super::bench::ProgramTestBench;
use crate::bench::WalletCookie;

use ltag::{
    get_tag_program_address, get_tag_record_factory_program_address, get_tag_record_program_address,
};
use shared::content::ContentSource;
use solana_program::{
    borsh::try_from_slice_unchecked, instruction::AccountMeta, program_pack::Pack,
    system_instruction, system_program,
};

use lgovernance::{
    instruction::{
        cast_vote, count_vote_max_weights, count_votes, create_delegatee, create_governance,
        create_native_treasury, create_proposal, create_proposal_option, create_realm,
        create_scope, create_token_owner_budget_record, delegate, delegate_history,
        deposit_governing_tag, deposit_governing_tokens, execute_transaction, finalize_draft,
        insert_scope, insert_transaction, uncast_vote, undelegate, undelegate_history,
        update_governance_authority, CreateProposalOptionType, SignedCreateProposal,
    },
    state::{
        delegation::scope_delegation_record_account::{
            get_scope_delegation_account_program_address, ScopeDelegationRecordAccount,
        },
        enums::ProposalState,
        governance::{get_governance_address, GovernanceV2},
        native_treasury::get_native_treasury_address,
        proposal::{
            get_proposal_address,
            proposal_option::{get_proposal_option_program_address, ProposalOption},
            proposal_transaction::{
                get_proposal_transaction_address, ConditionedInstruction, ProposalTransactionV2,
            },
            ProposalV2, VoteType,
        },
        realm::get_realm_mint_program_address,
        scopes::scope::{get_scope_program_address, Scope, ScopeConfig, ScopeMatch, VotePowerUnit},
        token_owner_budget_record::get_token_owner_budget_record_address,
        vote_power_origin_record::get_vote_power_origin_record_address,
        vote_power_owner_record::{get_vote_power_owner_record_address, VotePowerOwnerRecord},
        vote_record::{get_vote_record_address, Vote, VoteRecordV2},
    },
};

use solana_program_test::*;
use solana_sdk::{pubkey::Pubkey, signature::Keypair, signer::Signer};
use spl_associated_token_account::{create_associated_token_account, get_associated_token_address};

pub enum TestVotePowerSource<'a> {
    TestToken(&'a TestToken),
    TestTagRecordFactory(&'a TestTagRecordFactory),
}

pub async fn get_token_balance(banks_client: &mut BanksClient, token: &Pubkey) -> u64 {
    let token_account = banks_client.get_account(*token).await.unwrap().unwrap();
    let account_info: spl_token::state::Account =
        spl_token::state::Account::unpack_from_slice(token_account.data.as_slice()).unwrap();
    account_info.amount
}

pub fn create_post_hash() -> (Pubkey, [u8; 32]) {
    let hash = Pubkey::new_unique().to_bytes();
    (
        Pubkey::find_program_address(&[&hash], &lgovernance::id()).0,
        hash,
    )
}
/*
pub async fn create_mint_from_keypair(
    banks_client: &mut BanksClient,
    payer: &Keypair,
    recent_blockhash: &Hash,
    mint: &Keypair,
    mint_authority: &Pubkey,
) -> Result<(), TransportError> {
    let rent = banks_client.get_rent().await.unwrap();
    let mint_rent = rent.minimum_balance(spl_token::state::Mint::LEN);

    let mut transaction = Transaction::new_with_payer(
        &[
            system_instruction::create_account(
                &bench.payer.pubkey(),
                &mint.pubkey(),
                mint_rent,
                spl_token::state::Mint::LEN as u64,
                &spl_token::id(),
            ),
            spl_token::instruction::initialize_mint(
                &spl_token::id(),
                &mint.pubkey(),
                mint_authority,
                None,
                0,
            )
            .unwrap(),
        ],
        Some(&bench.payer.pubkey()),
    );
    transaction.sign(&[payer, mint], *recent_blockhash);
    #[allow(clippy::useless_conversion)] // Remove during upgrade to 1.10
    banks_client
        .process_transaction(transaction)
        .await
        .map_err(|e| e.into())
}
 */
/// For deposits of goverance tokens

pub async fn create_token_holder_account(bench: &mut ProgramTestBench, mint: &Pubkey) {
    bench
        .process_transaction(
            &[create_realm(
                &lgovernance::id(),
                mint,
                &bench.payer.pubkey(),
            )],
            None,
        )
        .await
        .unwrap();
}
/*
pub async fn create_mint_with_payer_authority(
    banks_client: &mut BanksClient,
    payer: &Keypair,
    recent_blockhash: &Hash,
    amount: u64,
) -> Keypair {
    let mint = Keypair::new();
    let _mint_pubkey = mint.pubkey();
    create_mint_from_keypair(
        banks_client,
        payer,
        recent_blockhash,
        &mint,
        &bench.payer.pubkey(),
    )
    .await
    .unwrap();
    mint
}
 */
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
/*
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
                &bench.payer.pubkey(),
            )],
            Some(&bench.payer.pubkey()),
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
/*
pub async fn create_and_verify_channel(
    bench: &mut ProgramTestBench,
    channel_name: &str,
    channel_creator_user: &Keypair,
    channel_authority: &Keypair,
    info: Option<ContentSource>,
) -> Result<Pubkey, TransportError> {
    let (channel_address_pda, _bump) =
        find_channel_program_address(&lchannel::id(), channel_name).unwrap();

    bench
        .process_transaction(
            &[create_channel(
                &lchannel::id(),
                channel_name,
                &channel_creator_user.pubkey(),
                &channel_authority.pubkey(),
                None,
                &ActivityAuthority::None,
                info,
                &bench.context.payer.pubkey(),
            )],
            Some(&[channel_authority, channel_creator_user]),
        )
        .await
        .unwrap();

    // Verify channel name
    let channel_account_info = bench
        .get_account(&channel_address_pda)
        .await
        .expect("channel_account not found");
    let channel_account =
        try_from_slice_unchecked::<ChannelAccount>(&channel_account_info.data).unwrap();

    assert_eq!(channel_account.name.as_str(), channel_name);
    Ok(channel_address_pda)
} */

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
            &bench.payer.pubkey(),
            &bench.payer.pubkey(),
            &stake_pool_accounts.pool_mint,
        )],
        Some(&bench.payer.pubkey()),
    );
    transaction.sign(&[payer], *recent_blockhash);
    banks_client.process_transaction(transaction).await.unwrap();

    let associated_token_address =
        get_associated_token_address(&bench.payer.pubkey(), &stake_pool_accounts.pool_mint);

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
                    &authority.pubkey(),
                    &bench.payer.pubkey(),
                    tag.as_ref(),
                    None,
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

pub struct TestUser {
    pub keypair: Keypair,
}

impl TestUser {
    pub fn new() -> Self {
        Self {
            keypair: Keypair::new(),
        }
    }

    pub async fn create_associated_token_account(
        &self,
        bench: &mut ProgramTestBench,
        token: &TestToken,
    ) {
        bench
            .process_transaction(
                &[create_associated_token_account(
                    &bench.payer.pubkey(),
                    &self.keypair.pubkey(),
                    &token.mint,
                )],
                None,
            )
            .await
            .unwrap();
    }

    pub async fn create_delegatee(
        &self,
        bench: &mut ProgramTestBench,
        source: &VotePowerUnit,
        scope: &Pubkey,
    ) {
        bench
            .process_transaction(
                &[create_delegatee(
                    &lgovernance::id(),
                    &self.keypair.pubkey(),
                    &bench.payer.pubkey(),
                    scope,
                    source,
                )],
                Some(&[&self.keypair]),
            )
            .await
            .unwrap();
    }

    pub fn get_associated_token_account_address(&self, token: &TestToken) -> Pubkey {
        get_associated_token_address(&self.keypair.pubkey(), &token.mint)
    }

    pub fn get_tag_record_address(&self, tag_record_factory: &TestTagRecordFactory) -> Pubkey {
        get_tag_record_program_address(
            &ltag::id(),
            &tag_record_factory.factory,
            &self.keypair.pubkey(),
        )
        .0
    }

    pub async fn deposit_governance_tokens(
        &self,
        bench: &mut ProgramTestBench,
        amount: u64,
        token: &TestToken,
    ) {
        bench
            .process_transaction(
                &[deposit_governing_tokens(
                    &lgovernance::id(),
                    &self.get_associated_token_account_address(token),
                    &self.keypair.pubkey(),
                    &self.keypair.pubkey(),
                    &bench.payer.pubkey(),
                    amount,
                    &token.mint,
                )],
                Some(&[&self.keypair]),
            )
            .await
            .unwrap();
    }
    pub async fn deposit_governance_tag(
        &self,
        bench: &mut ProgramTestBench,
        tag_record_factory: &TestTagRecordFactory,
    ) {
        bench
            .process_transaction(
                &[deposit_governing_tag(
                    &lgovernance::id(),
                    &self.get_tag_record_address(tag_record_factory),
                    &self.keypair.pubkey(),
                    &tag_record_factory.factory,
                    &bench.payer.pubkey(),
                )],
                Some(&[&self.keypair]),
            )
            .await
            .unwrap();
    }

    pub fn get_vote_power_origin_record_address(&self, source: &VotePowerUnit) -> Pubkey {
        get_vote_power_origin_record_address(&lgovernance::id(), source, &self.keypair.pubkey()).0
    }

    pub async fn get_token_owner_record(
        &self,
        bench: &mut ProgramTestBench,
        source: &VotePowerUnit,
    ) -> VotePowerOwnerRecord {
        let address = self.get_vote_power_origin_record_address(source);
        let account = bench.get_account(&address).await.unwrap();

        try_from_slice_unchecked::<VotePowerOwnerRecord>(&account.data).unwrap()
    }

    pub fn get_token_owner_delegate_record_address(
        &self,
        scope: &Pubkey,
        source: &VotePowerUnit,
    ) -> Pubkey {
        get_vote_power_owner_record_address(
            &lgovernance::id(),
            source,
            &self.keypair.pubkey(),
            scope,
        )
        .0
    }

    pub async fn get_token_owner_delegate_record(
        &self,
        bench: &mut ProgramTestBench,
        scope: &Pubkey,
        source: &VotePowerUnit,
    ) -> VotePowerOwnerRecord {
        let address = self.get_token_owner_delegate_record_address(scope, source);
        let account = bench.get_account(&address).await.unwrap();

        try_from_slice_unchecked::<VotePowerOwnerRecord>(&account.data).unwrap()
    }

    pub fn get_token_owner_budget_record_address(
        &self,
        source: &VotePowerUnit,
        scope: &Pubkey,
    ) -> Pubkey {
        get_token_owner_budget_record_address(
            &lgovernance::id(),
            &self.get_vote_power_origin_record_address(source),
            scope,
        )
        .0
    }

    pub async fn get_token_owner_budget_record(
        &self,
        bench: &mut ProgramTestBench,
        source: &VotePowerUnit,
        scope: &Pubkey,
    ) -> Option<VotePowerOwnerRecord> {
        let address = self.get_token_owner_budget_record_address(source, scope);
        if let Some(account) = bench.get_account(&address).await {
            Some(try_from_slice_unchecked::<VotePowerOwnerRecord>(&account.data).unwrap())
        } else {
            None
        }
    }

    pub async fn get_vote_record_delegate(
        &self,
        bench: &mut ProgramTestBench,
        proposal: &TestProposal,
        source: &VotePowerUnit,
        scope: &Pubkey,
    ) -> Option<(Pubkey, VoteRecordV2)> {
        let address = get_vote_record_address(
            &lgovernance::id(),
            &proposal.proposal,
            &self.get_token_owner_delegate_record_address(scope, source),
            scope,
        )
        .0;
        if let Some(account) = bench.get_account(&address).await {
            Some((
                address,
                try_from_slice_unchecked::<VoteRecordV2>(&account.data).unwrap(),
            ))
        } else {
            None
        }
    }

    pub async fn get_latest_vote_address(
        &self,
        bench: &mut ProgramTestBench,
        source: &VotePowerUnit,
    ) -> Option<Pubkey> {
        let token_owner_record = self.get_token_owner_record(bench, source).await;
        token_owner_record.latest_vote
    }

    pub async fn get_latest_vote_delegate_address(
        &self,
        bench: &mut ProgramTestBench,
        source: &VotePowerUnit,
        scope: &Pubkey,
    ) -> Option<Pubkey> {
        let token_owner_record = self
            .get_token_owner_delegate_record(bench, scope, source)
            .await;
        token_owner_record.latest_vote
    }

    pub async fn create_budget(
        &self,
        bench: &mut ProgramTestBench,
        source: &VotePowerUnit,
        scope: &Pubkey,
    ) {
        bench
            .process_transaction(
                &[create_token_owner_budget_record(
                    &lgovernance::id(),
                    &bench.payer.pubkey(),
                    &self.get_vote_power_origin_record_address(source),
                    scope,
                )],
                None,
            )
            .await
            .unwrap();
    }

    pub async fn get_token_account(
        &self,
        banks_client: &mut BanksClient,
        mint: &Pubkey,
    ) -> spl_token::state::Account {
        spl_token::state::Account::unpack(
            banks_client
                .get_account(get_associated_token_address(&self.keypair.pubkey(), mint))
                .await
                .unwrap()
                .unwrap()
                .data
                .as_slice(),
        )
        .unwrap()
    }

    /* pub async fn delegate(bench: &mut ProgramTestBench, token: TestToken, amount: u64) {
        bench
            .process_transaction(&[delegate_], Some(&[&self.keypair]))
            .await
            .unwrap();
    } */
}

pub struct TestDelegation<'a> {
    from: &'a TestUser,
    to: &'a TestUser,
    source: &'a VotePowerUnit,
    scope: Pubkey,
    delegation: Pubkey,
}
impl<'a> TestDelegation<'a> {
    pub async fn new(
        bench: &mut ProgramTestBench,
        from: &'a TestUser,
        to: &'a TestUser,
        source: &'a VotePowerUnit,
        scope: &Pubkey,
    ) -> TestDelegation<'a> {
        let from_token_record = from.get_vote_power_origin_record_address(source);
        let to_token_record = to.get_token_owner_delegate_record_address(scope, source);
        // Create budget
        if from
            .get_token_owner_budget_record(bench, source, scope)
            .await
            .is_none()
        {
            from.create_budget(bench, source, scope).await;
        }

        // Enable the delegatee
        if bench
            .get_account(&to.get_token_owner_delegate_record_address(scope, source))
            .await
            .is_none()
        {
            to.create_delegatee(bench, source, scope).await;
        }
        TestDelegation {
            from,
            to,
            source,
            scope: *scope,
            delegation: get_scope_delegation_account_program_address(
                &lgovernance::id(),
                &from_token_record,
                &to_token_record,
                scope,
            )
            .0,
        }
    }
    pub async fn delegate(&self, bench: &mut ProgramTestBench, amount: &u64) {
        let from_token_record = self.get_delegator_token_origin_record_address();
        let to_token_record = self.get_delegatee_vote_power_owner_record_address();
        bench
            .process_transaction(
                &[delegate(
                    &lgovernance::id(),
                    &from_token_record,
                    &self
                        .from
                        .get_token_owner_budget_record_address(self.source, &self.scope),
                    &self.from.keypair.pubkey(),
                    &to_token_record,
                    &self.to.keypair.pubkey(),
                    &bench.payer.pubkey(),
                    amount,
                    &self.scope,
                )],
                Some(&[&self.from.keypair]),
            )
            .await
            .unwrap();
    }

    pub async fn undelegate(&self, bench: &mut ProgramTestBench, amount: &u64) {
        // Undelegate
        let from_token_record = self.get_delegator_token_origin_record_address();
        let to_token_record = self.get_delegatee_vote_power_owner_record_address();
        bench
            .process_transaction(
                &[undelegate(
                    &lgovernance::id(),
                    &get_scope_delegation_account_program_address(
                        &lgovernance::id(),
                        &from_token_record,
                        &to_token_record,
                        &self.scope,
                    )
                    .0,
                    &from_token_record,
                    &self
                        .from
                        .get_token_owner_budget_record_address(self.source, &self.scope),
                    &self.from.keypair.pubkey(),
                    &to_token_record,
                    &self.to.keypair.pubkey(),
                    &bench.payer.pubkey(),
                    amount,
                )],
                Some(&[&self.from.keypair]),
            )
            .await
            .unwrap();
    }

    pub async fn delegate_history(&self, bench: &mut ProgramTestBench) {
        let token_origin_record = self.get_delegator_token_origin_record_address();
        let delegation_record = self.get_delegation_record(bench).await.unwrap();
        let delegatee_token_owner_record = self.get_delegatee_vote_power_owner_record_address();
        let delegatee_token_governing_owner = self
            .get_delegatee_token_owner_record(bench)
            .await
            .unwrap()
            .governing_owner;

        // Pick the scope delegation vote head. If missing we have already delegated votes for all active proposals
        // (since we delegated before any votes where casted by the delegatee)
        let vote_record_address = if let Some(vote_head) = &delegation_record.last_vote_head {
            *vote_head
        } else {
            panic!("Unexpected");
        };
        let vote_record = try_from_slice_unchecked::<VoteRecordV2>(
            &bench.get_account(&vote_record_address).await.unwrap().data,
        )
        .unwrap();
        /*
               let previous_vote_record = try_from_slice_unchecked::<VoteRecordV2>(
                   &bench
                       .get_account(&vote_record.previous_vote.unwrap())
                       .await
                       .unwrap()
                       .data,
               )
               .unwrap();
        */
        let vote_options = TestProposal::get_vote_option_for_proposal(
            bench,
            &vote_record.proposal,
            &vote_record.vote,
        )
        .await;

        bench
            .process_transaction(
                &[delegate_history(
                    &lgovernance::id(),
                    &vote_record_address,
                    &vote_record.proposal,
                    &vote_options,
                    &self.delegation,
                    &token_origin_record,
                    &self.from.keypair.pubkey(),
                    &delegatee_token_owner_record,
                    &delegatee_token_governing_owner,
                    &self.scope,
                    true,
                )],
                Some(&[&self.from.keypair]),
            )
            .await
            .unwrap();
    }

    pub async fn undelegate_history(&self, bench: &mut ProgramTestBench) {
        let token_owner_record = self.get_delegator_token_origin_record_address();
        let delegatee_token_owner_record =
            self.get_delegatee_token_owner_record(bench).await.unwrap();

        let delegation_record = self.get_delegation_record(bench).await.unwrap();
        let delegatee_vote_power_owner_record_address =
            self.get_delegatee_vote_power_owner_record_address();
        let delegatee_token_governing_owner = delegatee_token_owner_record.governing_owner;

        // Pick the scope delegation vote head, or the first vote casted (that is relavent)
        let (vote_record_address, previous_vote_record_address) =
            if let Some(vote_head) = &delegation_record.vote_head {
                (*vote_head, None)
            } else if let Some(previous_vote_head) = &delegation_record.last_vote_head {
                let previous_vote_record = try_from_slice_unchecked::<VoteRecordV2>(
                    &bench.get_account(previous_vote_head).await.unwrap().data,
                )
                .unwrap();
                (
                    previous_vote_record.next_vote.unwrap(),
                    Some(*previous_vote_head),
                )
            } else {
                (delegatee_token_owner_record.first_vote.unwrap(), None)
            };

        let vote_record = try_from_slice_unchecked::<VoteRecordV2>(
            &bench.get_account(&vote_record_address).await.unwrap().data,
        )
        .unwrap();

        let vote_options = TestProposal::get_vote_option_for_proposal(
            bench,
            &vote_record.proposal,
            &vote_record.vote,
        )
        .await;
        bench
            .process_transaction(
                &[undelegate_history(
                    &lgovernance::id(),
                    &vote_record_address,
                    &vote_record.proposal,
                    &vote_options,
                    &self.delegation,
                    &token_owner_record,
                    &self.from.keypair.pubkey(),
                    &delegatee_vote_power_owner_record_address,
                    &delegatee_token_governing_owner,
                    &self.scope,
                    previous_vote_record_address.as_ref(),
                    true,
                )],
                Some(&[&self.from.keypair]),
            )
            .await
            .unwrap();
    }

    pub fn get_delegator_token_origin_record_address(&self) -> Pubkey {
        self.from.get_vote_power_origin_record_address(self.source)
    }
    pub async fn get_delegator_token_origin_record(
        &self,
        bench: &mut ProgramTestBench,
    ) -> Option<VotePowerOwnerRecord> {
        let account = bench
            .get_account(&self.get_delegator_token_origin_record_address())
            .await;
        if let Some(account) = account {
            Some(try_from_slice_unchecked::<VotePowerOwnerRecord>(&account.data).unwrap())
        } else {
            None
        }
    }

    pub fn get_delegatee_vote_power_owner_record_address(&self) -> Pubkey {
        self.to
            .get_token_owner_delegate_record_address(&self.scope, self.source)
    }

    pub async fn get_delegatee_token_owner_record(
        &self,
        bench: &mut ProgramTestBench,
    ) -> Option<VotePowerOwnerRecord> {
        let account = bench
            .get_account(&self.get_delegatee_vote_power_owner_record_address())
            .await;
        if let Some(account) = account {
            Some(try_from_slice_unchecked::<VotePowerOwnerRecord>(&account.data).unwrap())
        } else {
            None
        }
    }

    pub async fn get_delegation_record(
        &self,
        bench: &mut ProgramTestBench,
    ) -> Option<ScopeDelegationRecordAccount> {
        let account = bench.get_account(&self.delegation).await;
        if let Some(account) = account {
            Some(try_from_slice_unchecked::<ScopeDelegationRecordAccount>(&account.data).unwrap())
        } else {
            None
        }
    }
}
pub struct TestGovernance {
    pub governance: Pubkey,
    pub temporary_authority: Keypair,
    pub seed: Pubkey,
}
impl TestGovernance {
    pub async fn new(bench: &mut ProgramTestBench) -> TestGovernance {
        let temporary_authority = Keypair::new();
        let seed = Pubkey::new_unique();
        bench
            .process_transaction(
                &[create_governance(
                    &lgovernance::id(),
                    &seed,
                    &temporary_authority.pubkey(),
                    &bench.payer.pubkey(),
                )],
                None,
            )
            .await
            .unwrap();
        let governande_address = get_governance_address(&lgovernance::id(), &seed).0;
        Self {
            governance: governande_address,
            temporary_authority,
            seed,
        }
    }

    pub async fn update_governance_authority(
        &self,
        bench: &mut ProgramTestBench,
        new_authority: Option<Pubkey>,
    ) {
        let account = self.get_governance_account(bench).await;
        bench
            .process_transaction(
                &[update_governance_authority(
                    &lgovernance::id(),
                    &self.governance,
                    account.optional_authority.as_ref(),
                    new_authority,
                )],
                Some(&[&self.temporary_authority]),
            )
            .await
            .unwrap();
    }

    pub async fn create_scope(&self, bench: &mut ProgramTestBench, scope: ScopeConfig) -> Pubkey {
        let id = Pubkey::new_unique();
        let (scope_address, _create_scope_address_bump_seed) =
            get_scope_program_address(&lgovernance::id(), &id);

        bench
            .process_transaction(
                &[create_scope(
                    &lgovernance::id(),
                    &id,
                    &self.governance,
                    &bench.payer.pubkey(),
                    &Some(self.temporary_authority.pubkey()),
                    &scope,
                )],
                Some(&[&self.temporary_authority]),
            )
            .await
            .unwrap();
        scope_address
    }

    pub async fn create_scope_system<'b>(
        &self,
        bench: &mut ProgramTestBench,
        criteria_from: TestVotePowerSource<'b>,
    ) -> Pubkey {
        let scope = self
            .create_scope(
                bench,
                match criteria_from {
                    TestVotePowerSource::TestToken(test_token) => {
                        ScopeConfig::get_single_mint_config(
                            &test_token.mint,
                            &Some(ScopeMatch::ProgramId(system_program::id())),
                            &None,
                            &None,
                        )
                    }
                    TestVotePowerSource::TestTagRecordFactory(tag_record_factory) => {
                        ScopeConfig::get_single_tag_config(
                            &tag_record_factory.factory,
                            &Some(ScopeMatch::ProgramId(system_program::id())),
                            &None,
                            &None,
                        )
                    }
                },
            )
            .await;
        scope
    }

    pub async fn get_governance_account(&self, bench: &mut ProgramTestBench) -> GovernanceV2 {
        let account = bench.get_account(&self.governance).await.unwrap();

        try_from_slice_unchecked::<GovernanceV2>(&account.data).unwrap()
    }

    #[allow(dead_code)]
    pub async fn with_native_treasury(&mut self, bench: &mut ProgramTestBench) {
        let create_treasury_ix =
            create_native_treasury(&lgovernance::id(), &self.governance, &bench.payer.pubkey());

        let treasury_address = get_native_treasury_address(&lgovernance::id(), &self.governance);

        let transfer_ix =
            system_instruction::transfer(&bench.payer.pubkey(), &treasury_address, 1_000_000_000);

        bench
            .process_transaction(&[create_treasury_ix, transfer_ix], None)
            .await
            .unwrap();
    }
}
pub struct TestProposal {
    pub proposal: Pubkey,
    pub proposal_transactions: HashMap<u16, Vec<Pubkey>>,
    pub scopes: Vec<Pubkey>,
    pub options: Vec<Pubkey>,
    pub instruction_index: u16,
}

impl TestProposal {
    pub async fn new(
        bench: &mut ProgramTestBench,
        proposal_index: u64,
        vote_type: VoteType,
        scopes: Vec<Pubkey>,
        governance: &TestGovernance,
        owner: &Keypair,
    ) -> Self {
        let governance_data = governance.get_governance_account(bench).await;
        let (proposal_address, _proposal_address_bump_seed) = get_proposal_address(
            &lgovernance::id(),
            &governance.governance,
            &governance_data.proposals_count.to_le_bytes(),
        );
        let mut instructions = vec![create_proposal(
            &lgovernance::id(),
            &owner.pubkey(),
            &governance.governance,
            &bench.payer.pubkey(),
            proposal_index,
            vote_type,
            scopes.len() as u8,
            &ContentSource::String("Info".into()),
        )];
        for scope in &scopes {
            instructions.push(insert_scope(
                &lgovernance::id(),
                scope,
                &proposal_address,
                &owner.pubkey(),
                &bench.payer.pubkey(),
            ))
        }

        bench
            .process_transaction(&instructions, Some(&[owner]))
            .await
            .unwrap();

        Self {
            proposal: proposal_address,
            proposal_transactions: HashMap::new(),
            scopes,
            instruction_index: 0,
            options: Vec::new(),
        }
    }

    pub async fn new_transfer_proposal(
        bench: &mut ProgramTestBench,
        owner: &TestUser,
        scope: &Pubkey,
        governance: &TestGovernance,
        transfer_amount: u64,
    ) -> (TestProposal, WalletCookie) {
        let mut proposal = TestProposal::new(
            bench,
            0,
            VoteType::SingleChoice,
            vec![*scope],
            governance,
            &owner.keypair,
        )
        .await;

        proposal
            .add_option(bench, &CreateProposalOptionType::Deny, &owner.keypair)
            .await;

        let instruction_option = proposal
            .add_option(
                bench,
                &CreateProposalOptionType::Instruction("Label".into()),
                &owner.keypair,
            )
            .await;

        let proposal_option = try_from_slice_unchecked::<ProposalOption>(
            &bench.get_account(&instruction_option).await.unwrap().data,
        )
        .unwrap();

        let recipent_wallet = bench.with_wallet().await;

        // Transaction from native treasury

        proposal
            .add_transaction(
                bench,
                proposal_option.index,
                0,
                vec![ConditionedInstruction {
                    instruction_data: system_instruction::transfer(
                        &get_native_treasury_address(&lgovernance::id(), &governance.governance),
                        &recipent_wallet.address,
                        transfer_amount,
                    )
                    .into(),
                    scope: *scope,
                }],
                &owner.keypair,
            )
            .await;

        proposal
            .finalize_draft(bench, governance, &owner.keypair)
            .await;
        (proposal, recipent_wallet)
    }

    pub async fn add_option(
        &mut self,
        bench: &mut ProgramTestBench,
        option_type: &CreateProposalOptionType,
        owner: &Keypair,
    ) -> Pubkey {
        let proposal = self.get_proposal_account(bench).await;
        let instructions = [create_proposal_option(
            &lgovernance::id(),
            &owner.pubkey(),
            &bench.payer.pubkey(),
            &self.proposal,
            option_type,
            proposal.options_count,
        )];
        let option_address = get_proposal_option_program_address(
            &lgovernance::id(),
            &self.proposal,
            &proposal.options_count.to_le_bytes(),
        )
        .0;

        bench
            .process_transaction(&instructions, Some(&[owner]))
            .await
            .unwrap();

        self.options.push(option_address);
        option_address
    }

    pub async fn add_transaction(
        &mut self,
        bench: &mut ProgramTestBench,
        option_index: u16,
        hold_up_time: u32,
        instructions: Vec<ConditionedInstruction>,
        owner: &Keypair,
    ) {
        self.proposal_transactions
            .entry(option_index)
            .or_insert_with(Vec::new);
        let option_instructions = self.proposal_transactions.get_mut(&option_index).unwrap();
        let instructions = [insert_transaction(
            &lgovernance::id(),
            &bench.payer.pubkey(),
            &owner.pubkey(),
            &self.proposal,
            option_index,
            option_instructions.len() as u16,
            hold_up_time,
            instructions,
        )];

        let instruction_key = get_proposal_transaction_address(
            &lgovernance::id(),
            &self.proposal,
            &option_index.to_le_bytes(),
            &(option_instructions.len() as u16).to_le_bytes(),
        );
        option_instructions.push(instruction_key);

        bench
            .process_transaction(&instructions, Some(&[owner]))
            .await
            .unwrap();
    }

    pub async fn finalize_draft(
        &self,
        bench: &mut ProgramTestBench,
        governance: &TestGovernance,
        owner: &Keypair,
    ) {
        let mut scope_accounts = Vec::new();
        for scope in &self.scopes {
            let account = bench.get_account(scope).await.unwrap();
            let scope_data = try_from_slice_unchecked::<Scope>(&account.data).unwrap();
            scope_accounts.push(scope_data)
        }
        let mut signed_scopes = Vec::new();
        for (scope, scope_data) in self.scopes.iter().zip(scope_accounts) {
            signed_scopes.push((
                *scope,
                match scope_data.config.proposal_config.create_proposal_criteria {
                    lgovernance::state::scopes::scope::CreateProposalCriteria::Tag {
                        record_factory,
                    } => {
                        let tag_record = get_tag_record_program_address(
                            &ltag::id(),
                            &record_factory,
                            &owner.pubkey(),
                        )
                        .0;

                        SignedCreateProposal::Tag {
                            owner: owner.pubkey(),
                            record: tag_record,
                        }
                    }
                    lgovernance::state::scopes::scope::CreateProposalCriteria::Token {
                        mint,
                        ..
                    } => {
                        let record = get_vote_power_owner_record_address(
                            &lgovernance::id(),
                            &VotePowerUnit::Mint(mint),
                            &owner.pubkey(),
                            scope,
                        )
                        .0;
                        SignedCreateProposal::Token {
                            governing_owner: owner.pubkey(),
                            owner_record: record,
                        }
                    }
                },
            ))
        }

        let instructions = [finalize_draft(
            &lgovernance::id(),
            &owner.pubkey(),
            &self.proposal,
            &governance.governance,
            &signed_scopes,
        )];

        bench
            .process_transaction(&instructions, Some(&[owner]))
            .await
            .unwrap();
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
    /* pub async fn get_proposal_used_scopes(&self, banks_client: &mut BanksClient) {
        let x = self.proposal_transactions.iter().map(|id| async {
            let transaction: ProposalTransactionV2 = try_from_slice_unchecked(
                &banks_client.get_account(*id).await.unwrap().unwrap().data,
            )
            .unwrap();
            return transaction.get_used_scopes();
        });

        if let PostType::Proposal(proposal) = post.post_type {
            banks_client.get_account(address)
        }
        return None;
    } */

    pub async fn get_vote_option(&self, _bench: &mut ProgramTestBench, vote: &Vote) -> Vec<Pubkey> {
        let mut vote_options = Vec::new();
        for index in vote {
            let option = get_proposal_option_program_address(
                &lgovernance::id(),
                &self.proposal,
                &index.to_le_bytes(),
            )
            .0;
            vote_options.push(option);
        }
        vote_options
    }

    pub async fn get_vote_option_for_proposal(
        _bench: &mut ProgramTestBench,
        proposal: &Pubkey,
        vote: &Vote,
    ) -> Vec<Pubkey> {
        let mut vote_options = Vec::new();
        for index in vote {
            let option = get_proposal_option_program_address(
                &lgovernance::id(),
                proposal,
                &index.to_le_bytes(),
            )
            .0;
            vote_options.push(option);
        }
        vote_options
    }

    /*   pub async fn vote(
           &self,
           bench: &mut ProgramTestBench,
           vote: &Vote,
           owner: &TestUser,
           source: &VoteSource,
           scope: &Pubkey,
       ) {
           if owner
               .get_token_owner_budget_record(bench, source, scope)
               .await
               .is_none()
           {
               owner.create_budget(bench, source, scope).await;
           }

           let vote_options = self.get_vote_option(bench, vote).await;
           let latest_vote = owner.get_latest_vote_address(bench, source).await;
           bench
               .process_transaction(
                   &[cast_vote(
                       &lgovernance::id(),
                       &bench.payer.pubkey(),
                       &self.proposal,
                       &owner.get_vote_power_origin_record_address(source),
                       &owner.keypair.pubkey(),
                       scope,
                       &vote_options,
                       latest_vote.as_ref(),
                       false,
                   )],
                   Some(&[&owner.keypair]),
               )
               .await
               .unwrap();
       }
    */
    pub async fn vote_with_delegate(
        &self,
        bench: &mut ProgramTestBench,
        vote: &Vote,
        owner: &TestUser,
        source: &VotePowerUnit,
        scope: &Pubkey,
    ) {
        let vote_options = self.get_vote_option(bench, vote).await;
        let latest_vote = owner
            .get_latest_vote_delegate_address(bench, source, scope)
            .await;

        bench
            .process_transaction(
                &[cast_vote(
                    &lgovernance::id(),
                    &bench.payer.pubkey(),
                    &self.proposal,
                    &owner.get_token_owner_delegate_record_address(scope, source),
                    &owner.keypair.pubkey(),
                    scope,
                    &vote_options,
                    latest_vote.as_ref(),
                    true,
                )],
                Some(&[&owner.keypair]),
            )
            .await
            .unwrap();
    }
    pub async fn unvote_with_delegate(
        &self,
        bench: &mut ProgramTestBench,
        vote: Vote,
        owner: &TestUser,
        source: &VotePowerUnit,
        scope: &Pubkey,
        beneficiary: &Pubkey,
    ) {
        let mut vote_options = Vec::new();
        for index in &vote {
            let option = get_proposal_option_program_address(
                &lgovernance::id(),
                &self.proposal,
                &index.to_le_bytes(),
            )
            .0;
            vote_options.push(option);
        }

        bench
            .process_transaction(
                &[uncast_vote(
                    &lgovernance::id(),
                    &self.proposal,
                    &owner.get_token_owner_delegate_record_address(scope, source),
                    &owner.keypair.pubkey(),
                    beneficiary,
                    scope,
                    &vote_options,
                )],
                Some(&[&owner.keypair]),
            )
            .await
            .unwrap();
    }
    /*
    pub async fn unvote(
        &self,
        bench: &mut ProgramTestBench,
        vote: Vote,
        owner: &TestUser,
        token: &TestToken,
        scope: &Pubkey,
        beneficiary: &Pubkey,
    ) {
        let mut vote_options = Vec::new();
        for index in &vote {
            let option = get_proposal_option_program_address(
                &lgovernance::id(),
                &self.proposal,
                &index.to_le_bytes(),
            )
            .0;
            vote_options.push(option);
        }

        bench
            .process_transaction(
                &[uncast_vote(
                    &lgovernance::id(),
                    &self.proposal,
                    &owner.get_vote_power_origin_record_address(&token.mint),
                    &owner.keypair.pubkey(),
                    beneficiary,
                    scope,
                    &vote_options,
                )],
                Some(&[&owner.keypair]),
            )
            .await
            .unwrap();
    }
    */

    pub async fn execute_transactions(
        &self,
        bench: &mut ProgramTestBench,
        option_index: u16,
        /*  instruction_accounts: &[AccountMeta], */
    ) {
        let governance = self.get_proposal_account(bench).await.governance;
        let governance_account = try_from_slice_unchecked::<GovernanceV2>(
            &bench.get_account(&governance).await.unwrap().data,
        )
        .unwrap();
        let native_treasury = get_native_treasury_address(&lgovernance::id(), &governance);
        let transactions = self.proposal_transactions.get(&option_index).unwrap();
        for (i, transaction) in transactions.iter().enumerate() {
            let proposal_transaction_data = try_from_slice_unchecked::<ProposalTransactionV2>(
                &bench
                    .get_account(&get_proposal_transaction_address(
                        &lgovernance::id(),
                        &self.proposal,
                        &option_index.to_le_bytes(),
                        &(i as u16).to_le_bytes(),
                    ))
                    .await
                    .unwrap()
                    .data,
            )
            .unwrap();
            let mut program_ids = proposal_transaction_data
                .instructions
                .iter()
                .map(|i| i.instruction_data.program_id)
                .collect::<HashSet<_>>()
                .into_iter()
                .map(|key| AccountMeta::new_readonly(key, false))
                .collect::<Vec<AccountMeta>>();

            let mut account_metas = proposal_transaction_data
                .instructions
                .iter()
                .map(|i| &i.instruction_data.accounts)
                .flatten()
                .map(|a| AccountMeta {
                    pubkey: a.pubkey,
                    is_signer: a.is_signer && a.pubkey != governance && a.pubkey != native_treasury, // we can not sign if program owned acc (because pda)
                    is_writable: a.is_writable,
                })
                .collect::<Vec<AccountMeta>>();
            account_metas.append(&mut program_ids);

            bench
                .process_transaction(
                    &[execute_transaction(
                        &lgovernance::id(),
                        &self.proposal,
                        transaction,
                        &get_proposal_option_program_address(
                            &lgovernance::id(),
                            &self.proposal,
                            &option_index.to_le_bytes(),
                        )
                        .0,
                        governance_account.seed,
                        &account_metas, // ?????
                    )],
                    None,
                )
                .await
                .unwrap();
        }
    }

    pub async fn count_votes(&self, bench: &mut ProgramTestBench) {
        let mut scope_sources = Vec::new();
        let proposal = self.get_proposal_account(bench).await;
        for scope_weight in &proposal.scopes_max_vote_weight {
            let scope = try_from_slice_unchecked::<Scope>(
                &bench.get_account(&scope_weight.scope).await.unwrap().data,
            )
            .unwrap();
            scope_sources.push((
                scope_weight.scope,
                scope
                    .config
                    .vote_config
                    .source_weights
                    .iter()
                    .map(|weight| weight.source.clone())
                    .collect::<Vec<VotePowerUnit>>(),
            ))
        }

        // Count votes for all options
        let mut instructions = vec![count_vote_max_weights(
            &lgovernance::id(),
            &self.proposal,
            &scope_sources,
        )];

        for option in &self.options {
            instructions.push(count_votes(
                &lgovernance::id(),
                &self.proposal,
                option,
                proposal.deny_option.as_ref(),
                &self.scopes,
            ));
        }

        bench
            .process_transaction(&instructions, None)
            .await
            .unwrap();
    }

    pub async fn advance_clock_past_max_hold_up_time(
        &self,
        bench: &mut ProgramTestBench,
        option: u16,
    ) {
        let mut max_holdup_time = 0;
        for key in &self.proposal_transactions[&option] {
            let transaction = try_from_slice_unchecked::<ProposalTransactionV2>(
                &bench.get_account(key).await.unwrap().data,
            )
            .unwrap();
            max_holdup_time = max_holdup_time.max(transaction.hold_up_time);
        }
        let current_time = bench.get_clock().await;
        bench
            .advance_clock_past_timestamp(
                current_time.unix_timestamp + 1_i64 + max_holdup_time as i64,
            )
            .await;
    }

    pub async fn get_state(&self, bench: &mut ProgramTestBench) -> ProposalState {
        let proposal = self.get_proposal_account(bench).await;
        proposal.state
    }
    pub async fn get_proposal_account(&self, bench: &mut ProgramTestBench) -> ProposalV2 {
        try_from_slice_unchecked::<ProposalV2>(
            &*bench.get_account(&self.proposal).await.unwrap().data,
        )
        .unwrap()
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

pub struct TestToken {
    pub mint: Pubkey,
    pub token_holder_account: Pubkey,
    pub authority: Keypair,
}
impl TestToken {
    pub async fn new(bench: &mut ProgramTestBench) -> Self {
        // Create mint
        let authority = Keypair::new();
        let mint = Keypair::new();
        bench.create_mint(&mint, &authority.pubkey(), None).await;

        let (token_holder_account, _) =
            get_realm_mint_program_address(&lgovernance::id(), &mint.pubkey());

        Self {
            mint: mint.pubkey(),
            token_holder_account,
            authority,
        }
    }

    pub async fn create_token_holder_account(&self, bench: &mut ProgramTestBench) {
        create_token_holder_account(bench, &self.mint).await;
    }

    pub async fn mint_to(&self, bench: &mut ProgramTestBench, amount: u64, to: &Pubkey) {
        /*  self.create_associated_token_account(to, banks_client, payer, recent_blockhash)
                   .await;
        */

        bench
            .process_transaction(
                &[spl_token::instruction::mint_to(
                    &spl_token::id(),
                    &self.mint,
                    /* &get_associated_token_address(to, &self.mint), */
                    to,
                    &self.authority.pubkey(),
                    &[&self.authority.pubkey()],
                    amount,
                )
                .unwrap()],
                Some(&[&self.authority]),
            )
            .await
            .unwrap();
    }

    /* pub async fn create_empty_token_account(
        &self,
        bench: &mut ProgramTestBench,
        token_account_keypair: &Keypair,
        owner: &Pubkey,
    ) -> Pubkey {
        let rent = banks_client.get_rent().await.unwrap();
        let create_account_instruction = system_instruction::create_account(
            &bench.payer.pubkey(),
            &token_account_keypair.pubkey(),
            rent.minimum_balance(spl_token::state::Account::get_packed_len()),
            spl_token::state::Account::get_packed_len() as u64,
            &spl_token::id(),
        );

        let initialize_account_instruction = spl_token::instruction::initialize_account(
            &spl_token::id(),
            &token_account_keypair.pubkey(),
            &self.mint,
            owner,
        )
        .unwrap();
        let mut tx = Transaction::new_with_payer(
            &[create_account_instruction, initialize_account_instruction],
            Some(&bench.payer.pubkey()),
        );
        tx.sign(&[payer, token_account_keypair], *recent_blockhash);
        banks_client.process_transaction(tx).await.unwrap();
        token_account_keypair.pubkey()
    } */
}
/*
pub struct TestChannel {
    pub channel: Pubkey,
    pub authority: Keypair,
}

impl TestChannel {
    pub async fn new(
        bench: &mut ProgramTestBench,
        creator_user: &TestUser,
        authority: Keypair,
    ) -> Self {
        let channel_name = Pubkey::new_unique().to_string();
        let channel = create_and_verify_channel(
            bench,
            channel_name.as_ref(),
            &creator_user.keypair,
            &authority,
            None,
        )
        .await
        .unwrap();

        Self { channel, authority }
    }

    pub async fn get_channel_account(&self, banks_client: &mut BanksClient) -> ChannelAccount {
        let channel = try_from_slice_unchecked::<ChannelAccount>(
            &*banks_client
                .get_account(self.channel)
                .await
                .unwrap()
                .unwrap()
                .data,
        )
        .unwrap();
        channel
    }

    pub fn get_treasury_address(&self, mint: &Pubkey) -> Pubkey {
        find_treasury_token_account_address(
            &lgovernance::id(),
            &self.channel,
            mint,
            &spl_token::id(),
        )
        .0
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
 */
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
