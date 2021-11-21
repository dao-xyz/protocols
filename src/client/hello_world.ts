/* eslint-disable @typescript-eslint/no-unsafe-assignment */
/* eslint-disable @typescript-eslint/no-unsafe-member-access */

import {
  Keypair,
  Connection,
  PublicKey,
  LAMPORTS_PER_SOL,
  TransactionInstruction,
  Transaction,
  SystemProgram,
  sendAndConfirmTransaction,
} from '@solana/web3.js';
import fs from 'mz/fs';
import path from 'path';
import {serialize, deserialize} from '@quantleaf/borsh';

import { ChannelAccount, SCHEMAS } from './schema';

import {getPayer, getRpcUrl, createKeypairFromFile} from './utils';



/**
 * Hello world's program id
 */
let programId: PublicKey;

/**
 * Path to program files
 */
const PROGRAM_PATH = path.resolve(__dirname, '../../dist/program');

/**
 * Path to program shared object file which should be deployed on chain.
 * This file is created when running either:
 *   - `npm run build:program-rust`
 */
const PROGRAM_SO_PATH = path.join(PROGRAM_PATH, 'solvei.so');

/**
 * Path to the keypair of the deployed program.
 * This file is created when running `solana program deploy dist/program/helloworld.so`
 */
const PROGRAM_KEYPAIR_PATH = path.join(PROGRAM_PATH, 'solvei-keypair.json');


/**
 * Establish a connection to the cluster
 */
export async function establishConnection(): Promise<Connection> {
  const rpcUrl = await getRpcUrl();
  const connection = new Connection(rpcUrl, 'confirmed');
  const version = await connection.getVersion();
  console.log('Connection to cluster established:', rpcUrl, version);
  return connection;
}

/**
 * Establish an account to pay for everything
 */
export async function establishPayer(connection:Connection, payer?:Keypair): Promise<Keypair> {
  let fees = 0;
  if (!payer) {
    const {feeCalculator} = await connection.getRecentBlockhash();

    // Calculate the cost to fund the greeter account
    fees += 0.001;// some random value await connection.getMinimumBalanceForRentExemption();

    // Calculate the cost of sending transactions
    fees += feeCalculator.lamportsPerSignature * 100; // wag

    payer = await getPayer();
  }

  let lamports = await connection.getBalance(payer.publicKey);
  if (lamports < fees) {
    // If current balance is not enough to pay for fees, request an airdrop
    const sig = await connection.requestAirdrop(
      payer.publicKey,
      fees - lamports,
    );
    await connection.confirmTransaction(sig);
    lamports = await connection.getBalance(payer.publicKey);
  }

  console.log(
    'Using account',
    payer.publicKey.toBase58(),
    'containing',
    lamports / LAMPORTS_PER_SOL,
    'SOL to pay for fees',
  );
  return payer;
}

/**
 * Check if the hello world BPF program has been deployed
 */
export async function createChannelAccount(payer:Keypair, connection:Connection): Promise<PublicKey> {
  // Read program id from keypair file
  try {
    const programKeypair = await createKeypairFromFile(PROGRAM_KEYPAIR_PATH);
    programId = programKeypair.publicKey;
  } catch (err) {
    const errMsg = (err as Error).message;
    throw new Error(
      `Failed to read program keypair at '${PROGRAM_KEYPAIR_PATH}' due to error: ${errMsg}. Program may need to be deployed with \`solana program deploy dist/program/solvei.so\``,
    );
  }

  // Check if the program has been deployed
  const programInfo = await connection.getAccountInfo(programId);
  if (programInfo === null) {
    if (fs.existsSync(PROGRAM_SO_PATH)) {
      throw new Error(
        'Program needs to be deployed with `solana program deploy dist/program/solvei.so`',
      );
    } else {
      throw new Error('Program needs to be built and deployed');
    }
  } else if (!programInfo.executable) {
    throw new Error(`Program is not executable`);
  }
  console.log(`Using program ${programId.toBase58()}`);

  // Derive the address (public key) of a greeting account from the program so that it's easy to find later.
  const channelName = 'New channel!';
  const [channelAccountPubkey, _] = await PublicKey.findProgramAddress(
    [Uint8Array.from(channelName, x => x.charCodeAt(0))],
    programId,
  );

  // Check if the greeting account has already been created
  const channelAccount = await connection.getAccountInfo(channelAccountPubkey);
  if (channelAccount === null) {
    console.log(
      'Creating account',
      channelAccountPubkey.toBase58(),
      'to send messages to',
    );
    const transanction = new TransactionInstruction({
      keys: [
        {
          pubkey: SystemProgram.programId, isSigner: false, isWritable: true
        },
        {
          pubkey: programId, isSigner: false, isWritable: true
        },
        {
          pubkey: payer.publicKey, isSigner: true, isWritable: true
        },
        {
          pubkey: channelAccountPubkey, isSigner: false, isWritable: true
        }
      
      ],
      programId,
      data:Buffer.from(Uint8Array.of(0, ...serialize(
        SCHEMAS,
        new ChannelAccount({name: channelName, tail_message: PublicKey.default }),
      )))
    });
    await sendAndConfirmTransaction(connection, new Transaction().add(transanction), [payer]);
  }
  return channelAccountPubkey;

}


