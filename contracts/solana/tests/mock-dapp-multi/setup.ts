import * as anchor from "@coral-xyz/anchor";
import { PublicKey, Connection, Keypair } from "@solana/web3.js";

import { MockDappMulti } from "../../target/types/mock_dapp_multi";
import { SYSTEM_PROGRAM_ID } from "@coral-xyz/anchor/dist/cjs/native/system";
import { TxnHelpers } from "../utils";

import { Xcall } from "../../target/types/xcall";

const xcallProgram: anchor.Program<Xcall> = anchor.workspace.Xcall;

const dappProgram: anchor.Program<MockDappMulti> =
  anchor.workspace.MockDappMulti;

export class TestContext {
  program: anchor.Program<MockDappMulti>;
  signer: Keypair;
  admin: Keypair;
  connection: Connection;
  networkId: string;
  txnHelpers: TxnHelpers;
  isInitialized: boolean;

  constructor(connection: Connection, txnHelpers: TxnHelpers, admin: Keypair) {
    let provider = anchor.AnchorProvider.env();
    anchor.setProvider(provider);

    this.program = anchor.workspace.MockDappMulti;
    this.signer = admin;
    this.admin = admin;
    this.connection = connection;
    this.txnHelpers = txnHelpers;
    this.networkId = "icon";
    this.isInitialized = false;
  }

  async initialize() {
    await this.program.methods
      .initialize(xcallProgram.programId)
      .signers([this.signer])
      .accountsStrict({
        sender: this.signer.publicKey,
        systemProgram: SYSTEM_PROGRAM_ID,
        config: DappPDA.config().pda,
      })
      .rpc();

    this.isInitialized = true;
  }

  async add_connection(
    _networkId: string,
    src_endpoint: string,
    dst_endpoint: string
  ) {
    const result = await this.program.methods
      .addConnection(_networkId, src_endpoint, dst_endpoint)
      .accounts({
        connectionAccount: DappPDA.connections(this.networkId).pda,
        sender: this.signer.publicKey,
        systemProgram: SYSTEM_PROGRAM_ID,
      })
      .signers([this.admin])
      .rpc();

    return result;
  }

  async getConfig() {
    return await this.program.account.config.fetch(
      DappPDA.config().pda,
      "confirmed"
    );
  }
}

export class DappPDA {
  constructor() {}

  static config() {
    let [pda, bump] = PublicKey.findProgramAddressSync(
      [Buffer.from("config")],
      dappProgram.programId
    );

    return { bump, pda };
  }

  static connections(networkId: string) {
    const buffer1 = Buffer.from("connections");
    const buffer2 = Buffer.from(networkId);
    const seed = [buffer1, buffer2];

    const [pda, bump] = PublicKey.findProgramAddressSync(
      seed,
      dappProgram.programId
    );

    return { pda, bump };
  }
}
