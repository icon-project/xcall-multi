import * as anchor from "@coral-xyz/anchor";
import { Keypair } from "@solana/web3.js";
import { assert } from "chai";
import { SYSTEM_PROGRAM_ID } from "@coral-xyz/anchor/dist/cjs/native/system";

import { TestContext, XcallPDA } from "./setup";
import { SYSVAR_INSTRUCTIONS_ID, TxnHelpers, sleep } from "../utils";
import { Xcall } from "../../target/types/xcall";
import {
  Envelope,
  CallMessage,
  MessageType,
  CallMessageWithRollback,
} from "./types";

import { CentralizedConnection } from "../../target/types/centralized_connection";
import { ConnectionPDA } from "../centralized-connection/setup";

const connectionProgram: anchor.Program<CentralizedConnection> =
  anchor.workspace.CentralizedConnection;

const xcallProgram: anchor.Program<Xcall> = anchor.workspace.Xcall;

describe("xcall - send message", () => {
  const provider = anchor.AnchorProvider.env();
  const connection = provider.connection;
  const wallet = provider.wallet as anchor.Wallet;

  let txnHelpers = new TxnHelpers(connection, wallet.payer);
  let ctx = new TestContext(connection, txnHelpers, wallet.payer);

  before(async () => {
    let fee_handler = Keypair.generate();
    await ctx.setFeeHandler(fee_handler);
    await txnHelpers.airdrop(fee_handler.publicKey, 1e9);

    await ctx.setProtocolFee(5000);
  });

  it("should send message", async () => {
    let envelope = new Envelope(
      MessageType.CallMessage,
      new CallMessage(new Uint8Array([1, 2])).encode(),
      [connectionProgram.programId.toString()],
      [wallet.publicKey.toString()]
    ).encode();
    const to = { "0": "0x3.icon/abc" };

    let config = await ctx.getConfig();
    let feeHandler = await connection.getAccountInfo(ctx.feeHandler.publicKey);
    let nextSequence = config.sequenceNo.toNumber() + 1;

    let sendCallIx = await xcallProgram.methods
      .sendCall(Buffer.from(envelope), to)
      .accountsStrict({
        systemProgram: SYSTEM_PROGRAM_ID,
        config: XcallPDA.config().pda,
        signer: wallet.payer.publicKey,
        dappAuthority: xcallProgram.programId,
        rollbackAccount: null,
        instructionSysvar: SYSVAR_INSTRUCTIONS_ID,
        feeHandler: ctx.feeHandler.publicKey,
      })
      .remainingAccounts([
        {
          pubkey: connectionProgram.programId,
          isSigner: false,
          isWritable: true,
        },
        {
          pubkey: ConnectionPDA.config().pda,
          isSigner: false,
          isWritable: true,
        },
        {
          pubkey: ConnectionPDA.network_fee(ctx.dstNetworkId).pda,
          isSigner: false,
          isWritable: true,
        },
      ])
      .instruction();

    let sendCallTx = await txnHelpers.buildV0Txn([sendCallIx], [wallet.payer]);
    await connection.sendTransaction(sendCallTx);
    await sleep(2);

    assert.equal(
      nextSequence.toString(),
      (await ctx.getConfig()).sequenceNo.toString()
    );
    assert.equal(
      (await connection.getAccountInfo(ctx.feeHandler.publicKey)).lamports,
      feeHandler.lamports + ctx.protocolFee
    );
  });
});
