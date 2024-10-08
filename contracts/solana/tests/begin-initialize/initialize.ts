import * as anchor from "@coral-xyz/anchor";
import { assert, expect } from "chai";

import { TestContext as ConnectionTestContext } from "../centralized-connection/setup";
import { TxnHelpers, sleep } from "../utils";

import { TestContext as DappTestCtx, DappPDA } from "../mock-dapp-multi/setup";

import { Xcall } from "../../target/types/xcall";
import { TestContext as XcallTestContext, XcallPDA } from "../xcall/setup";
import { CentralizedConnection } from "../../target/types/centralized_connection";

import { MockDappMulti } from "../../target/types/mock_dapp_multi";
const dappProgram: anchor.Program<MockDappMulti> =
  anchor.workspace.MockDappMulti;

const xcallProgram: anchor.Program<Xcall> = anchor.workspace.Xcall;
const connectionProgram: anchor.Program<CentralizedConnection> =
  anchor.workspace.CentralizedConnection;

describe("Initialize", () => {
  const provider = anchor.AnchorProvider.env();
  let connection = provider.connection;
  let wallet = provider.wallet as anchor.Wallet;

  let txnHelpers = new TxnHelpers(connection, wallet.payer);

  let connectionCtx = new ConnectionTestContext(
    connection,
    txnHelpers,
    wallet.payer
  );
  let xcallCtx = new XcallTestContext(connection, txnHelpers, wallet.payer);
  let dappCtx = new DappTestCtx(connection, txnHelpers, wallet.payer);

  before(async () => {
    await dappCtx.add_connection(
      connectionCtx.dstNetworkId,
      connectionProgram.programId.toString(),
      connectionProgram.programId.toString()
    );
    await sleep(2);
  });

  it("should initialize xcall program", async () => {
    let ctx = new XcallTestContext(connection, txnHelpers, wallet.payer);

    let networkId = "solana";

    await xcallCtx.initialize(networkId);
    await sleep(2);

    let data = await ctx.getConfig();

    assert.equal(data.networkId.toString(), networkId);
    assert.equal(data.admin.toString(), wallet.publicKey.toString());
    assert.equal(data.feeHandler.toString(), wallet.publicKey.toString());
    assert.equal(data.protocolFee.toString(), new anchor.BN(0).toString());
    assert.equal(data.sequenceNo.toString(), new anchor.BN(0).toString());
    assert.equal(data.lastReqId.toString(), new anchor.BN(0).toString());
  });

  it("should fail when initializing xcall program two times", async () => {
    try {
      await xcallCtx.initialize("solana");
    } catch (err) {
      expect(err.message).to.includes(
        "Error processing Instruction 0: custom program error: 0x0"
      );
    }
  });

  it("should initialize centralized connection program", async () => {
    await connectionCtx.initialize();
    await sleep(2);

    let data = await connectionCtx.getConfig();

    assert.equal(
      data.admin.toString(),
      connectionCtx.signer.publicKey.toString()
    );
    assert.equal(data.xcall.toString(), xcallProgram.programId.toString());
    assert.equal(data.sn.toString(), new anchor.BN(0).toString());
  });

  it("should fail when initializing connection progarm two times", async () => {
    try {
      await connectionCtx.initialize();
    } catch (err) {
      expect(err.message).to.includes(
        "Error processing Instruction 0: custom program error: 0x0"
      );
    }
  });

  it("should initialize dapp program", async () => {
    await dappCtx.initialize();
    await sleep(2);

    let { xcallAddress } = await dappCtx.getConfig();

    assert.equal(xcallProgram.programId.toString(), xcallAddress.toString());
  });

  it("should fail when initializing dapp program two times", async () => {
    try {
      await dappCtx.initialize();
    } catch (err) {
      expect(err.message).to.includes(
        "Error processing Instruction 0: custom program error: 0x0"
      );
    }
  });
});
