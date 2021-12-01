/* eslint-disable @typescript-eslint/no-unsafe-assignment */
/* eslint-disable @typescript-eslint/no-unsafe-member-access */
import {
  Keypair,
  Connection,
  PublicKey,
  LAMPORTS_PER_SOL,
  AccountInfo,
} from '@solana/web3.js';
import fs from 'mz/fs';
import path from 'path';
import {getPayer, getRpcUrl, createKeypairFromFile} from './utils';



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
 * This file is created when running `solana program deploy dist/program/solvei.so`
 */
const PROGRAM_KEYPAIR_PATH = path.join(PROGRAM_PATH, 'solvei-keypair.json');

export interface ProgramInfo
{
  key:PublicKey,
  info: AccountInfo<Buffer>
}
export const getProgramInfo = async(connection:Connection):Promise<ProgramInfo> => {
  let programId = undefined;

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
  return {
    info: programInfo,
    key: programId
  }
}

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