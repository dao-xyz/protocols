
use std::io::Error;

use account::AccountMaxSize;
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{ next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    pubkey::{Pubkey},
    rent::Rent,
    sysvar::Sysvar,
};

use crate::{
    account::create_and_serialize_account_signed,
};

pub static NULL_KEY: Pubkey = Pubkey::new_from_array([0_u8; 32]);

mod account;
pub mod address;
mod error;

pub static MESSAGE_TRANSACTION_MAX_SIZE: usize = 1200;

/// Trait for accounts to return their max size

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct ChannelAccount {
    pub name: String,
    pub tail_message: Pubkey,
}

impl ChannelAccount {
    pub fn new(name: String) -> ChannelAccount {
        ChannelAccount {
            name,
            tail_message: NULL_KEY, // tail_message: None
        }
    }
}

impl AccountMaxSize for ChannelAccount {
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
pub struct MessageAccount {
    pub from: Pubkey,
    pub message: Message,
    #[borsh_skip]
    pub size: u64,
    //pub parts: u64,
    pub next: Pubkey // Next message in the channel
}

pub type MessageAccountSplitted = (MessageAccount, Vec<String>);
pub enum MessageAccountSubmittable
{
    Split(MessageAccountSplitted),
    Single(MessageAccount)
}

impl MessageAccount 
{
    /* 
    // Is this message too large for one transaction?
    pub fn must_split(&self) -> bool {
        self.get_max_size().unwrap() > MESSAGE_TRANSACTION_MAX_SIZE
    } */
    
    // A generic message that might have to be split into multiple transactions
    pub fn new(message:Message, from:Pubkey) -> Self 
    {
        match &message
        {       
             Message::String(string) => 
             {
                let message_size = string.as_bytes().len() as u64 + 4; // +4 because Borsh encodes length
                Self {
                    from,
                    message,
                    size:message_size,
                  //  parts: 1,
                    next: NULL_KEY
                }
             }
        }
        
    }

    /* pub fn to_submittable(mut self) -> MessageAccountSubmittable {
        // Return 
        // 1. MessageAccount, but modified with partial message
        // 2. 3. 4... parts of just strings
        if self.must_split()
        {
            let num_of_chunks = self.message.as_bytes().len() as f64 /  MESSAGE_TRANSACTION_MAX_SIZE as f64 ;
            let mut strings = self.message
                .chars()
                .chunks(num_of_chunks.ceil() as usize )
                .into_iter()
                .map(|chunk| chunk.collect::<String>())
                .collect::<Vec<String>>();
            self.message = strings.remove(0);

            return MessageAccountSubmittable::Split(
                (
                    self,
                    strings
                )
            )
        }
        return MessageAccountSubmittable::Single(self)
    } */


}

impl AccountMaxSize for MessageAccount 
{
    fn get_max_size(&self) -> Option<usize> {
        // we calcualte this manually since, the max size of this MessageAccount might be greate
        // than what it currently contains in the "message"
        let message_size_borsh = self.size + 1; // 1 byte for the enum 
        let keys_size = 64;
        let message_size_size = 8;
        let parts_sizes = 8;
        Some((message_size_borsh + keys_size + message_size_size + parts_sizes) as usize)
    }
}

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct SubmitMessage {
    pub from: Pubkey
}

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, PartialEq)]
pub enum ChatInstruction {
    // Create channel, that keep tracks of the message tail
    CreateChannel(ChannelAccount),

    // Update channel (the tail message)
    UpdateChannel(ChannelAccount),

    // Message builder is user to build a message that later can be submitted with the submitt message instruction
    SendMessage(MessageAccount),

    // Add message to message builder
    //BuildMessagePart(String),

    // Submit message from BuildMessage invocations
    //SubmitMessage,
}

/*

/// Define the type of state stored in accounts
#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct GreetingAccount {
    /// number of greetings
    pub counter: u32,
}
*/
// Declare and export the program's entrypoint
entrypoint!(process_instruction);

