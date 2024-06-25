import * as anchor from "@coral-xyz/anchor";
import { Keypair, PublicKey } from "@solana/web3.js";
import { SYSTEM_PROGRAM_ID } from "@coral-xyz/anchor/dist/cjs/native/system";

import { TestContext, XcallPDA } from "./setup";
import { TxnHelpers } from "../utils";
import { Xcall } from "../../target/types/xcall";
import { CentralizedConnection } from "../../target/types/centralized_connection";
import { Envelope, CallMessage, MessageType } from "./types";

const xcallProgram: anchor.Program<Xcall> = anchor.workspace.Xcall;

const connectionProgram: anchor.Program<CentralizedConnection> =
  anchor.workspace.CentralizedConnection;
import { connectionPDA } from "../centralized-connection/setup";

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
      new CallMessage(new Uint8Array([])).encode(),
      [connectionProgram.programId.toString()],
      [wallet.publicKey.toString()]
    ).encode();
    const to = { "0": "icx/abc" };

    let sendCallIx = await xcallProgram.methods
      .sendCall(Buffer.from(envelope), to)
      .accountsStrict({
        systemProgram: SYSTEM_PROGRAM_ID,
        config: XcallPDA.config().pda,
        signer: wallet.payer.publicKey,
        reply: XcallPDA.reply().pda,
        rollbackAccount: XcallPDA.rollback(1).pda,
        feeHandler: ctx.fee_handler.publicKey,
        defaultConnection: XcallPDA.defaultConnection("icx").pda,
      })
      .remainingAccounts([
        {
          pubkey: connectionProgram.programId,
          isSigner: false,
          isWritable: true,
        },
        {
          pubkey: connectionPDA.config().pda,
          isSigner: false,
          isWritable: true,
        },
        {
          pubkey: connectionPDA.fee("icx").pda,
          isSigner: false,
          isWritable: true,
        },
        {
          pubkey: connectionPDA.claimFees().pda,
          isSigner: false,
          isWritable: true,
        },
      ])
      .instruction();

    let sendCallTx = await txnHelpers.buildV0Txn([sendCallIx], [wallet.payer]);
    let sendCallTxSignature = await connection.sendTransaction(sendCallTx);
    await txnHelpers.logParsedTx(sendCallTxSignature);
  });
});
