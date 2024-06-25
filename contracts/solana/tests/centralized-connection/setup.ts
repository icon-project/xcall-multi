import * as anchor from "@coral-xyz/anchor";
import { PublicKey, Connection, Keypair } from "@solana/web3.js";

import { CentralizedConnection } from "../../target/types/centralized_connection";
import { SYSTEM_PROGRAM_ID } from "@coral-xyz/anchor/dist/cjs/native/system";
import { Xcall } from "../../target/types/xcall";

const connectionProgram: anchor.Program<CentralizedConnection> =
  anchor.workspace.CentralizedConnection;

const xcallProgram: anchor.Program<Xcall> = anchor.workspace.Xcall;

export class TestContext {
  program: anchor.Program<CentralizedConnection>;
  signer: anchor.Wallet;
  admin: Keypair;
  connection: Connection;
  networkId: string;

  constructor() {
    let provider = anchor.AnchorProvider.env();
    anchor.setProvider(provider);

    this.program = anchor.workspace.CentralizedConnection;
    this.signer = provider.wallet as anchor.Wallet;
    this.admin = (provider.wallet as anchor.Wallet).payer;
    this.connection = new Connection("http://127.0.0.1:8899", "processed");
    this.networkId = "icx";
  }

  async initialize() {
    await this.program.methods
      .initialize(xcallProgram.programId, this.signer.publicKey)
      .signers([this.signer.payer])
      .accountsStrict({
        signer: this.signer.publicKey,
        systemProgram: SYSTEM_PROGRAM_ID,
        config: connectionPDA.config().pda,
        claimFee: connectionPDA.claimFees().pda,
      })
      .rpc();
  }

  async setAdmin(keypair: Keypair) {
    await this.program.methods
      .setAdmin(keypair.publicKey)
      .accountsStrict({
        admin: this.admin.publicKey,
        config: connectionPDA.config().pda,
      })
      .signers([this.admin])
      .rpc();

    this.admin = keypair;
  }

  async getConfig() {
    return await this.program.account.config.fetch(
      connectionPDA.config().pda,
      "confirmed"
    );
  }

  async getFee(nid: string) {
    return await this.program.account.fee.fetch(
      connectionPDA.fee(nid).pda,
      "confirmed"
    );
  }
}

export class connectionPDA {
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
}
