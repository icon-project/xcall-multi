import * as anchor from "@coral-xyz/anchor";
import {
  PublicKey,
  Connection,
  SystemProgram,
  TransactionMessage,
  VersionedTransaction,
} from "@solana/web3.js";

import { CentralizedConnection } from "../../target/types/centralized_connection";

export class TestContext {
  program: anchor.Program<CentralizedConnection>;
  signer: anchor.Wallet;
  admin: anchor.Wallet;
  connection: Connection;
  networkId: string;

  constructor() {
    let provider = anchor.AnchorProvider.env();
    anchor.setProvider(provider);

    this.program = anchor.workspace.CentralizedConnection;
    this.signer = provider.wallet as anchor.Wallet;
    this.admin = provider.wallet as anchor.Wallet;
    this.connection = new Connection("http://127.0.0.1:8899", "processed");
    this.networkId = "icx";
  }

  async initialize() {
    await this.program.methods
      .initialize(this.signer.publicKey, this.signer.publicKey)
      .signers([this.signer.payer])
      .accounts({})
      .rpc();
  }

  async setAdmin(pubkey: PublicKey) {
    await this.program.methods
      .setAdmin(pubkey)
      .accounts({})
      .signers([this.signer.payer])
      .rpc();
  }

  async getConfigAccount() {
    let [config_account] = PublicKey.findProgramAddressSync(
      [Buffer.from("config")],
      this.program.programId
    );

    let { data } = await this.program.account.config.fetchAndContext(
      config_account
    );

    return data;
  }

  async getFeeAccount() {
    let [fee_account] = PublicKey.findProgramAddressSync(
      [Buffer.from("fee"), Buffer.from(this.networkId)],
      this.program.programId
    );

    let { data } = await this.program.account.fee.fetchAndContext(fee_account);

    return data;
  }

  getClaimFeesAddress() {
    let [claimFees] = PublicKey.findProgramAddressSync(
      [Buffer.from("claim_fees")],
      this.program.programId
    );

    return claimFees;
  }

  sleep(ms: number) {
    return new Promise((resolve) => setTimeout(resolve, ms));
  }

  async transferLamports(to: PublicKey, lamports: number) {
    let recentBlock = await this.connection.getLatestBlockhash();

    let transfer_to_pda = SystemProgram.transfer({
      lamports,
      fromPubkey: this.signer.publicKey,
      toPubkey: to,
      programId: SystemProgram.programId,
    });

    const message = new TransactionMessage({
      payerKey: this.signer.publicKey,
      recentBlockhash: recentBlock.blockhash,
      instructions: [transfer_to_pda],
    }).compileToV0Message();

    let tx = new VersionedTransaction(message);
    tx.sign([this.signer.payer]);

    const signature = await this.connection.sendTransaction(tx);
    return signature;
  }
}
