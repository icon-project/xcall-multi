import * as anchor from "@coral-xyz/anchor";

import { PublicKey, Connection, Keypair } from "@solana/web3.js";
import { Xcall } from "../../target/types/xcall";
import { TxnHelpers, sleep, uint128ToArray } from "../utils";
import { SYSTEM_PROGRAM_ID } from "@coral-xyz/anchor/dist/cjs/native/system";

const xcallProgram: anchor.Program<Xcall> = anchor.workspace.Xcall;

import { DappPDA } from "../mock-dapp-multi/setup";
import { MockDappMulti } from "../../target/types/mock_dapp_multi";
const mockDappProgram: anchor.Program<MockDappMulti> =
  anchor.workspace.MockDappMulti;

import { ConnectionPDA } from "../centralized-connection/setup";
import { CentralizedConnection } from "../../target/types/centralized_connection";
const connectionProgram: anchor.Program<CentralizedConnection> =
  anchor.workspace.CentralizedConnection;

export class TestContext {
  networkId: string;
  dstNetworkId: string;
  admin: Keypair;
  feeHandler: Keypair;
  connection: Connection;
  txnHelpers: TxnHelpers;
  protocolFee: number;

  constructor(connection: Connection, txnHelpers: TxnHelpers, admin: Keypair) {
    this.networkId = "solana";
    this.dstNetworkId = "icon";
    this.connection = connection;
    this.txnHelpers = txnHelpers;
    this.admin = admin;
    this.feeHandler = admin;
  }

  async initialize(netId: string) {
    let initializeIx = await xcallProgram.methods
      .initialize(netId)
      .accountsStrict({
        signer: this.admin.publicKey,
        systemProgram: SYSTEM_PROGRAM_ID,
        config: XcallPDA.config().pda,
      })
      .instruction();

    let tx = await this.txnHelpers.buildV0Txn([initializeIx], [this.admin]);
    await this.connection.sendTransaction(tx);
    await sleep(2);
  }

  async setFeeHandler(fee_handler: Keypair) {
    this.feeHandler = fee_handler;

    let ix = await xcallProgram.methods
      .setProtocolFeeHandler(fee_handler.publicKey)
      .accountsStrict({
        admin: this.admin.publicKey,
        config: XcallPDA.config().pda,
      })
      .instruction();

    let tx = await this.txnHelpers.buildV0Txn([ix], [this.admin]);
    await this.connection.sendTransaction(tx);
    await sleep(2);
  }

  async setProtocolFee(fee: number) {
    this.protocolFee = fee;

    let ix = await xcallProgram.methods
      .setProtocolFee(new anchor.BN(fee))
      .accountsStrict({
        feeHandler: this.feeHandler.publicKey,
        config: XcallPDA.config().pda,
      })
      .instruction();

    let tx = await this.txnHelpers.buildV0Txn([ix], [this.feeHandler]);
    await this.connection.sendTransaction(tx);
    await sleep(2);
  }

  async getExecuteCallAccounts(reqId: number, data: Uint8Array) {
    const res = await xcallProgram.methods
      .queryExecuteCallAccounts(new anchor.BN(reqId), Buffer.from(data), 1, 30)
      .accountsStrict({
        config: XcallPDA.config().pda,
        proxyRequest: XcallPDA.proxyRequest(reqId).pda,
      })
      .remainingAccounts([
        {
          pubkey: ConnectionPDA.config().pda,
          isWritable: true,
          isSigner: false,
        },
        {
          pubkey: DappPDA.config().pda,
          isWritable: true,
          isSigner: false,
        },
        {
          pubkey: connectionProgram.programId,
          isWritable: true,
          isSigner: false,
        },
        {
          pubkey: mockDappProgram.programId,
          isWritable: true,
          isSigner: false,
        },
      ])
      .view({ commitment: "confirmed" });

    return res.accounts;
  }

  async getConfig() {
    let { pda } = XcallPDA.config();
    return await xcallProgram.account.config.fetch(pda);
  }

  async getProxyRequest(requestId: number) {
    return await xcallProgram.account.proxyRequest.fetch(
      XcallPDA.proxyRequest(requestId).pda,
      "confirmed"
    );
  }

  async getSuccessRes(sequenceNo: number) {
    return await xcallProgram.account.successfulResponse.fetch(
      XcallPDA.successRes(sequenceNo).pda,
      "confirmed"
    );
  }

  async getPendingRequest(messageBytes: Buffer) {
    return await xcallProgram.account.pendingRequest.fetch(
      XcallPDA.pendingRequest(messageBytes).pda,
      "confirmed"
    );
  }

  async getPendingResponse(messageBytes: Buffer) {
    return await xcallProgram.account.pendingResponse.fetch(
      XcallPDA.pendingResponse(messageBytes).pda,
      "confirmed"
    );
  }

  async getRollback(sequenceNo: number) {
    return await xcallProgram.account.rollbackAccount.fetch(
      XcallPDA.rollback(sequenceNo).pda,
      "confirmed"
    );
  }
}
export class XcallPDA {
  constructor() {}

  static config() {
    let [pda, bump] = PublicKey.findProgramAddressSync(
      [Buffer.from("config")],
      xcallProgram.programId
    );

    return { bump, pda };
  }

  static proxyRequest(requestId: number) {
    const [pda, bump] = PublicKey.findProgramAddressSync(
      [Buffer.from("proxy"), uint128ToArray(requestId)],
      xcallProgram.programId
    );

    return { pda, bump };
  }

  static successRes(sequenceNo: number) {
    const [pda, bump] = PublicKey.findProgramAddressSync(
      [Buffer.from("success"), uint128ToArray(sequenceNo)],
      xcallProgram.programId
    );

    return { pda, bump };
  }

  static defaultConnection(netId: String) {
    const [pda, bump] = PublicKey.findProgramAddressSync(
      [Buffer.from("conn"), Buffer.from(netId)],
      xcallProgram.programId
    );

    return { pda, bump };
  }

  static pendingRequest(messageBytes: Buffer) {
    const [pda, bump] = PublicKey.findProgramAddressSync(
      [Buffer.from("req"), messageBytes],
      xcallProgram.programId
    );

    return { pda, bump };
  }

  static pendingResponse(messageBytes: Buffer) {
    const [pda, bump] = PublicKey.findProgramAddressSync(
      [Buffer.from("res"), messageBytes],
      xcallProgram.programId
    );

    return { pda, bump };
  }

  static rollback(sequenceNo: number) {
    const [pda, bump] = PublicKey.findProgramAddressSync(
      [Buffer.from("rollback"), uint128ToArray(sequenceNo)],
      xcallProgram.programId
    );

    return { pda, bump };
  }
}
