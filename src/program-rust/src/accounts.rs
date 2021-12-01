use std::io::Error;
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{account_info::{  AccountInfo}, borsh::try_from_slice_unchecked, entrypoint, entrypoint::ProgramResult, msg, program_error::ProgramError, pubkey::{Pubkey}, rent::Rent, sysvar::Sysvar};

use crate::account::{MaxSize};

pub static NULL_KEY: Pubkey = Pubkey::new_from_array([0_u8; 32]);

pub static MESSAGE_TRANSACTION_MAX_SIZE: usize = 1200;

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, PartialEq)]
pub enum Description
{
    String(String)
}

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct ChannelAccount {
    pub owner: Pubkey,
    pub name: String,
    pub description: Description
}



impl ChannelAccount {
    pub fn new(owner: Pubkey, name: String, description: Description) -> ChannelAccount {
        ChannelAccount {
            owner,
            name,
            description
        }
    }
}

impl MaxSize for ChannelAccount {
    fn get_max_size(&self) -> Option<usize> {
        None
    }
}



#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, PartialEq)]
pub enum Message
{
    String(String),
    // image
    // videos
    // files etc
}


#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct UserAccount {

    pub owner: Pubkey,
    pub name: String
    
}

impl MaxSize for UserAccount {
    fn get_max_size(&self) -> Option<usize> {
        None
    }
}




/* 
pub type MessageAccountSplitted = (MessageAccount, Vec<String>);
pub enum MessageAccountSubmittable
{
    Split(MessageAccountSplitted),
    Single(MessageAccount)
}
 */

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct MessageAccount {

    pub user: Pubkey,
    pub channel: Pubkey,
    pub timestamp: u64,
    pub message: Message,

    #[borsh_skip]
    pub size: u64

}


impl MessageAccount 
{

    pub fn new(user:Pubkey, channel:Pubkey, timestamp:u64,  message:Message ) -> Self 
    {
        match &message
        {       
             Message::String(string) => 
             {
                let message_size = string.as_bytes().len() as u64 + 4; // +4 because Borsh encodes length
                Self {
                    timestamp,
                    channel,
                    user,
                    message,
                    size:message_size
                }
             }
        }
    }
}

impl MaxSize for MessageAccount 
{
    fn get_max_size(&self) -> Option<usize> {
       return None
    }
}




// Used to serialization and deserialization to keep track of account types
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, PartialEq)]

pub enum AccountContainer {

    UserAccount(UserAccount),
    ChannelAccount(ChannelAccount),
    MessageAccount(MessageAccount)
}

impl MaxSize for AccountContainer 
{
    fn get_max_size(&self) -> Option<usize> {
        return None
     }
}

// Ugly helper methods below to reduce some bloat

pub fn deserialize_user_account(data:&[u8]) -> UserAccount
{
    
    if let AccountContainer::UserAccount(account) = try_from_slice_unchecked(&data).unwrap()
    {
        return account
    }
    panic!();
}

pub fn deserialize_channel_account(data:&[u8]) -> ChannelAccount
{
    
    if let AccountContainer::ChannelAccount(account) = try_from_slice_unchecked(&data).unwrap()
    {
        return account
    }
    panic!();
}


pub fn deserialize_message_account(data:&[u8]) -> MessageAccount
{
    
    if let AccountContainer::MessageAccount(account) = try_from_slice_unchecked(&data).unwrap()
    {
        return account
    }
    panic!();
}