/**
 * Check if the hello world BPF program has been deployed
 */
 export async function sendMessage(payer:Keypair, connection:Connection): Promise<PublicKey> {
  // Read program id from keypair file
  try {
    const programKeypair = await createKeypairFromFile(PROGRAM_KEYPAIR_PATH);
    programId = programKeypair.publicKey;
  } catch (err) {
    const errMsg = (err as Error).message;
    throw new Error(
      `Failed to read program keypair at '${PROGRAM_KEYPAIR_PATH}' due to error: ${errMsg}. Program may need to be deployed with \`solana program deploy dist/program/solvei.so\``,
    );
  }

  // Check if the program has been deployed
  const programInfo = await connection.getAccountInfo(programId);
  if (programInfo === null) {
    if (fs.existsSync(PROGRAM_SO_PATH)) {
      throw new Error(
        'Program needs to be deployed with `solana program deploy dist/program/solvei.so`',
      );
    } else {
      throw new Error('Program needs to be built and deployed');
    }
  } else if (!programInfo.executable) {
    throw new Error(`Program is not executable`);
  }
  console.log(`Using program ${programId.toBase58()}`);

  // Derive the address (public key) of a greeting account from the program so that it's easy to find later.
  const channelName = 'New channel!';
  const [messageAccountPubkey, _] = await PublicKey.findProgramAddress(
    [payer.publicKey.toBuffer()],
    programId,
  );

  // Check if the greeting account has already been created

  const transanction = new TransactionInstruction({
    keys: [
      {
        pubkey: SystemProgram.programId, isSigner: false, isWritable: true
      },
      {
        pubkey: programId, isSigner: false, isWritable: true
      },
      {
        pubkey: payer.publicKey, isSigner: true, isWritable: true
      },
      {
        pubkey: channelAccountPubkey, isSigner: false, isWritable: true
      }
    
    ],
    programId,
    data:Buffer.from(Uint8Array.of(0, ...serialize(
      SCHEMAS,
      new ChannelAccount({name: channelName, tail_message: PublicKey.default }),
    )))
  });
  await sendAndConfirmTransaction(connection, new Transaction().add(transanction), [payer]);
  
  return channelAccountPubkey;

}

/**
 * Say hello
 */
/*export async function sayHello(): Promise<void> {
  console.log('Saying hello to', greetedPubkey.toBase58());
  const instruction = new TransactionInstruction({
    keys: [{pubkey: greetedPubkey, isSigner: false, isWritable: true}],
    programId,
    data: Buffer.alloc(0), // All instructions are hellos
  });
  await sendAndConfirmTransaction(
    connection,
    new Transaction().add(instruction),
    [payer],
  );
}*/

/**
 * Report the number of times the greeted account has been said hello to
 */
export async function reportFindings(channelAccount:PublicKey, connection:Connection): Promise<void> {
  const accountInfo = await connection.getAccountInfo(channelAccount);
  if (accountInfo === null) {
    throw 'Error: cannot find the greeted account';
  }
  const greeting = deserialize(
    SCHEMAS,
    ChannelAccount,
    accountInfo.data,
  );
  console.log(
    channelAccount.toBase58(),
    'has been created',
    JSON.stringify(greeting),
    'time(s)',
  );
}
