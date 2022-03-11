use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};
use solana_program::pubkey::Pubkey;

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, BorshSchema, PartialEq)]
pub enum ContentSource {
    External { url: String },
    String(String),
}

impl From<&str> for ContentSource {
    fn from(string: &str) -> Self {
        return ContentSource::String(string.to_string());
    }
}
