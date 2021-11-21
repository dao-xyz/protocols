/**
 * Hello world
 */

import {
  establishConnection,
  establishPayer,
  createChannelAccount,
  //sayHello,
  printChannelMessages,
  getProgramInfo,
  sendMessage,
} from './hello_world';

async function main() {
  console.log("Let's say hello to a Solana account...");

  // Establish connection to the cluster
  const connection = await establishConnection();
  
  // Get program info 
  const programInfo = await getProgramInfo(connection);

  // Determine who pays for the fees
  const payer = await establishPayer(connection);

  // Check if the program has been deployed
  const channelKey = await createChannelAccount(payer,connection,programInfo.key);

  // Say hello to an account
  await sendMessage("Hello world!", channelKey, payer,connection,programInfo.key);

  // Find out how many times that account has been greeted
  await printChannelMessages(channelKey, connection);

  console.log('Success');
}

main().then(
  () => process.exit(),
  err => {
    console.error(err);
    process.exit(-1);
  },
);
