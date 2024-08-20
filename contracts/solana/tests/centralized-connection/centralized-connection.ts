import * as anchor from "@coral-xyz/anchor";
import { assert, expect } from "chai";
import { Keypair, PublicKey } from "@solana/web3.js";

import { TestContext, ConnectionPDA } from "./setup";
import { SYSVAR_INSTRUCTIONS_ID, TxnHelpers, hash, sleep } from "../utils";
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
  MessageType,
} from "../xcall/types";
import { TestContext as XcallTestContext } from "../xcall/setup";

import { MockDappMulti } from "../../target/types/mock_dapp_multi";
import { DappPDA, TestContext as MockDappCtx } from "../mock-dapp-multi/setup";

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

  let mockDappCtx = new MockDappCtx(connection, txnHelpers, ctx.admin);

  before(async () => {
    await ctx.setNetworkFee("icon", 50, 50);
    sleep(2);
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

    await ctx.setNetworkFee(ctx.dstNetworkId, msg_fee, res_fee);
    await sleep(2);

    let fee = await ctx.getFee(ctx.dstNetworkId);
    assert.equal(fee.messageFee.toNumber(), msg_fee);
    assert.equal(fee.responseFee.toNumber(), res_fee);
  });

  it("[claim_fees]: should claim fee stored in PDA account", async () => {
    let config = ConnectionPDA.config().pda;

    let transfer_amount = 500_000;
    await txnHelpers.airdrop(config, transfer_amount);
    await sleep(2);

    const min_rent_exempt_balance =
      await ctx.connection.getMinimumBalanceForRentExemption(90);
    const before_pda_balance = (await ctx.connection.getAccountInfo(config))
      .lamports;
    assert.equal(min_rent_exempt_balance + transfer_amount, before_pda_balance);

    await ctx.program.methods
      .claimFees()
      .accountsStrict({
        admin: ctx.admin.publicKey,
        config: ConnectionPDA.config().pda,
      })
      .signers([ctx.admin])
      .rpc();

    const after_pda_balance = (await ctx.connection.getAccountInfo(config))
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
          authority: ConnectionPDA.authority().pda,
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
    const fromNetwork = "icon";
    let nextReqId = xcallConfig.lastReqId.toNumber() + 1;
    let nextSequenceNo = xcallConfig.sequenceNo.toNumber() + 1;

    let data = Buffer.from("rollback", "utf-8");
    let request = new CSMessageRequest(
      "icon/abc",
      mockDappProgram.programId.toString(),
      nextSequenceNo,
      MessageType.CallMessageWithRollback,
      data,
      [connectionProgram.programId.toString()]
    );

    let cs_message = new CSMessage(
      CSMessageType.CSMessageRequest,
      request.encode()
    ).encode();

    let recvMessageAccounts = await ctx.getRecvMessageAccounts(
      connSn,
      nextSequenceNo,
      cs_message,
      CSMessageType.CSMessageRequest
    );

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
        authority: ConnectionPDA.authority().pda,
        systemProgram: SYSTEM_PROGRAM_ID,
      })
      .remainingAccounts([...recvMessageAccounts.slice(4)])
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
      Buffer.from(hash(data), "hex").toString()
    );

    // expect request to be increased in xcall config PDA's
    expect((await xcallCtx.getConfig()).lastReqId.toString()).to.equal(
      nextReqId.toString()
    );

    // call xcall execute_call
    let executeCallAccounts = await xcallCtx.getExecuteCallAccounts(
      nextReqId,
      data
    );

    await xcallProgram.methods
      .executeCall(new anchor.BN(nextReqId), Buffer.from(data))
      .accounts({
        signer: ctx.admin.publicKey,
        systemProgram: SYSTEM_PROGRAM_ID,
        config: XcallPDA.config().pda,
        admin: xcallConfig.admin,
        proxyRequest: XcallPDA.proxyRequest(nextReqId).pda,
      })
      .remainingAccounts([...executeCallAccounts.slice(4)])
      .signers([ctx.admin])
      .rpc();
  });

  it("[recv_message]: should receive message and call xcall handle message result", async () => {
    // send rollback message using mock dapp
    await mockDappCtx.add_connection(
      "icon",
      connectionProgram.programId.toString(),
      connectionProgram.programId.toString()
    );
    await sleep(2);

    const to = { "0": "icon/abc" };
    let data = Buffer.from("rollback", "utf-8");
    let msgType = MessageType.CallMessageWithRollback;

    let xcallConfig = await xcallCtx.getConfig();
    let nextSequenceNo = xcallConfig.sequenceNo.toNumber() + 1;

    let sendCallIx = await mockDappProgram.methods
      .sendCallMessage(to, data, msgType, data)
      .accountsStrict({
        config: DappPDA.config().pda,
        systemProgram: SYSTEM_PROGRAM_ID,
        connectionsAccount: DappPDA.connections("icon").pda,
        sender: ctx.admin.publicKey,
        authority: DappPDA.authority().pda,
      })
      .remainingAccounts([
        {
          pubkey: SYSVAR_INSTRUCTIONS_ID,
          isSigner: false,
          isWritable: false,
        },
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
          pubkey: ConnectionPDA.network_fee("icon").pda,
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

    // receive message of rollback message
    let connSn = 2;
    let responseCode = CSResponseType.CSResponseSuccess;

    let request = new CSMessageRequest(
      "icon/abc",
      "icon",
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

    let recvMessageAccounts = await ctx.getRecvMessageAccounts(
      connSn,
      nextSequenceNo,
      csMessage,
      CSMessageType.CSMessageResult
    );

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
        authority: ConnectionPDA.authority().pda,
        systemProgram: SYSTEM_PROGRAM_ID,
      })
      .remainingAccounts([...recvMessageAccounts.slice(4)])
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

  it("[recv_message]: should receive message and execute rollback", async () => {
    let data = Buffer.from("rollback_data", "utf-8");

    let xcallConfig = await xcallCtx.getConfig();
    let nextSequenceNo = xcallConfig.sequenceNo.toNumber() + 1;

    const to = { "0": "0x3.icon/" + mockDappProgram.programId.toString() };
    const msg_type = MessageType.CallMessageWithRollback;

    let sendCallIx = await mockDappProgram.methods
      .sendCallMessage(to, data, msg_type, data)
      .accountsStrict({
        config: DappPDA.config().pda,
        systemProgram: SYSTEM_PROGRAM_ID,
        connectionsAccount: DappPDA.connections(ctx.dstNetworkId).pda,
        sender: ctx.admin.publicKey,
        authority: DappPDA.authority().pda,
      })
      .remainingAccounts([
        {
          pubkey: SYSVAR_INSTRUCTIONS_ID,
          isSigner: false,
          isWritable: false,
        },
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

    let recvMessageAccounts = await ctx.getRecvMessageAccounts(
      connSn,
      nextSequenceNo,
      csMessage,
      CSMessageType.CSMessageResult
    );

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
        authority: ConnectionPDA.authority().pda,
        systemProgram: SYSTEM_PROGRAM_ID,
      })
      .remainingAccounts([...recvMessageAccounts.slice(4)])
      .instruction();

    let recvMessageTx = await txnHelpers.buildV0Txn(
      [recvMessageIx],
      [ctx.admin]
    );
    await connection.sendTransaction(recvMessageTx);
    await sleep(2);

    // execute rollback message
    let rollbackAccount = await xcallCtx.getRollback(nextSequenceNo);
    assert.equal(rollbackAccount.rollback.enabled, true);

    let executeRollbackAccounts = await xcallCtx.getExecuteRollbackAccounts(
      nextSequenceNo
    );

    let executeRollbackIx = await xcallProgram.methods
      .executeRollback(new anchor.BN(nextSequenceNo))
      .accountsStrict({
        signer: ctx.admin.publicKey,
        systemProgram: SYSTEM_PROGRAM_ID,
        config: XcallPDA.config().pda,
        admin: xcallConfig.admin,
        rollbackAccount: XcallPDA.rollback(nextSequenceNo).pda,
      })
      .remainingAccounts([...executeRollbackAccounts.slice(4)])
      .instruction();

    let executeRollbackTx = await txnHelpers.buildV0Txn(
      [executeRollbackIx],
      [ctx.admin]
    );
    await connection.sendTransaction(executeRollbackTx);
  });

  it("[revert_message]: should fail if not called by an admin", async () => {
    let fromNetwork = ctx.dstNetworkId;
    let sequenceNo = 1;

    try {
      await connectionProgram.methods
        .revertMessage(new anchor.BN(sequenceNo))
        .accountsStrict({
          config: ConnectionPDA.config().pda,
          admin: ctx.signer.publicKey,
          authority: ConnectionPDA.authority().pda,
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
    let xcallConfig = await xcallCtx.getConfig();
    let nextSequenceNo = xcallConfig.sequenceNo.toNumber() + 1;

    let data = Buffer.from("rollback", "utf-8");
    const to = { "0": "0x3.icon/" + mockDappProgram.programId.toString() };

    const msg_type = MessageType.CallMessageWithRollback;

    // send rollback message using mock dapp
    let sendCallIx = await mockDappProgram.methods
      .sendCallMessage(to, data, msg_type, data)
      .accountsStrict({
        config: DappPDA.config().pda,
        systemProgram: SYSTEM_PROGRAM_ID,
        connectionsAccount: DappPDA.connections(ctx.dstNetworkId).pda,
        sender: ctx.admin.publicKey,
        authority: DappPDA.authority().pda,
      })
      .remainingAccounts([
        {
          pubkey: SYSVAR_INSTRUCTIONS_ID,
          isSigner: false,
          isWritable: false,
        },
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
      ])
      .instruction();

    let sendCallTx = await txnHelpers.buildV0Txn([sendCallIx], [ctx.admin]);
    await connection.sendTransaction(sendCallTx);
    await sleep(2);

    expect(await xcallCtx.getRollback(nextSequenceNo)).to.not.be.empty;

    let revertMessageAccounts = await ctx.getRevertMessageAccounts(
      nextSequenceNo
    );

    let revertMessageIx = await connectionProgram.methods
      .revertMessage(new anchor.BN(nextSequenceNo))
      .accountsStrict({
        config: ConnectionPDA.config().pda,
        admin: ctx.admin.publicKey,
        authority: ConnectionPDA.authority().pda,
        systemProgram: SYSTEM_PROGRAM_ID,
      })
      .remainingAccounts([...revertMessageAccounts.slice(3)])
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
