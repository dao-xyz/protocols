/**
 * Hello world
 */

import {
  establishConnection,
  establishPayer,
  getProgramInfo,
} from '.';

async function main() {
  console.log("Let's say hello to a Solana account...");

  // Establish connection to the cluster
  const connection = await establishConnection();
  
  // Get program info 
  const programInfo = await getProgramInfo(connection);

  // Determine who pays for the fees
  const payer = await establishPayer(connection);

  // Sandbox here
}

main().then(
  () => process.exit(),
  err => {
    console.error(err);
    process.exit(-1);
  },
);
