import * as anchor from "@coral-xyz/anchor";

import { TxnHelpers } from "../utils/transaction";
import { connection, wallet, xcallProgram } from "..";
import { XcallPDA } from "./setup";

let args = process.argv.slice(2);
if (args.length != 1) throw new Error("Invalid arguments");

const protocolFee = args[0];

let txnHelpers = new TxnHelpers(connection, wallet.payer);

const setProtocolFee = async () => {
  return await xcallProgram.methods
    .setProtocolFee(new anchor.BN(protocolFee))
    .accountsStrict({
      admin: wallet.publicKey,
      config: XcallPDA.config().pda,
    })
    .signers([wallet.payer])
    .rpc();
};

setProtocolFee()
  .then(async (sig) => {
    await txnHelpers.logParsedTx(sig);
  })
  .catch((err) => {
    console.log("Error while setting fee: ", err);
  });
