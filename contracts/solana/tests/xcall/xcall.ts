import * as anchor from "@coral-xyz/anchor";
import { assert } from "chai";

import { TxnHelpers } from "../utils/transaction";
import { Xcall } from "../../target/types/xcall";

import { CentralizedConnection } from "../../target/types/centralized_connection";
import { TestContext as ConnectionTestContext } from "../centralized-connection/setup";
import { ConnectionPDA } from "../centralized-connection/setup";
import { TestContext, XcallPDA } from "./setup";
import { sleep } from "../utils";

const connectionProgram: anchor.Program<CentralizedConnection> =
  anchor.workspace.CentralizedConnection;

const xcallProgram: anchor.Program<Xcall> = anchor.workspace.Xcall;

describe("Xcall", async () => {
  const provider = anchor.AnchorProvider.env();
  const connection = provider.connection;
  const wallet = provider.wallet as anchor.Wallet;

  let txnHelpers = new TxnHelpers(connection, wallet.payer);
  let ctx = new TestContext(connection, txnHelpers, wallet.payer);

  it("[get_fee]: should get fee", async () => {
    let isResponse = true;
    let networkId = "icx";

    let fee = await xcallProgram.methods
      .getFee(networkId, isResponse, [connectionProgram.programId.toString()])
      .accountsStrict({
        config: XcallPDA.config().pda,
        reply: XcallPDA.reply().pda,
        defaultConnection: XcallPDA.defaultConnection("icx").pda,
      })
      .remainingAccounts([
        {
          pubkey: ConnectionPDA.fee("icx").pda,
          isSigner: false,
          isWritable: true,
        },
        {
          pubkey: connectionProgram.programId,
          isSigner: false,
          isWritable: true,
        },
      ])
      .view({ commitment: "confirmed" });
    await sleep(2);

    let connectionFee = await connectionProgram.methods
      .getFee(networkId, isResponse)
      .accountsStrict({
        networkFee: ConnectionPDA.fee("icx").pda,
      })
      .view();

    let xcallConfig = await ctx.getConfig();
    assert.equal(
      fee.toString(),
      xcallConfig.protocolFee.toNumber() + connectionFee.toNumber()
    );
  });
});
