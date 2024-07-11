import * as anchor from "@coral-xyz/anchor";
import { assert, expect } from "chai";
import { Keypair } from "@solana/web3.js";

import { TestContext, DappPDA } from "./setup";
import { TxnHelpers, sleep } from "../utils";
import { SYSTEM_PROGRAM_ID } from "@coral-xyz/anchor/dist/cjs/native/system";
import { TestContext as XcallTestCtx, XcallPDA } from "../xcall/setup";

import { PublicKey } from "@solana/web3.js";

import { Xcall } from "../../target/types/xcall";
import { Envelope, CallMessage, MessageType } from "../xcall/types";

import { CentralizedConnection } from "../../target/types/centralized_connection";

import { ConnectionPDA } from "../centralized-connection/setup";
import { DappMulti } from "../../target/types/dapp_multi";

const connectionProgram: anchor.Program<CentralizedConnection> =
  anchor.workspace.CentralizedConnection;

const xcallProgram: anchor.Program<Xcall> = anchor.workspace.Xcall;
const dappProgram: anchor.Program<DappMulti> = anchor.workspace.DappMulti;

describe("Mock Dapp", () => {
  const provider = anchor.AnchorProvider.env();
  const connection = provider.connection;
  const wallet = provider.wallet as anchor.Wallet;

  let txnHelpers = new TxnHelpers(connection, wallet.payer);
  let ctx = new TestContext(connection, txnHelpers, wallet.payer);

  const airdrop = async (publicKey: anchor.web3.PublicKey) => {
    const airdropSignature = await provider.connection.requestAirdrop(
      publicKey,
      anchor.web3.LAMPORTS_PER_SOL // Adjust amount as necessary
    );
    await provider.connection.confirmTransaction(airdropSignature);
  };

  const getTxnLogs = async (tx) => {
    const confirmation = await provider.connection.confirmTransaction(
      tx,
      "confirmed"
    );
    console.log("Transaction confirmation status:", confirmation.value.err);

    let txDetails = await provider.connection.getTransaction(tx, {
      commitment: "confirmed",
    });

    if (txDetails?.meta?.logMessages) {
      txDetails.meta.logMessages.forEach((log) => {
        console.log("Log:", log);
      });
    }
  };

  const getBalance = async (acc: PublicKey) => {
    let balance = await provider.connection.getBalance(acc);
    console.log("Account Balance is: ", balance);
  };

  it("should initialize dapp", async () => {
    let newAdmin = Keypair.generate();
    // xcall address is passed in argument in setup.ts
    await ctx.initialize();
    await sleep(3);

    let { xcallAddress } = await ctx.getConfig();
    let config;

    assert.equal(xcallProgram.programId.toString(), xcallAddress.toString());
  });

  it("should add connection to dapp", async () => {
    let newAdmin = Keypair.generate();
    // xcall address is passed in argument in setup.ts
    // const src_endpoint= "src" ;
    const src_endpoint = connectionProgram.programId.toString();
    const dst_endpoint = "dst";

    let connectionsPDA = DappPDA.connections(ctx.networkId).pda;

    const result = await ctx.add_connection(
      ctx.networkId,
      src_endpoint,
      dst_endpoint
    );

    await sleep(3);

    let connections = await dappProgram.account.connections.fetch(
      connectionsPDA
    );

    console.log("connectioins : " , connections.connections[0]);

    assert.equal(connections.connections[0].dstEndpoint, dst_endpoint);
    assert.equal(connections.connections[0].srcEndpoint, src_endpoint);
  });

  it("should send message", async () => {
    let xcall_context = new XcallTestCtx(connection, txnHelpers, wallet.payer);
    // await ctx.initialize();

    // set default connection
    let result = await xcall_context.setDefaultConnection(xcall_context.networkId , connectionProgram.programId);

    console.log("default_connection in xcall " , await xcall_context.getDefaultConnection(xcall_context.networkId))
    
    let envelope = new Envelope(
      MessageType.CallMessage,
      new CallMessage(new Uint8Array([])).encode(),
      [connectionProgram.programId.toString()],
      [wallet.publicKey.toString()]
    ).encode();
    const to = { "0": "icon/abc" };
    const msg_type = 0;
    const rollback = Buffer.from("rollback");
    const message = Buffer.from(envelope);


    let remaining_accounts = [
      {
        pubkey: XcallPDA.config().pda,
        isSigner: false,
        isWritable: true,
      },
      {
        pubkey: XcallPDA.reply().pda,
        isSigner: false,
        isWritable: true,
      },
      {
        pubkey: XcallPDA.rollback(
          (await xcall_context.getConfig()).sequenceNo.toNumber() + 1
        ).pda,
        isSigner: false,
        isWritable: true,
      },
      {
        pubkey: XcallPDA.defaultConnection(ctx.networkId).pda,
        isSigner: false,
        isWritable: true,
      },
      {
        pubkey: (await xcall_context.getConfig()).feeHandler,
        isSigner: false,
        isWritable: true,
      },
      //centralized connection accounts into remaining accounts that is slpited in contract
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
        pubkey: ConnectionPDA.network_fee(ctx.networkId).pda,
        isSigner: false,
        isWritable: true,
      },
      {
        pubkey: ConnectionPDA.claimFees().pda,
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
    ];


   
    let sendCallIx = await dappProgram.methods
      .sendCallMessage( to, message, msg_type, rollback)
      .accountsStrict({
        config: DappPDA.config().pda,
        systemProgram: SYSTEM_PROGRAM_ID,
        connectionsAccount: DappPDA.connections(ctx.networkId).pda,
        sender: wallet.payer.publicKey,
      })
      .remainingAccounts(remaining_accounts)
      .instruction();

    let sendCallTx = await txnHelpers.buildV0Txn([sendCallIx], [wallet.payer]);

    let sendCallTxSignature = await connection.sendTransaction(sendCallTx);
    // await txnHelpers.logParsedTx(sendCallTxSignature);
  });
});
