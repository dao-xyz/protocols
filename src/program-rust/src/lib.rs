use accounts::{ChannelAccount, Message, UserAccount};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{account_info::{ next_account_info, AccountInfo}, entrypoint, entrypoint::ProgramResult, msg, program_error::ProgramError, pubkey::{Pubkey}, rent::Rent, sysvar::Sysvar};

use crate::{account::{create_and_serialize_account_signed, create_and_serialize_account_signed_verify}, accounts::{AccountContainer, MessageAccount, deserialize_user_account}, address::generate_seeds_from_string};

pub static NULL_KEY: Pubkey = Pubkey::new_from_array([0_u8; 32]);

mod account;
pub mod accounts;

pub mod address;
mod error;

pub static MESSAGE_TRANSACTION_MAX_SIZE: usize = 1200;



#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct SendMessage {
    pub user: Pubkey,
    pub channel: Pubkey,
    pub timestamp: u64,
    pub message: Message,
    pub bump_seed: u8 
}




#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct SubmitMessage {
    pub from: Pubkey
}

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, PartialEq)]
pub enum ChatInstruction {

    // Message builder is user to build a message that later can be submitted with the submitt message instruction
    CreateUser(UserAccount),

    // Create channel, that keep tracks of the message tail
    CreateChannel(ChannelAccount),

    // Update channel (the tail message)
    UpdateChannel(ChannelAccount),

    // Message builder is user to build a message that later can be submitted with the submitt message instruction
    SendMessage(SendMessage),

    // Add message to message builder
    //BuildMessagePart(String),

    // Submit message from BuildMessage invocations
    //SubmitMessage,
}


// Declare and export the program's entrypoint
entrypoint!(process_instruction);

// Program entrypoint's implementation
pub fn process_instruction(
    program_id: &Pubkey,      // Public key of the account the program was loaded into
    accounts: &[AccountInfo], // The account to say hello to
    input: &[u8],
) -> ProgramResult {
    msg!("Chat program entrypoint ABC!");

    let instruction = ChatInstruction::try_from_slice(input)?;

    // Iterating accounts is safer then indexing
    let accounts_iter = &mut accounts.iter();

    let system_account = next_account_info(accounts_iter)?;
    let _program_account = next_account_info(accounts_iter)?;
    let payer_account = next_account_info(accounts_iter)?;

    match instruction {
        ChatInstruction::CreateUser(user ) => {
            msg!("Create user" );
            msg!(user.name.as_str());
            msg!(user.owner.to_string().as_str());

            if user.name.len() == 0 
            {
                return Err(ProgramError::InvalidArgument);
            }
            // check if leading or trailing spaces, if so name is invalid
            let mut chars = user.name.chars();
            if chars.next().unwrap().is_whitespace() || chars.last().unwrap_or('a').is_whitespace()
            { 
                return Err(ProgramError::InvalidArgument);
            }

            if &user.owner != payer_account.key
            {
                return Err(ProgramError::IllegalOwner) // requires payer as owner (for now)
            }   
            msg!("Error checks done");


            let user_acount_info = next_account_info(accounts_iter)?;
            let rent = Rent::get()?;
            let seeds = generate_seeds_from_string(&user.name)?;
            let seed_slice = &seeds.iter().map(|x| &x[..]).collect::<Vec<&[u8]>>()[..];
            create_and_serialize_account_signed(
                payer_account,
                user_acount_info,
                &AccountContainer::UserAccount(user),
                seed_slice,
                program_id,
                system_account,
                &rent,
            )?;

        }

        
        ChatInstruction::CreateChannel(channel) => {
            let user_account_info = next_account_info(accounts_iter)?;
            let user = deserialize_user_account(user_account_info.data.borrow().as_ref());
            if &user.owner != payer_account.key
            {
                return Err(ProgramError::IllegalOwner) // requires payer as owner (for now)
            }

            let channel_account_info = next_account_info(accounts_iter)?;
      
            let rent = Rent::get()?;
            let seeds = generate_seeds_from_string(&channel.name)?;
            let seed_slice = &seeds.iter().map(|x| &x[..]).collect::<Vec<&[u8]>>()[..];
            create_and_serialize_account_signed(
                payer_account,
                channel_account_info,
                &AccountContainer::ChannelAccount(channel),
                seed_slice,
                program_id,
                system_account,
                &rent,
            )?;

        }
        ChatInstruction::UpdateChannel(_) => {
           /*  let channel_account_info = next_account_info(accounts_iter)?;

            // Don't allow channel name to be updated, since it would require us to resize the account size
            // This would also mean that the PDA would change!
            channel.serialize(&mut *channel_account_info.data.borrow_mut())? */
        }

        ChatInstruction::SendMessage(mut send_message) => {
            // Initializes an account for us that lets us build an message
            let user_account_info = next_account_info(accounts_iter)?;
            let channel_account_info = next_account_info(accounts_iter)?;
            let message_account = MessageAccount::new(send_message.user, send_message.channel, send_message.timestamp, send_message.message);
            let message_account_info = next_account_info(accounts_iter)?;
            let rent = Rent::get()?;
            let user = deserialize_user_account(user_account_info.data.borrow().as_ref());
            if &user.owner != payer_account.key
            {
                return Err(ProgramError::IllegalOwner) // requires payer as owner (for now)
            }
   
            let temp = AccountContainer::MessageAccount(message_account.clone());
            let vec = temp.try_to_vec()?;
            for v in vec
            {
                msg!(v.to_string().as_str());
            }
            create_and_serialize_account_signed_verify(
                payer_account,
                message_account_info,
                &AccountContainer::MessageAccount(message_account),
                &[&user_account_info.key.to_bytes(),&channel_account_info.key.to_bytes(),&send_message.timestamp.to_le_bytes()],
                program_id,
                system_account,
                &rent,
                message_account_info.key.clone(),
                send_message.bump_seed
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
/* 
pub fn replace_tail_message(message_account: &mut MessageAccount, messsage_account_key:&Pubkey, channel_account: &mut  ChannelAccount) {
        
    
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
    
} */


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

    use crate::accounts::{Message, MessageAccount};

    #[test]
    fn test_serialization() {

        
  /*       #[derive(Clone, Debug, BorshSerialize, BorshDeserialize, PartialEq)]
        struct Struct {
            u64: u64,
            a: Message
        }

        let message_account =  Struct {
            u64: 123,
            a: Message::String("Hello world!".into())
        };
        let ser = message_account.try_to_vec().unwrap();
        let x = 123;  */
        let message_account_2 =  MessageAccount::new(Pubkey::new_unique(), Pubkey::new_unique(),   123, Message::String("Hello world!".into()));
        let ser2 = message_account_2.try_to_vec().unwrap();
        let x2 = 123; 
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
