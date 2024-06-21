import * as anchor from "@coral-xyz/anchor";
import { createHash } from "crypto";
import { Keypair } from "@solana/web3.js";
import { assert } from "chai";

import { XcallPDA } from "./setup";
import { TxnHelpers, sleep } from "../utils";
import { Xcall } from "../../target/types/xcall";
import {
  CSMessage,
  CSMessageRequest,
  CSMessageType,
  MessageType,
} from "./types";

describe("xcall - handle message", () => {
  const provider = anchor.AnchorProvider.env();
  const connection = provider.connection;
  const wallet = provider.wallet as anchor.Wallet;

  let txnHelpers = new TxnHelpers(connection, wallet.payer);

  const xcallProgram: anchor.Program<Xcall> = anchor.workspace.Xcall;

  const setDefaultConnection = async () => {
    let ix = await xcallProgram.methods
      .setDefaultConnection("icon", Keypair.generate().publicKey)
      .accounts({})
      .instruction();

    let tx = await txnHelpers.buildV0Txn([ix], [wallet.payer]);
    await connection.sendTransaction(tx);
    await sleep(3);
  };

  const initializeXcall = async () => {
    let initializeIx = await xcallProgram.methods
      .initialize("solana")
      .accounts({})
      .instruction();

    let tx = await txnHelpers.buildV0Txn([initializeIx], [wallet.payer]);
    await connection.sendTransaction(tx);
    await sleep(3);
  };

  before(async () => {
    let configPda = XcallPDA.config();
    let configAccount = await connection.getAccountInfo(configPda.pda, {
      commitment: "confirmed",
    });

    if (!configAccount || configAccount.lamports < 0) {
      await initializeXcall();
    }

    await setDefaultConnection();
  });

  it("should create and extend the lookup table", async () => {
    let lookupTable = await txnHelpers.createAddressLookupTable();
    await sleep(5);

    assert.equal(lookupTable, (await txnHelpers.getAddressLookupTable()).key);
  });

  it("should handle message request", async () => {
    let netId = "icon";

    let request = new CSMessageRequest(
      "icon/abc",
      "icon",
      1,
      MessageType.CallMessage,
      new Uint8Array([0, 1, 2, 3]),
      [wallet.publicKey.toString()]
    );

    let cs_message = new CSMessage(
      CSMessageType.CSMessageRequest,
      request.encode()
    ).encode();
    let message_seed = createHash("sha256").update(cs_message).digest("hex");

    let handleMessageIx = await xcallProgram.methods
      .handleMessage(
        netId,
        Buffer.from(cs_message),
        new anchor.BN(1),
        new anchor.BN(1),
        Buffer.from(message_seed, "hex")
      )
      .accountsPartial({
        rollbackAccunt: null,
        pendingResponse: null,
        successfulResponse: null,
        proxyRequest: XcallPDA.proxyRequest(1).pda,
      })
      .instruction();

    let handleMessageTx = await txnHelpers.buildTxnWithLookupTable(
      [handleMessageIx],
      [wallet.payer]
    );
    await sleep(3);

    let handleMessageTxSignature = await connection.sendTransaction(
      handleMessageTx
    );

    await sleep(3);
    console.log(
      await connection.getParsedTransaction(handleMessageTxSignature, {
        commitment: "confirmed",
        maxSupportedTransactionVersion: 0,
      })
    );
  });
});
