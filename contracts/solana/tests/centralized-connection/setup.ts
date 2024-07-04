import * as anchor from "@coral-xyz/anchor";
import { PublicKey, Connection, Keypair } from "@solana/web3.js";

import { CentralizedConnection } from "../../target/types/centralized_connection";
import { SYSTEM_PROGRAM_ID } from "@coral-xyz/anchor/dist/cjs/native/system";
import { TxnHelpers, uint128ToArray } from "../utils";

import { Xcall } from "../../target/types/xcall";

const xcallProgram: anchor.Program<Xcall> = anchor.workspace.Xcall;

const connectionProgram: anchor.Program<CentralizedConnection> =
  anchor.workspace.CentralizedConnection;

export class TestContext {
  program: anchor.Program<CentralizedConnection>;
  signer: Keypair;
  admin: Keypair;
  connection: Connection;
  networkId: string;
  txnHelpers: TxnHelpers;
  isInitialized: boolean;

  constructor(connection: Connection, txnHelpers: TxnHelpers, admin: Keypair) {
    let provider = anchor.AnchorProvider.env();
    anchor.setProvider(provider);

    this.program = anchor.workspace.CentralizedConnection;
    this.signer = admin;
    this.admin = admin;
    this.connection = connection;
    this.txnHelpers = txnHelpers;
    this.networkId = "icx";
    this.isInitialized = false;
  }

  async initialize() {
    await this.program.methods
      .initialize(xcallProgram.programId, this.signer.publicKey)
      .signers([this.signer])
      .accountsStrict({
        signer: this.signer.publicKey,
        systemProgram: SYSTEM_PROGRAM_ID,
        config: ConnectionPDA.config().pda,
        claimFee: ConnectionPDA.claimFees().pda,
      })
      .rpc();

    this.isInitialized = true;
  }

  async setAdmin(keypair: Keypair) {
    await this.program.methods
      .setAdmin(keypair.publicKey)
      .accountsStrict({
        admin: this.admin.publicKey,
        config: ConnectionPDA.config().pda,
      })
      .signers([this.admin])
      .rpc();

    this.admin = keypair;
  }

  async getConfig() {
    return await this.program.account.config.fetch(
      ConnectionPDA.config().pda,
      "confirmed"
    );
  }

  async getFee(nid: string) {
    return await this.program.account.networkFee.fetch(
      ConnectionPDA.fee(nid).pda,
      "confirmed"
    );
  }

  async getReceipt(sequenceNo: number) {
    return await this.program.account.receipt.fetch(
      ConnectionPDA.receipt(sequenceNo).pda,
      "confirmed"
    );
  }
}

export class ConnectionPDA {
  constructor() {}

  static config() {
    let [pda, bump] = PublicKey.findProgramAddressSync(
      [Buffer.from("config")],
      connectionProgram.programId
    );

    return { bump, pda };
  }

  static fee(networkId: string) {
    const [pda, bump] = PublicKey.findProgramAddressSync(
      [Buffer.from("fee"), Buffer.from(networkId)],
      connectionProgram.programId
    );

    return { pda, bump };
  }

  static claimFees() {
    const [pda, bump] = PublicKey.findProgramAddressSync(
      [Buffer.from("claim_fees")],
      connectionProgram.programId
    );

    return { pda, bump };
  }

  static receipt(sn: number) {
    const [pda, bump] = PublicKey.findProgramAddressSync(
      [Buffer.from("receipt"), uint128ToArray(sn)],
      connectionProgram.programId
    );

    return { pda, bump };
  }
}
