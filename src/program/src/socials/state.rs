use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};

use super::{
    channel::state::ChannelAccount, post::state::PostAccount, user::state::UserAccount, MaxSize,
};

/// Used to prefix accounts
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, BorshSchema, PartialEq)]

pub enum AccountContainer {
    UserAccount(UserAccount),
    ChannelAccount(ChannelAccount),
    PostAccount(PostAccount),
}

impl MaxSize for AccountContainer {
    fn get_max_size(&self) -> Option<usize> {
        match self {
            AccountContainer::UserAccount(user) => user.get_max_size(),
            AccountContainer::ChannelAccount(channel) => channel.get_max_size(),
            AccountContainer::PostAccount(post) => post.get_max_size(),
        }
    }
}
