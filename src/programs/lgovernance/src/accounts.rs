use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, BorshSchema, PartialEq)]
pub enum AccountType {
    Proposal,
    Governance,
    Scope,
    ScopeVoteWeight,
    Transaction,
    VoteRecordV2,
    TokenOwnerRecordV2,
    TokenOwnerBudgetRecord,
    DelegationRecord,
    ProposalVoteWeight,
    ProposalOption,
}
