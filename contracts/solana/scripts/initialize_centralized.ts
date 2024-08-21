import * as anchor from "@coral-xyz/anchor";
import { CentralizedConnection } from "../target/types/centralized_connection"; 
import fs from 'fs';
import { homedir } from 'os';
import { join } from 'path';

const args = process.argv.slice(2);

const xcall_param = args[0];
const admin_param = args[1];
const environment = args[2]


 const sleep = (seconds: number) => {
  return new Promise((resolve) => setTimeout(resolve, seconds * 1000));
};

const centralizedProgram: anchor.Program<CentralizedConnection> = anchor.workspace.CentralizedConnection;

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

  class CentralizedContext {
    connection: anchor.web3.Connection;
    wallet: anchor.Wallet;
    program: anchor.Program<CentralizedConnection>;

    constructor(connection: anchor.web3.Connection, wallet: anchor.Wallet, program: anchor.Program<CentralizedConnection>) {
      this.connection = connection;
      this.wallet = wallet;
      this.program = program;
    }

    async initialize(xcall: anchor.web3.PublicKey , admin: anchor.web3.PublicKey) {
      await this.program.rpc.initialize(xcall, admin, {
        accounts: {
          config: (await anchor.web3.PublicKey.findProgramAddress(
            [Buffer.from("config")],
            this.program.programId
          ))[0],
          signer: this.wallet.publicKey,
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

  const centralizedCtx = new CentralizedContext(connection, wallet, centralizedProgram);

  const admin = new anchor.web3.PublicKey(admin_param)
  const xcall = new anchor.web3.PublicKey(xcall_param)

  console.log("initializing centralized connection ")

  await centralizedCtx.initialize(xcall , admin);
  
  // Fetch and check the configuration
  const data = await centralizedCtx.getConfig();

  console.log("Centralized Connection program initialized successfully.");
})().catch(err => {
  console.error(err);
});
