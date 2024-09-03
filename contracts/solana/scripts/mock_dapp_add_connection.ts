import * as anchor from "@coral-xyz/anchor";
import { MockDappMulti } from "../target/types/mock_dapp_multi"; 
import { PublicKey } from "@solana/web3.js";

import fs from 'fs';
import { homedir } from 'os';
import { join } from 'path';

const args = process.argv.slice(2);

const environment = args[0];
const network_id = args[1];
const solana_connection = args[2]
const icon_connection = args[3]

const sleep = (seconds: number) => {
  return new Promise((resolve) => setTimeout(resolve, seconds * 1000));
};

const mockDappProgram: anchor.Program<MockDappMulti> = anchor.workspace.MockDappMulti;

(async () => {
  const testnetRpcUrl = environment;
  const connection = new anchor.web3.Connection(testnetRpcUrl);

  // Load the default Solana CLI wallet
  const keypairPath = join(homedir(), '.config', 'solana', 'id.json');
  const keypairArray = JSON.parse(fs.readFileSync(keypairPath, 'utf-8'));
  const keypair = anchor.web3.Keypair.fromSecretKey(new Uint8Array(keypairArray));
  const wallet = new anchor.Wallet(keypair);

  // Airdrop SOL to the wallet
  const airdropSignature = await connection.requestAirdrop(wallet.publicKey, anchor.web3.LAMPORTS_PER_SOL);
  await connection.confirmTransaction(airdropSignature);

  const provider = new anchor.AnchorProvider(connection, wallet, {
    preflightCommitment: "recent",
  });
  anchor.setProvider(provider);

  class MockDappContext {
    connection: anchor.web3.Connection;
    wallet: anchor.Wallet;
    program: anchor.Program<MockDappMulti>;

    constructor(connection: anchor.web3.Connection, wallet: anchor.Wallet, program: anchor.Program<MockDappMulti>) {
      this.connection = connection;
      this.wallet = wallet;
      this.program = program;
    }


  async add_connection(
    _networkId: string,
    src_endpoint: string,
    dst_endpoint: string
  ) {

    const buffer1 = Buffer.from("connections");
    const buffer2 = Buffer.from(_networkId);
    const seed = [buffer1, buffer2];

    const [pda, bump] = PublicKey.findProgramAddressSync(
      seed,
      this.program.programId
    );
    const result = await this.program.methods
      .addConnection(_networkId, src_endpoint, dst_endpoint)
      .accounts({
        connectionAccount: pda,
        sender: this.wallet.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([this.wallet.payer])
      .rpc();

    return result;
  }
  }

  const mockDappCtx = new MockDappContext(connection, wallet, mockDappProgram);

  console.log("Adding connection to mock dapp multi")
  await mockDappCtx.add_connection(network_id , solana_connection , icon_connection )

  console.log(" Connection added successfully.");
})().catch(err => {
  console.error(err);
});

