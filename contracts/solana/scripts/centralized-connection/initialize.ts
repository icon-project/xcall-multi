import { PublicKey } from "@solana/web3.js";
import { SYSTEM_PROGRAM_ID } from "@coral-xyz/anchor/dist/cjs/native/system";

import { ConnectionPDA } from "../../tests/centralized-connection/setup";
import { connection, connectionProgram, wallet } from "..";
import { TxnHelpers } from "../utils/transaction";

const args = process.argv.slice(2);
if (args.length != 2) throw new Error("Invalid arguments");

const xcallKey = new PublicKey(args[0]);
const adminKey = new PublicKey(args[1]);
let txnHelpers = new TxnHelpers(connection, wallet.payer);

const initializeContract = async () => {
  let connConfig = await connectionProgram.account.config.fetchNullable(
    ConnectionPDA.config().pda
  );
  if (!connConfig) {
    return await connectionProgram.methods
      .initialize(xcallKey, adminKey)
      .signers([wallet.payer])
      .accountsStrict({
        signer: wallet.publicKey,
        systemProgram: SYSTEM_PROGRAM_ID,
        config: ConnectionPDA.config().pda,
        authority: ConnectionPDA.authority().pda,
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
