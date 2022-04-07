use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, BorshSchema, PartialEq)]
pub enum AccountType {
    Channel,
    Post,
    VoteRecord,
    ChannelAuthority,
}
