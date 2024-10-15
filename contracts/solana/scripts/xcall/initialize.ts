import { SYSTEM_PROGRAM_ID } from "@coral-xyz/anchor/dist/cjs/native/system";

import { XcallPDA } from "./setup";
import { connection, xcallProgram, wallet } from "..";
import { TxnHelpers } from "../utils/transaction";

const args = process.argv.slice(2);
if (args.length != 1) throw new Error("Invalid arguments");

const networkId = args[0];
let txnHelpers = new TxnHelpers(connection, wallet.payer);

const initializeContract = async () => {
  let xcallConfig = await xcallProgram.account.config.fetchNullable(
    XcallPDA.config().pda
  );
  if (!xcallConfig) {
    return await xcallProgram.methods
      .initialize(networkId)
      .signers([wallet.payer])
      .accountsStrict({
        signer: wallet.publicKey,
        systemProgram: SYSTEM_PROGRAM_ID,
        config: XcallPDA.config().pda,
      })
      .rpc();
  }
};

initializeContract()
  .then(async (res) => {
    console.log("Contract initializing");
    if (res) await txnHelpers.logParsedTx(res);
    console.log("Contract initialized successfully");
  })
  .catch((err) => {
    console.log(err);
  });
