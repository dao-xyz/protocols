/**
 * Hello world
 */

import {
  establishConnection,
  establishPayer,
  checkProgram,
  //sayHello,
  reportFindings,
} from './hello_world';

async function main() {
  console.log("Let's say hello to a Solana account...");

  // Establish connection to the cluster
  await establishConnection();

  // Determine who pays for the fees
  await establishPayer();

  // Check if the program has been deployed
  await checkProgram();

  // Say hello to an account
  ///await sayHello();

  // Find out how many times that account has been greeted
  await reportFindings();

  console.log('Success');
}

main().then(
  () => process.exit(),
  err => {
    console.error(err);
    process.exit(-1);
  },
);