// Program entrypoint's implementation
pub fn process_instruction(
    program_id: &Pubkey,      // Public key of the account the program was loaded into
    accounts: &[AccountInfo], // The account to say hello to
    input: &[u8],
) -> ProgramResult {
    msg!("Chat program entrypoint!");

    let instruction = ChatInstruction::try_from_slice(input)?;

    // Iterating accounts is safer then indexing
    let accounts_iter = &mut accounts.iter();

    let system_account = next_account_info(accounts_iter)?;
    let _program_account = next_account_info(accounts_iter)?;
    let payer_account = next_account_info(accounts_iter)?;

    match instruction {
        ChatInstruction::CreateChannel(channel) => {
            let channel_account_info = next_account_info(accounts_iter)?;

            /* msg!("CREATE CHANNEL ACCOUNT ");
            msg!(channel.try_to_vec().unwrap().len().to_string().as_str()); */
            let rent = Rent::get()?;
            create_and_serialize_account_signed(
                payer_account,
                channel_account_info,
                &channel,
                &[channel.name.as_bytes()],
                program_id,
                system_account,
                &rent,
            )?;
        }
        ChatInstruction::UpdateChannel(channel) => {
            let channel_account_info = next_account_info(accounts_iter)?;

            // Don't allow channel name to be updated, since it would require us to resize the account size
            // This would also mean that the PDA would change!
            channel.serialize(&mut *channel_account_info.data.borrow_mut())?
        }

        ChatInstruction::SendMessage(mut message_account) => {
            // Initializes an account for us that lets us build an message
            let message_account_info = next_account_info(accounts_iter)?;
            let rent = Rent::get()?;

            if true // Assume that messages only contain 1 part for now
            {
                // we have recieved the whole message, let modify it for submission
                let channel_account_info = next_account_info(accounts_iter)?;

                 // Load accounts
                let mut channel_account = ChannelAccount::try_from_slice(&channel_account_info.data.borrow())?;
                replace_tail_message(&mut message_account, message_account_info.key, &mut channel_account);

                // serialize/save the channel account, 
                channel_account.serialize(&mut *channel_account_info.data.borrow_mut())?;

                // message account will be saved below
            }


            create_and_serialize_account_signed(
                payer_account,
                message_account_info,
                &message_account,
                &[&payer_account.key.to_bytes()],
                program_id,
                system_account,
                &rent,
            )?;

           
        }
        /* ChatInstruction::SubmitMessage =>
        {

            let message_account_info = next_account_info(accounts_iter)?;
            let channel_account_info = next_account_info(accounts_iter)?;
            submit_message(&message_account_info, &channel_account_info)?;
        } */
        
        
        /* ChatInstruction::SubmitMessage(submittable) =>
          {
               // Iterating accounts is safer then indexing
              let accounts_iter = &mut accounts.iter();

              // Get the channeel account
              let channel_info = next_account_info(accounts_iter)?;

              // The account must be owned by the program in order to modify its data
              if channel_info.owner != program_id {
                  msg!("Channnel account does not have the correct program id");
                  return Err(ProgramError::IncorrectProgramId);
              }

              msg!("Send  message channel!");

              let channel_account_metas = vec![AccountMeta::new(channel_info.key.clone(), false)];
              invoke(&Instruction::new_with_bincode(program_id.clone(), &Message {
                  message: message,
                  next: None // should bee previorus
              }.try_to_vec()?,
              channel_account_metas), accounts)?;

              msg!("Message sent to channel!");

          } */
    }

    Ok(())
}

pub fn replace_tail_message(message_account: &mut MessageAccount, messsage_account_key:&Pubkey, channel_account: &mut  ChannelAccount) {
    
    // Last message is newest message "next"
    message_account.next = channel_account.tail_message;

    // Replace last message with newset message
    channel_account.tail_message = *messsage_account_key;
            
    
}

pub fn submit_message(message_account_info: &AccountInfo, channel_account_info: &AccountInfo) -> Result<(),Error> {
    

    // Load accounts
    let mut channel_account = ChannelAccount::try_from_slice(&channel_account_info.data.borrow())?;
    let mut message_account = MessageAccount::try_from_slice(&message_account_info.data.borrow())?;

    replace_tail_message(&mut message_account,message_account_info.key, &mut channel_account);
    // Save message and channel
    message_account.serialize(&mut *message_account_info.data.borrow_mut())?;
    channel_account.serialize(&mut *channel_account_info.data.borrow_mut())?;
                
    Ok(())
    
}


