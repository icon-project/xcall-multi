import * as anchor from "@coral-xyz/anchor";
import { assert, expect } from "chai";
import { Keypair } from "@solana/web3.js";

import { TestContext, ConnectionPDA } from "./setup";
import { TxnHelpers, hash, sleep } from "../utils";
import { SYSTEM_PROGRAM_ID } from "@coral-xyz/anchor/dist/cjs/native/system";
import { CentralizedConnection } from "../../target/types/centralized_connection";

import { Xcall } from "../../target/types/xcall";
import { XcallPDA } from "../xcall/setup";
import {
  CSMessage,
  CSMessageRequest,
  CSMessageResult,
  CSMessageType,
  CSResponseType,
  CallMessageWithRollback,
  Envelope,
  MessageType,
} from "../xcall/types";
import { TestContext as XcallTestContext } from "../xcall/setup";

const xcallProgram: anchor.Program<Xcall> = anchor.workspace.Xcall;
const connectionProgram: anchor.Program<CentralizedConnection> =
  anchor.workspace.CentralizedConnection;

describe("CentralizedConnection", () => {
  const provider = anchor.AnchorProvider.env();
  const connection = provider.connection;
  const wallet = provider.wallet as anchor.Wallet;

  let txnHelpers = new TxnHelpers(connection, wallet.payer);
  let ctx = new TestContext(connection, txnHelpers, wallet.payer);

  let xcallCtx = new XcallTestContext(connection, txnHelpers, wallet.payer);

  before(async () => {
    await xcallCtx.setDefaultConnection(
      ctx.dstNetworkId,
      connectionProgram.programId
    );
    await sleep(2);
  });

  it("[set_admin]: should set the new admin", async () => {
    let newAdmin = Keypair.generate();
    await ctx.setAdmin(newAdmin);

    await sleep(2);

    let { admin } = await ctx.getConfig();
    assert.equal(ctx.admin.publicKey.toString(), admin.toString());
  });

  it("[set_admin]: should fail if not called by admin", async () => {
    let non_admin = Keypair.generate();

    try {
      await ctx.program.methods
        .setAdmin(Keypair.generate().publicKey)
        .accountsStrict({
          admin: non_admin.publicKey,
          config: ConnectionPDA.config().pda,
        })
        .signers([non_admin])
        .rpc();
    } catch (err) {
      expect(err.message).to.includes("Only admin");
    }
  });

  it("[set_fee]: should set the fee for network ID", async () => {
    let msg_fee = 50;
    let res_fee = 100;

    await txnHelpers.airdrop(ctx.admin.publicKey, 1e9);
    await sleep(2);

    await ctx.program.methods
      .setFee(ctx.dstNetworkId, new anchor.BN(msg_fee), new anchor.BN(res_fee))
      .accountsStrict({
        config: ConnectionPDA.config().pda,
        networkFee: ConnectionPDA.network_fee(ctx.dstNetworkId).pda,
        admin: ctx.admin.publicKey,
        systemProgram: SYSTEM_PROGRAM_ID,
      })
      .signers([ctx.admin])
      .rpc();

    await sleep(2);

    let fee = await ctx.getFee(ctx.dstNetworkId);
    assert.equal(fee.messageFee.toNumber(), msg_fee);
    assert.equal(fee.responseFee.toNumber(), res_fee);
  });

  it("[claim_fees]: should claim fee stored in PDA account", async () => {
    let claimFee = ConnectionPDA.claimFees().pda;

    let transfer_amount = 500_000;
    await txnHelpers.airdrop(claimFee, transfer_amount);
    await sleep(2);

    const min_rent_exempt_balance =
      await ctx.connection.getMinimumBalanceForRentExemption(9);
    const before_pda_balance = (await ctx.connection.getAccountInfo(claimFee))
      .lamports;
    assert.equal(min_rent_exempt_balance + transfer_amount, before_pda_balance);

    await ctx.program.methods
      .claimFees()
      .accountsStrict({
        admin: ctx.admin.publicKey,
        config: ConnectionPDA.config().pda,
        claimFee,
      })
      .signers([ctx.admin])
      .rpc();

    const after_pda_balance = (await ctx.connection.getAccountInfo(claimFee))
      .lamports;
    assert.equal(min_rent_exempt_balance, after_pda_balance);
  });

  it("[claim_fees]: should fail if not called by admin", async () => {
    let new_admin = Keypair.generate();

    try {
      await ctx.program.methods
        .claimFees()
        .accountsStrict({
          admin: new_admin.publicKey,
          config: ConnectionPDA.config().pda,
          claimFee: ConnectionPDA.claimFees().pda,
        })
        .signers([new_admin])
        .rpc();
    } catch (err) {
      expect(err.message).includes("OnlyAdmin");
    }
  });

  it("[recv_message]: should fail if not called by an admin", async () => {
    const connSn = 1;
    const fromNetwork = ctx.dstNetworkId;
    let csMessage = new Uint8Array([1, 2, 3]);

    try {
      await ctx.program.methods
        .recvMessage(
          fromNetwork,
          new anchor.BN(connSn),
          Buffer.from(csMessage),
          new anchor.BN(connSn)
        )
        .accountsStrict({
          config: ConnectionPDA.config().pda,
          admin: ctx.signer.publicKey,
          receipt: ConnectionPDA.receipt(connSn).pda,
          systemProgram: SYSTEM_PROGRAM_ID,
        })
        .signers([ctx.signer])
        .rpc();
    } catch (err) {
      expect(err.message).includes("Only admin");
    }
  });

  it("[recv_message]: should receive message and call handle message request of xcall", async () => {
    let xcallConfig = await xcallCtx.getConfig();

    const connSn = 1;
    const fromNetwork = ctx.dstNetworkId;
    let nextReqId = xcallConfig.lastReqId.toNumber() + 1;
    let nextSequenceNo = xcallConfig.sequenceNo.toNumber() + 1;

    let request = new CSMessageRequest(
      "icon/abc",
      ctx.dstNetworkId,
      nextSequenceNo,
      MessageType.CallMessagePersisted,
      new Uint8Array([0, 1, 2, 3]),
      [connectionProgram.programId.toString()]
    );

    let cs_message = new CSMessage(
      CSMessageType.CSMessageRequest,
      request.encode()
    ).encode();

    await ctx.program.methods
      .recvMessage(
        fromNetwork,
        new anchor.BN(connSn),
        Buffer.from(cs_message),
        new anchor.BN(nextSequenceNo)
      )
      .accountsStrict({
        config: ConnectionPDA.config().pda,
        admin: ctx.admin.publicKey,
        receipt: ConnectionPDA.receipt(connSn).pda,
        systemProgram: SYSTEM_PROGRAM_ID,
      })
      .remainingAccounts([
        {
          pubkey: XcallPDA.config().pda,
          isSigner: false,
          isWritable: true,
        },
        {
          pubkey: xcallCtx.admin.publicKey,
          isSigner: false,
          isWritable: true,
        },
        {
          pubkey: XcallPDA.defaultConnection(ctx.dstNetworkId).pda,
          isSigner: false,
          isWritable: true,
        },
        {
          pubkey: XcallPDA.proxyRequest(nextReqId).pda,
          isSigner: false,
          isWritable: true,
        },
        {
          pubkey: xcallProgram.programId,
          isSigner: false,
          isWritable: true,
        },
        {
          pubkey: xcallProgram.programId,
          isSigner: false,
          isWritable: true,
        },
        {
          pubkey: xcallProgram.programId,
          isSigner: false,
          isWritable: true,
        },
        {
          pubkey: xcallProgram.programId,
          isSigner: false,
          isWritable: true,
        },
      ])
      .signers([ctx.admin])
      .rpc();

    await sleep(2);

    // expect receipt account to be initialized
    expect(await ctx.getReceipt(nextSequenceNo)).to.be.empty;

    // expect proxy request in xcall PDA's account
    let proxyRequest = await xcallCtx.getProxyRequest(nextReqId);
    expect(proxyRequest.req.protocols).to.includes(
      connectionProgram.programId.toString()
    );
    expect(proxyRequest.req.from[0]).to.equal(request.from);
    expect(proxyRequest.req.data.toString()).to.equal(
      Buffer.from(hash(new Uint8Array([0, 1, 2, 3])), "hex").toString()
    );

    // expect request to be increased in xcall config PDA's
    expect((await xcallCtx.getConfig()).lastReqId.toString()).to.equal(
      nextReqId.toString()
    );
  });

  it("[recv_message]: should receive message and call xcall handle message result", async () => {
    // send rollback message
    let envelope = new Envelope(
      MessageType.CallMessageWithRollback,
      new CallMessageWithRollback(
        new Uint8Array([1, 2, 3]),
        new Uint8Array([1, 2, 3])
      ).encode(),
      [connectionProgram.programId.toString()],
      [wallet.publicKey.toString()]
    ).encode();
    const to = { "0": "icon/abc" };

    let xcallConfig = await xcallCtx.getConfig();
    let nextSequenceNo = xcallConfig.sequenceNo.toNumber() + 1;
    let nextReqId = xcallConfig.lastReqId.toNumber() + 1;

    let sendCallIx = await xcallProgram.methods
      .sendCall(Buffer.from(envelope), to)
      .accountsStrict({
        systemProgram: SYSTEM_PROGRAM_ID,
        config: XcallPDA.config().pda,
        signer: wallet.payer.publicKey,
        rollbackAccount: XcallPDA.rollback(1).pda,
        feeHandler: xcallCtx.feeHandler.publicKey,
        defaultConnection: XcallPDA.defaultConnection(ctx.dstNetworkId).pda,
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
        {
          pubkey: ConnectionPDA.claimFees().pda,
          isSigner: false,
          isWritable: true,
        },
      ])
      .instruction();

    let sendCallTx = await txnHelpers.buildV0Txn([sendCallIx], [wallet.payer]);
    await connection.sendTransaction(sendCallTx);
    await sleep(2);

    // receive message of rollback message
    let connSn = 2;
    let responseCode = CSResponseType.CSResponseSuccess;

    let request = new CSMessageRequest(
      "icon/abc",
      ctx.dstNetworkId,
      nextSequenceNo,
      MessageType.CallMessagePersisted,
      new Uint8Array([0, 1, 2, 3]),
      [connectionProgram.programId.toString()]
    );

    let result = new CSMessageResult(
      nextSequenceNo,
      responseCode,
      request.encode()
    );
    let csMessage = new CSMessage(
      CSMessageType.CSMessageResult,
      result.encode()
    ).encode();

    await ctx.program.methods
      .recvMessage(
        ctx.dstNetworkId,
        new anchor.BN(connSn),
        Buffer.from(csMessage),
        new anchor.BN(nextSequenceNo)
      )
      .accountsStrict({
        config: ConnectionPDA.config().pda,
        admin: ctx.admin.publicKey,
        receipt: ConnectionPDA.receipt(connSn).pda,
        systemProgram: SYSTEM_PROGRAM_ID,
      })
      .remainingAccounts([
        {
          pubkey: XcallPDA.config().pda,
          isSigner: false,
          isWritable: true,
        },
        {
          pubkey: xcallCtx.admin.publicKey,
          isSigner: false,
          isWritable: true,
        },
        {
          pubkey: XcallPDA.defaultConnection(ctx.dstNetworkId).pda,
          isSigner: false,
          isWritable: true,
        },
        {
          pubkey: XcallPDA.proxyRequest(nextReqId).pda,
          isSigner: false,
          isWritable: true,
        },
        {
          pubkey: xcallProgram.programId,
          isSigner: false,
          isWritable: true,
        },
        {
          pubkey: xcallProgram.programId,
          isSigner: false,
          isWritable: true,
        },
        {
          pubkey: XcallPDA.successRes(nextSequenceNo).pda,
          isSigner: false,
          isWritable: true,
        },
        {
          pubkey: XcallPDA.rollback(nextSequenceNo).pda,
          isSigner: false,
          isWritable: true,
        },
      ])
      .signers([ctx.admin])
      .rpc();
    await sleep(2);

    assert.equal((await xcallCtx.getSuccessRes(nextSequenceNo)).success, true);

    try {
      await xcallCtx.getRollback(nextSequenceNo);
    } catch (err) {
      expect(err.message).to.includes("Account does not exist");
    }
  });

  it("[recv_message]: should receive message and call xcall handle message resultt", async () => {
    // send rollback message
    let envelope = new Envelope(
      MessageType.CallMessageWithRollback,
      new CallMessageWithRollback(
        new Uint8Array([1, 2, 3]),
        new Uint8Array([1, 2, 3])
      ).encode(),
      [connectionProgram.programId.toString()],
      [wallet.publicKey.toString()]
    ).encode();
    const to = { "0": "icon/abc" };

    let xcallConfig = await xcallCtx.getConfig();
    let nextSequenceNo = xcallConfig.sequenceNo.toNumber() + 1;
    let nextReqId = xcallConfig.lastReqId.toNumber() + 1;

    let sendCallIx = await xcallProgram.methods
      .sendCall(Buffer.from(envelope), to)
      .accountsStrict({
        systemProgram: SYSTEM_PROGRAM_ID,
        config: XcallPDA.config().pda,
        signer: wallet.payer.publicKey,
        rollbackAccount: XcallPDA.rollback(nextSequenceNo).pda,
        feeHandler: xcallCtx.feeHandler.publicKey,
        defaultConnection: XcallPDA.defaultConnection(ctx.dstNetworkId).pda,
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
        {
          pubkey: ConnectionPDA.claimFees().pda,
          isSigner: false,
          isWritable: true,
        },
      ])
      .instruction();

    let sendCallTx = await txnHelpers.buildV0Txn([sendCallIx], [wallet.payer]);
    await connection.sendTransaction(sendCallTx);
    await sleep(2);

    // receive message of rollback message
    let connSn = 3;

    let result = new CSMessageResult(
      nextSequenceNo,
      CSResponseType.CSMessageFailure,
      new Uint8Array([])
    );
    let csMessage = new CSMessage(
      CSMessageType.CSMessageResult,
      result.encode()
    ).encode();

    let recvMessageIx = await ctx.program.methods
      .recvMessage(
        ctx.dstNetworkId,
        new anchor.BN(connSn),
        Buffer.from(csMessage),
        new anchor.BN(nextSequenceNo)
      )
      .accountsStrict({
        config: ConnectionPDA.config().pda,
        admin: ctx.admin.publicKey,
        receipt: ConnectionPDA.receipt(connSn).pda,
        systemProgram: SYSTEM_PROGRAM_ID,
      })
      .remainingAccounts([
        {
          pubkey: XcallPDA.config().pda,
          isSigner: false,
          isWritable: true,
        },
        {
          pubkey: xcallCtx.admin.publicKey,
          isSigner: false,
          isWritable: true,
        },
        {
          pubkey: XcallPDA.defaultConnection(ctx.dstNetworkId).pda,
          isSigner: false,
          isWritable: true,
        },
        {
          pubkey: XcallPDA.proxyRequest(nextReqId).pda,
          isSigner: false,
          isWritable: true,
        },
        {
          pubkey: xcallProgram.programId,
          isSigner: false,
          isWritable: true,
        },
        {
          pubkey: xcallProgram.programId,
          isSigner: false,
          isWritable: true,
        },
        {
          pubkey: XcallPDA.successRes(nextSequenceNo).pda,
          isSigner: false,
          isWritable: true,
        },
        {
          pubkey: XcallPDA.rollback(nextSequenceNo).pda,
          isSigner: false,
          isWritable: true,
        },
      ])
      .instruction();

    let recvMessageTx = await txnHelpers.buildV0Txn(
      [recvMessageIx],
      [ctx.admin]
    );
    await connection.sendTransaction(recvMessageTx);
    await sleep(2);

    let rollback = await xcallCtx.getRollback(nextSequenceNo);
    assert.equal(rollback.rollback.enabled, true);

    // let executeRollbackIx = await xcallProgram.methods.executeRollback()
  });

  it("[revert_message]: should fail if not called by an admin", async () => {
    let fromNetwork = ctx.dstNetworkId;
    let sequenceNo = 1;

    try {
      await connectionProgram.methods
        .revertMessage(fromNetwork, new anchor.BN(sequenceNo))
        .accountsStrict({
          config: ConnectionPDA.config().pda,
          admin: ctx.signer.publicKey,
          systemProgram: SYSTEM_PROGRAM_ID,
        })
        .remainingAccounts([])
        .signers([ctx.signer])
        .rpc();
    } catch (err) {
      expect(err.message).includes("Only admin");
    }
  });

  it("[revert_message]: should revert message and call xcall handle error", async () => {
    let fromNetwork = ctx.dstNetworkId;

    let xcallConfig = await xcallCtx.getConfig();
    let nextSequenceNo = xcallConfig.sequenceNo.toNumber() + 1;

    // send rollback message
    let envelope = new Envelope(
      MessageType.CallMessageWithRollback,
      new CallMessageWithRollback(
        new Uint8Array([1, 2, 3]),
        new Uint8Array([1, 2, 3, 4, 5])
      ).encode(),
      [connectionProgram.programId.toString()],
      [wallet.publicKey.toString()]
    ).encode();
    const to = { "0": "icon/abc" };

    let sendCallIx = await xcallProgram.methods
      .sendCall(Buffer.from(envelope), to)
      .accountsStrict({
        systemProgram: SYSTEM_PROGRAM_ID,
        config: XcallPDA.config().pda,
        signer: wallet.payer.publicKey,
        rollbackAccount: XcallPDA.rollback(nextSequenceNo).pda,
        feeHandler: xcallCtx.feeHandler.publicKey,
        defaultConnection: XcallPDA.defaultConnection(ctx.dstNetworkId).pda,
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
        {
          pubkey: ConnectionPDA.claimFees().pda,
          isSigner: false,
          isWritable: true,
        },
      ])
      .instruction();

    let sendCallTx = await txnHelpers.buildV0Txn([sendCallIx], [wallet.payer]);
    await connection.sendTransaction(sendCallTx);
    await sleep(2);

    expect(await xcallCtx.getRollback(nextSequenceNo)).to.not.be.empty;

    let messageSeed = Buffer.from(
      hash(new Uint8Array([195, nextSequenceNo, 0, 128])),
      "hex"
    );

    let revertMessageIx = await connectionProgram.methods
      .revertMessage(fromNetwork, new anchor.BN(nextSequenceNo))
      .accountsStrict({
        config: ConnectionPDA.config().pda,
        admin: ctx.admin.publicKey,
        systemProgram: SYSTEM_PROGRAM_ID,
      })
      .remainingAccounts([
        {
          pubkey: XcallPDA.config().pda,
          isSigner: false,
          isWritable: true,
        },
        {
          pubkey: xcallCtx.admin.publicKey,
          isSigner: false,
          isWritable: true,
        },
        {
          pubkey: XcallPDA.defaultConnection(ctx.dstNetworkId).pda,
          isSigner: false,
          isWritable: true,
        },
        {
          pubkey: XcallPDA.pendingResponse(messageSeed).pda,
          isSigner: false,
          isWritable: true,
        },
        {
          pubkey: XcallPDA.rollback(nextSequenceNo).pda,
          isSigner: false,
          isWritable: true,
        },
        {
          pubkey: xcallProgram.programId,
          isSigner: false,
          isWritable: true,
        },
      ])
      .instruction();

    let revertMessageTx = await txnHelpers.buildV0Txn(
      [revertMessageIx],
      [ctx.admin]
    );
    await connection.sendTransaction(revertMessageTx);
    await sleep(2);

    let rollback = await xcallCtx.getRollback(nextSequenceNo);
    assert.equal(rollback.rollback.enabled, true);
  });
});
