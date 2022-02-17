use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};

/// Enum representing the account type managed by the program
#[derive(Clone, Debug, PartialEq, BorshDeserialize, BorshSerialize, BorshSchema)]
pub enum AccountType {
    /// User account
    UserAccount,
    /// Stake pool
    ChannelAccount,
    /// Post account
    PostAccount,
    /// Rule account
    RuleAccount,
}

/* impl MaxSize for AccountContainer {
    fn get_max_size(&self) -> Option<usize> {
        match self {
            AccountContainer::UserAccount(user) => user.get_max_size(),
            AccountContainer::ChannelAccount(channel) => channel.get_max_size(),
            AccountContainer::PostAccount(post) => post.get_max_size(),
        }
    }
}
 */
