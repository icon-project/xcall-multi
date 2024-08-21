import * as anchor from "@coral-xyz/anchor";
import { Xcall } from "../target/types/xcall"; 
import fs from 'fs';
import { homedir } from 'os';
import { join } from 'path';

const args = process.argv.slice(2);

const admin_address = args[0];
const environment = args[1]


 const sleep = (seconds: number) => {
  return new Promise((resolve) => setTimeout(resolve, seconds * 1000));
};

const xcallProgram: anchor.Program<Xcall> = anchor.workspace.Xcall;

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

  class XcallContext {
    connection: anchor.web3.Connection;
    wallet: anchor.Wallet;
    program: anchor.Program<Xcall>;

    constructor(connection: anchor.web3.Connection, wallet: anchor.Wallet, program: anchor.Program<Xcall>) {
      this.connection = connection;
      this.wallet = wallet;
      this.program = program;
    }

    async set_admin(admin: anchor.web3.PublicKey) {
      await this.program.rpc.setAdmin(admin, {
        accounts: {
          config: (await anchor.web3.PublicKey.findProgramAddress(
            [Buffer.from("config")],
            this.program.programId
          ))[0],
          admin: this.wallet.publicKey,
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

  const xcallCtx = new XcallContext(connection, wallet, xcallProgram);

  const admin = new anchor.web3.PublicKey(admin_address)
  console.log("setting xcall admin")

  await xcallCtx.set_admin(admin);
  
  // Fetch and check the configuration
  const data = await xcallCtx.getConfig();

  console.log("Xcall admin set successfully.");
})().catch(err => {
  console.error(err);
});
