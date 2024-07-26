import * as anchor from "@coral-xyz/anchor";
import { assert } from "chai";

import { TestContext as DappTestCtx, DappPDA } from "./setup";
import { TxnHelpers, sleep } from "../utils";
import { SYSTEM_PROGRAM_ID } from "@coral-xyz/anchor/dist/cjs/native/system";
import { TestContext as XcallTestCtx, XcallPDA } from "../xcall/setup";

import { Xcall } from "../../target/types/xcall";
import { Envelope, CallMessage, MessageType } from "../xcall/types";

import { CentralizedConnection } from "../../target/types/centralized_connection";

import { ConnectionPDA } from "../centralized-connection/setup";
import { MockDappMulti } from "../../target/types/mock_dapp_multi";

const connectionProgram: anchor.Program<CentralizedConnection> =
  anchor.workspace.CentralizedConnection;

const xcallProgram: anchor.Program<Xcall> = anchor.workspace.Xcall;
const dappProgram: anchor.Program<MockDappMulti> =
  anchor.workspace.MockDappMulti;

describe("Mock Dapp", () => {
  const provider = anchor.AnchorProvider.env();
  const connection = provider.connection;
  const wallet = provider.wallet as anchor.Wallet;

  let txnHelpers = new TxnHelpers(connection, wallet.payer);
  let ctx = new DappTestCtx(connection, txnHelpers, wallet.payer);

  it("should send message", async () => {
    let xcall_context = new XcallTestCtx(connection, txnHelpers, wallet.payer);

    let envelope = new Envelope(
      MessageType.CallMessage,
      new CallMessage(new Uint8Array([])).encode(),
      [connectionProgram.programId.toString()],
      [wallet.publicKey.toString()]
    ).encode();

    const to = { "0": "0x3.icon/abc" };
    const msg_type = 0;
    const rollback = Buffer.from("rollback");
    const message = Buffer.from(envelope);

    let remaining_accounts = [
      {
        pubkey: XcallPDA.config().pda,
        isSigner: false,
        isWritable: true,
      },
      {
        pubkey: (await xcall_context.getConfig()).feeHandler,
        isSigner: false,
        isWritable: true,
      },
      {
        pubkey: XcallPDA.rollback(
          (await xcall_context.getConfig()).sequenceNo.toNumber() + 1
        ).pda,
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
        pubkey: ConnectionPDA.network_fee(ctx.networkId).pda,
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
    ];

    let sendCallIx = await dappProgram.methods
      .sendCallMessage(to, message, msg_type, rollback)
      .accountsStrict({
        config: DappPDA.config().pda,
        systemProgram: SYSTEM_PROGRAM_ID,
        connectionsAccount: DappPDA.connections(ctx.networkId).pda,
        sender: wallet.payer.publicKey,
      })
      .remainingAccounts(remaining_accounts)
      .instruction();

    let sendCallTx = await txnHelpers.buildV0Txn([sendCallIx], [wallet.payer]);
    await connection.sendTransaction(sendCallTx);
  });
});
