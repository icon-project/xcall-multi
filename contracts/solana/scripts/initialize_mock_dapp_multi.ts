import * as anchor from "@coral-xyz/anchor";
import { MockDappMulti } from "../target/types/mock_dapp_multi"; 
import fs from 'fs';
import { homedir } from 'os';
import { join } from 'path';

const args = process.argv.slice(2);

const xcall_param = args[0];
const environment = args[1]


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

    async initialize(xcall: anchor.web3.PublicKey) {
      await this.program.rpc.initialize(xcall, {
        accounts: {
          config: (await anchor.web3.PublicKey.findProgramAddress(
            [Buffer.from("config")],
            this.program.programId
          ))[0],
          sender: this.wallet.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
        },
        signers: [this.wallet.payer],
      });
    }

    async getConfig() {
      const configKey = (await anchor.web3.PublicKey.findProgramAddress(
        [Buffer.from("config")],
        this.program.programId
      ))[0];
      return this.program.account.config.fetch(configKey);
    }
  }

  const mockDappCtx = new MockDappContext(connection, wallet, mockDappProgram);

  const xcall = new anchor.web3.PublicKey(xcall_param)

  console.log("Initializing mock dapp multi")

  await mockDappCtx.initialize(xcall);
  
  // Fetch and check the configuration
  const data = await mockDappCtx.getConfig();

  console.log("Mock Dapp Multi program initialized successfully.");
})().catch(err => {
  console.error(err);
});
