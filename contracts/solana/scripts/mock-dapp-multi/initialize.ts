import { PublicKey } from "@solana/web3.js";
import { SYSTEM_PROGRAM_ID } from "@coral-xyz/anchor/dist/cjs/native/system";

import { DappPDA } from "../../tests/mock-dapp-multi/setup";
import { connection, mockDappProgram, wallet } from "..";
import { TxnHelpers } from "../utils/transaction";

const args = process.argv.slice(2);
if (args.length != 1) throw new Error("Invalid arguments");

const xcallKey = new PublicKey(args[0]);
let txnHelpers = new TxnHelpers(connection, wallet.payer);

const initializeContract = async () => {
  let dappConfig = await mockDappProgram.account.config.fetchNullable(
    DappPDA.config().pda
  );
  if (!dappConfig) {
    return await mockDappProgram.methods
      .initialize(xcallKey)
      .signers([wallet.payer])
      .accountsStrict({
        sender: wallet.publicKey,
        authority: DappPDA.authority().pda,
        systemProgram: SYSTEM_PROGRAM_ID,
        config: DappPDA.config().pda,
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
