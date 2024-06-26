import * as anchor from "@coral-xyz/anchor";
import { Keypair } from "@solana/web3.js";
import { assert } from "chai";

import { TestContext, XcallPDA } from "./setup";
import { TxnHelpers, hash, sleep } from "../utils";
import { Xcall } from "../../target/types/xcall";
import {
  CSMessage,
  CSMessageRequest,
  CSMessageResult,
  CSMessageType,
  CSResponseType,
  MessageType,
} from "./types";
import { SYSTEM_PROGRAM_ID } from "@coral-xyz/anchor/dist/cjs/native/system";

describe("xcall - execute rollback", () => {
  const provider = anchor.AnchorProvider.env();
  const connection = provider.connection;
  const wallet = provider.wallet as anchor.Wallet;

  const txnHelpers = new TxnHelpers(connection, wallet.payer);
  const ctx = new TestContext(connection, txnHelpers, wallet.payer);

  const xcallProgram: anchor.Program<Xcall> = anchor.workspace.Xcall;

  before(async () => {
    // await ctx.setDefaultConnection("icx", Keypair.generate().publicKey);
  });

//   it("should create and extend the lookup table", async () => {
//     let lookupTable = await txnHelpers.createAddressLookupTable();
//     await sleep(5);

//     assert.equal(lookupTable, (await txnHelpers.getAddressLookupTable()).key);
//   });

  it("should execute rollback", async () => {
    let netId = 5;
    let newKeypair = Keypair.generate();

    let request = new CSMessageRequest(
      "icx/abc",
      "icon",
      1,
      MessageType.CallMessage,
      new Uint8Array([0, 1, 2, 3]),
      [wallet.publicKey.toString(), newKeypair.publicKey.toString()]
    );

 

    await txnHelpers.airdrop(newKeypair.publicKey, 1e9);
    await sleep(3);

    let sources = [wallet.payer, newKeypair];
        let sn = new anchor.BN(1)
    for (let i = 0; i < sources.length; i++) {
      let executeRollbackIx = await xcallProgram.methods
        .executeRollback(sn)
        .accountsStrict({
          signer: sources[i].publicKey,
          systemProgram: SYSTEM_PROGRAM_ID,
    
          rollback: XcallPDA.rollback(1).pda,
     
        })
        .instruction();

      let handleMessageTx = await txnHelpers.buildTxnWithLookupTable(
        [executeRollbackIx],
        [sources[i]]
      );
      // await connection.sendTransaction(handleMessageTx);
    }
  });


});