/*
const ORGANIZATION_LEN:usize = 1000;
impl Sealed for OrganizationAccount {}
impl Pack for OrganizationAccount {
    const LEN: usize = ORGANIZATION_LEN;

    fn pack_into_slice(&self, dst: &mut [u8]) {
        let output = array_mut_ref![dst, 0, ORGANIZATION_LEN];
        #[allow(clippy::ptr_offset_with_cast)]
        let (
            version,
            last_update_slot,
            last_update_stale,
            lending_market,
            owner,
            deposited_value,
            borrowed_value,
            allowed_borrow_value,
            unhealthy_borrow_value,
            deposits_len,
            borrows_len,
            data_flat,
        ) = mut_array_refs![
            output,
            1,
            8,
            1,
            PUBKEY_BYTES,
            PUBKEY_BYTES,
            16,
            16,
            16,
            16,
            1,
            1,
            OBLIGATION_COLLATERAL_LEN + (OBLIGATION_LIQUIDITY_LEN * (MAX_OBLIGATION_RESERVES - 1))
        ];

        // obligation
        *version = self.version.to_le_bytes();
        *last_update_slot = self.last_update.slot.to_le_bytes();
        pack_bool(self.last_update.stale, last_update_stale);
        lending_market.copy_from_slice(self.lending_market.as_ref());
        owner.copy_from_slice(self.owner.as_ref());
        pack_decimal(self.deposited_value, deposited_value);
        pack_decimal(self.borrowed_value, borrowed_value);
        pack_decimal(self.allowed_borrow_value, allowed_borrow_value);
        pack_decimal(self.unhealthy_borrow_value, unhealthy_borrow_value);
        *deposits_len = u8::try_from(self.deposits.len()).unwrap().to_le_bytes();
        *borrows_len = u8::try_from(self.borrows.len()).unwrap().to_le_bytes();

        let mut offset = 0;

        // deposits
        for collateral in &self.deposits {
            let deposits_flat = array_mut_ref![data_flat, offset, OBLIGATION_COLLATERAL_LEN];
            #[allow(clippy::ptr_offset_with_cast)]
            let (deposit_reserve, deposited_amount, market_value) =
                mut_array_refs![deposits_flat, PUBKEY_BYTES, 8, 16];
            deposit_reserve.copy_from_slice(collateral.deposit_reserve.as_ref());
            *deposited_amount = collateral.deposited_amount.to_le_bytes();
            pack_decimal(collateral.market_value, market_value);
            offset += OBLIGATION_COLLATERAL_LEN;
        }

        // borrows
        for liquidity in &self.borrows {
            let borrows_flat = array_mut_ref![data_flat, offset, OBLIGATION_LIQUIDITY_LEN];
            #[allow(clippy::ptr_offset_with_cast)]
            let (borrow_reserve, cumulative_borrow_rate_wads, borrowed_amount_wads, market_value) =
                mut_array_refs![borrows_flat, PUBKEY_BYTES, 16, 16, 16];
            borrow_reserve.copy_from_slice(liquidity.borrow_reserve.as_ref());
            pack_decimal(
                liquidity.cumulative_borrow_rate_wads,
                cumulative_borrow_rate_wads,
            );
            pack_decimal(liquidity.borrowed_amount_wads, borrowed_amount_wads);
            pack_decimal(liquidity.market_value, market_value);
            offset += OBLIGATION_LIQUIDITY_LEN;
        }
    }

    /// Unpacks a byte buffer into an [ObligationInfo](struct.ObligationInfo.html).
    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let input = array_ref![src, 0, OBLIGATION_LEN];
        #[allow(clippy::ptr_offset_with_cast)]
        let (
            version,
            last_update_slot,
            last_update_stale,
            lending_market,
            owner,
            deposited_value,
            borrowed_value,
            allowed_borrow_value,
            unhealthy_borrow_value,
            deposits_len,
            borrows_len,
            data_flat,
        ) = array_refs![
            input,
            1,
            8,
            1,
            PUBKEY_BYTES,
            PUBKEY_BYTES,
            16,
            16,
            16,
            16,
            1,
            1,
            OBLIGATION_COLLATERAL_LEN + (OBLIGATION_LIQUIDITY_LEN * (MAX_OBLIGATION_RESERVES - 1))
        ];

        let version = u8::from_le_bytes(*version);
        if version > PROGRAM_VERSION {
            msg!("Obligation version does not match lending program version");
            return Err(ProgramError::InvalidAccountData);
        }

        let deposits_len = u8::from_le_bytes(*deposits_len);
        let borrows_len = u8::from_le_bytes(*borrows_len);
        let mut deposits = Vec::with_capacity(deposits_len as usize + 1);
        let mut borrows = Vec::with_capacity(borrows_len as usize + 1);

        let mut offset = 0;
        for _ in 0..deposits_len {
            let deposits_flat = array_ref![data_flat, offset, OBLIGATION_COLLATERAL_LEN];
            #[allow(clippy::ptr_offset_with_cast)]
            let (deposit_reserve, deposited_amount, market_value) =
                array_refs![deposits_flat, PUBKEY_BYTES, 8, 16];
            deposits.push(ObligationCollateral {
                deposit_reserve: Pubkey::new(deposit_reserve),
                deposited_amount: u64::from_le_bytes(*deposited_amount),
                market_value: unpack_decimal(market_value),
            });
            offset += OBLIGATION_COLLATERAL_LEN;
        }
        for _ in 0..borrows_len {
            let borrows_flat = array_ref![data_flat, offset, OBLIGATION_LIQUIDITY_LEN];
            #[allow(clippy::ptr_offset_with_cast)]
            let (borrow_reserve, cumulative_borrow_rate_wads, borrowed_amount_wads, market_value) =
                array_refs![borrows_flat, PUBKEY_BYTES, 16, 16, 16];
            borrows.push(ObligationLiquidity {
                borrow_reserve: Pubkey::new(borrow_reserve),
                cumulative_borrow_rate_wads: unpack_decimal(cumulative_borrow_rate_wads),
                borrowed_amount_wads: unpack_decimal(borrowed_amount_wads),
                market_value: unpack_decimal(market_value),
            });
            offset += OBLIGATION_LIQUIDITY_LEN;
        }

        Ok(Self {
            version,
            last_update: LastUpdate {
                slot: u64::from_le_bytes(*last_update_slot),
                stale: unpack_bool(last_update_stale)?,
            },
            lending_market: Pubkey::new_from_array(*lending_market),
            owner: Pubkey::new_from_array(*owner),
            deposits,
            borrows,
            deposited_value: unpack_decimal(deposited_value),
            borrowed_value: unpack_decimal(borrowed_value),
            allowed_borrow_value: unpack_decimal(allowed_borrow_value),
            unhealthy_borrow_value: unpack_decimal(unhealthy_borrow_value),
        })
    }
}*/
#[cfg(test)]
mod test {
    use std::str::FromStr;

