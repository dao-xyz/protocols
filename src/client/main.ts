/**
 * Hello world
 */

import {
  establishConnection,
  establishPayer,
  createChannelAccount,
  //sayHello,
  reportFindings,
} from './hello_world';

async function main() {
  console.log("Let's say hello to a Solana account...");

  // Establish connection to the cluster
  const connection = await establishConnection();

  // Determine who pays for the fees
  const payer = await establishPayer(connection);

  // Check if the program has been deployed
  const key = await createChannelAccount(payer,connection);

  // Say hello to an account
  ///await sayHello();

  // Find out how many times that account has been greeted
  await reportFindings(key, connection);

  console.log('Success');
}

main().then(
  () => process.exit(),
  err => {
    console.error(err);
    process.exit(-1);
  },
);
