import { SYSTEM_PROGRAM_ID } from "@coral-xyz/anchor/dist/cjs/native/system";

import { DappPDA } from "../../tests/mock-dapp-multi/setup";
import { connection, mockDappProgram, wallet } from "..";
import { TxnHelpers } from "../utils/transaction";

const args = process.argv.slice(2);
if (args.length != 3) throw new Error("Invalid arguments");

const networkId = args[0];
const srcEndpoint = args[1];
const dstEndpoint = args[2];
let txnHelpers = new TxnHelpers(connection, wallet.payer);

const addConnection = async () => {
  return await mockDappProgram.methods
    .addConnection(networkId, srcEndpoint, dstEndpoint)
    .accounts({
      connectionAccount: DappPDA.connections(networkId).pda,
      sender: wallet.publicKey,
      systemProgram: SYSTEM_PROGRAM_ID,
    })
    .signers([wallet.payer])
    .rpc();
};

addConnection()
  .then(async (sig) => {
    await txnHelpers.logParsedTx(sig);
  })
  .catch((err) => {
    console.log("Error while adding connection: ", err);
  });
