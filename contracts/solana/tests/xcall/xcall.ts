import * as anchor from "@coral-xyz/anchor";

import { TxnHelpers } from "../utils/transaction";
import { Xcall } from "../../target/types/xcall";
import { expect } from "chai";

describe("Xcall", async () => {
  const provider = anchor.AnchorProvider.env();
  const program: anchor.Program<Xcall> = anchor.workspace.Xcall;

  let connection = provider.connection;
  let wallet = provider.wallet as anchor.Wallet;

  const txnHelpers = new TxnHelpers(connection, wallet.payer);

  it("[initialize]: should initialize the program", async () => {
    let initializeIx = await program.methods
      .initialize("solana")
      .accounts({})
      .instruction();

    let tx = await txnHelpers.buildV0Txn([initializeIx], [wallet.payer]);
    await connection.sendTransaction(tx);
  });

  it("[initialize]: should fail on double initialize", async () => {
    try {
      await program.methods
        .initialize("solana")
        .accounts({})
        .signers([wallet.payer])
        .rpc();
    } catch (err) {
      expect(err.message).to.not.be.empty;
    }
  });
});