    use solana_program::pubkey::Pubkey;
    use borsh::*;
    use crate::{Message, MessageAccount};

    #[test]
    fn test_serialization() {
        let message_account =  MessageAccount::new(Message::String("Hello world!".into()), Pubkey::from_str("6yFmQCDXxuKdrou1dnag8zqg9LZKuuhjhJwGzoeSghrM").unwrap());
        let ser = message_account.try_to_vec().unwrap();
        dbg!(ser);
        let q = Pubkey::from_str("6yFmQCDXxuKdrou1dnag8zqg9LZKuuhjhJwGzoeSghrM").unwrap().try_to_vec().unwrap();
        let x = 1;
    }
}
// Sanity tests
/* #[cfg(test)]
mod test {
    use super::*;
    use solana_program::clock::Epoch;

    #[test]
    fn test_sanity() {
        let program_id = Pubkey::default();
        let key = Pubkey::default();
        let mut lamports = 30000000;
        let channel_account = ChannelAccount::new("org".into());
        let mut channel_account_serialization = channel_account.try_to_vec().unwrap();
        let me = Pubkey::default();
        let account = AccountInfo::new(
            &key,
            false,
            true,
            &mut lamports,
            &mut channel_account_serialization,
            &me,
            false,
            Epoch::default(),
        );

        //AccountInfo::new(key, is_signer, is_writable, lamports, data, owner, executable, rent_epoch)
        let accounts = vec![account];

        let create_channel_instruction_1  = ChatInstruction::CreateChannel(ChannelAccount::new("1".into()));
        let ser = create_channel_instruction_1.try_to_vec().unwrap();
        process_instruction(&program_id, &accounts, &ser).unwrap();

    }
} */

/*
assert_eq!(
    OrganizationAccount::try_from_slice(&accounts[0].data.borrow())
        .unwrap()
        .channels.len(),
    1
);

let create_channel_instruction_2  = ChatInstruction::CreateChannel(Channel::new("2".into()));
process_instruction(&program_id, &accounts, &create_channel_instruction_2.try_to_vec().unwrap()).unwrap();
assert_eq!(
    OrganizationAccount::try_from_slice(&accounts[0].data.borrow())
        .unwrap()
        .channels.len(),
    2
);*/
