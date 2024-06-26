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

describe("xcall - handle message", () => {
  const provider = anchor.AnchorProvider.env();
  const connection = provider.connection;
  const wallet = provider.wallet as anchor.Wallet;

  const txnHelpers = new TxnHelpers(connection, wallet.payer);
  const ctx = new TestContext(connection, txnHelpers, wallet.payer);

  const xcallProgram: anchor.Program<Xcall> = anchor.workspace.Xcall;

  before(async () => {
    await ctx.setDefaultConnection("icx", Keypair.generate().publicKey);
  });

  it("should create and extend the lookup table", async () => {
    let lookupTable = await txnHelpers.createAddressLookupTable();
    await sleep(5);

    assert.equal(lookupTable, (await txnHelpers.getAddressLookupTable()).key);
  });

  it("should handle message request", async () => {
    let netId = "icx";
    let newKeypair = Keypair.generate();

    let request = new CSMessageRequest(
      "icx/abc",
      "icon",
      1,
      MessageType.CallMessage,
      new Uint8Array([0, 1, 2, 3]),
      [wallet.publicKey.toString(), newKeypair.publicKey.toString()]
    );

    let cs_message = new CSMessage(
      CSMessageType.CSMessageRequest,
      request.encode()
    ).encode();
    let message_seed = Buffer.from(hash(cs_message), "hex");

    await txnHelpers.airdrop(newKeypair.publicKey, 1e9);
    await sleep(3);

    let sources = [wallet.payer, newKeypair];

    for (let i = 0; i < sources.length; i++) {
      let handleMessageIx = await xcallProgram.methods
        .handleMessage(netId, Buffer.from(cs_message), new anchor.BN(1))
        .accountsStrict({
          signer: sources[i].publicKey,
          systemProgram: SYSTEM_PROGRAM_ID,
          config: XcallPDA.config().pda,
          pendingRequest: XcallPDA.pendingRequest(message_seed).pda,
          defaultConnection: XcallPDA.defaultConnection("icx").pda,
          rollbackAccount: null,
          pendingResponse: null,
          successfulResponse: null,
          proxyRequest: XcallPDA.proxyRequest(1).pda,
        })
        .instruction();

      let handleMessageTx = await txnHelpers.buildTxnWithLookupTable(
        [handleMessageIx],
        [sources[i]]
      );
      // await connection.sendTransaction(handleMessageTx);
    }
  });

  it("should handle message result", async () => {
    let nid = "icon";
    let newKeypair = Keypair.generate();
    let sequenceNo = 100;

    let result = new CSMessageResult(
      sequenceNo,
      CSResponseType.CSMessageFailure,
      new Uint8Array([])
    );

    let cs_message = new CSMessage(
      CSMessageType.CSMessageResult,
      result.encode()
    ).encode();
    let message_seed = Buffer.from(hash(cs_message), "hex");

    let sources = [wallet.payer, newKeypair];

    for (let i = 0; i < sources.length; i++) {
      const handleMessageIx = await xcallProgram.methods
        .handleMessage(nid, Buffer.from(cs_message), new anchor.BN(sequenceNo))
        .accountsStrict({
          signer: sources[i].publicKey,
          systemProgram: SYSTEM_PROGRAM_ID,
          config: XcallPDA.config().pda,
          pendingRequest: null,
          defaultConnection: XcallPDA.defaultConnection("icon").pda,
          rollbackAccount: XcallPDA.rollback(sequenceNo).pda,
          pendingResponse: XcallPDA.pendingResponse(message_seed).pda,
          successfulResponse: XcallPDA.successRes(sequenceNo).pda,
          proxyRequest: XcallPDA.proxyRequest(1).pda,
        })
        .instruction();

      const handleMessageTx = await txnHelpers.buildTxnWithLookupTable(
        [handleMessageIx],
        [sources[i]]
      );
      // await connection.sendTransaction(handleMessageTx);
    }
  });
});
