import * as anchor from "@coral-xyz/anchor";

import { TestContext, DappPDA } from "./setup";
import { TxnHelpers, sleep } from "../utils";
import { SYSTEM_PROGRAM_ID } from "@coral-xyz/anchor/dist/cjs/native/system";
import { CentralizedConnection } from "../../target/types/centralized_connection";

import { ConnectionPDA } from "../centralized-connection/setup";

import { Xcall } from "../../target/types/xcall";
import { XcallPDA } from "../xcall/setup";
import { CallMessageWithRollback, Envelope, MessageType } from "../xcall/types";
import { TestContext as XcallTestContext } from "../xcall/setup";

import { MockDappMulti } from "../../target/types/mock_dapp_multi";

const xcallProgram: anchor.Program<Xcall> = anchor.workspace.Xcall;
const connectionProgram: anchor.Program<CentralizedConnection> =
  anchor.workspace.CentralizedConnection;
const mockDappProgram: anchor.Program<MockDappMulti> =
  anchor.workspace.MockDappMulti;

describe("CentralizedConnection", () => {
  const provider = anchor.AnchorProvider.env();
  const connection = provider.connection;
  const wallet = provider.wallet as anchor.Wallet;

  let txnHelpers = new TxnHelpers(connection, wallet.payer);
  let ctx = new TestContext(connection, txnHelpers, wallet.payer);

  let xcallCtx = new XcallTestContext(connection, txnHelpers, wallet.payer);

  it("should send rollback message", async () => {
    let data = Buffer.from("rollback", "utf-8");
    let rollback_data = Buffer.from("rollback", "utf-8");

    let xcallConfig = await xcallCtx.getConfig();
    let nextSequenceNo = xcallConfig.sequenceNo.toNumber() + 1;

    let envelope = new Envelope(
      MessageType.CallMessageWithRollback,
      new CallMessageWithRollback(data, rollback_data).encode(),
      [connectionProgram.programId.toString()],
      [connectionProgram.programId.toString()]
    ).encode();

    const to = {
      "0": `${ctx.dstNetworkId}/${mockDappProgram.programId.toString()}`,
    };

    let sendCallIx = await mockDappProgram.methods
      .sendCallMessage(
        to,
        Buffer.from(envelope),
        MessageType.CallMessageWithRollback,
        Buffer.from("rollback")
      )
      .accountsStrict({
        config: DappPDA.config().pda,
        systemProgram: SYSTEM_PROGRAM_ID,
        connectionsAccount: DappPDA.connections(ctx.dstNetworkId).pda,
        sender: ctx.admin.publicKey,
      })
      .remainingAccounts([
        {
          pubkey: XcallPDA.config().pda,
          isSigner: false,
          isWritable: true,
        },
        {
          pubkey: xcallConfig.feeHandler,
          isSigner: false,
          isWritable: true,
        },
        {
          pubkey: XcallPDA.rollback(nextSequenceNo).pda,
          isSigner: false,
          isWritable: true,
        },
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
        {
          pubkey: xcallProgram.programId,
          isSigner: false,
          isWritable: true,
        },
        {
          pubkey: connectionProgram.programId,
          isSigner: false,
          isWritable: true,
        },
      ])
      .instruction();

    let sendCallTx = await txnHelpers.buildV0Txn([sendCallIx], [ctx.admin]);
    await connection.sendTransaction(sendCallTx);
    await sleep(2);
  });
});
