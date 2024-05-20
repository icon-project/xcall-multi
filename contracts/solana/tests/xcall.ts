import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { PublicKey, SystemProgram } from '@solana/web3.js';
import { Xcall } from "../target/types/xcall";
import { CentralizedConnection } from "../target/types/centralized_connection";
import { assert } from "chai";
import * as rlp from 'rlp';

interface XCallEnvelope {
  msg_type: number;
  message: Uint8Array;
  sources: string[];
  destinations: string[];
}

function encodeXCallEnvelope(envelope: XCallEnvelope): string {
  // Convert the XCallEnvelope object to a format suitable for RLP encoding
  const rlpArray: any[] = [
    envelope.msg_type,
    Buffer.from(envelope.message),
    envelope.sources,
    envelope.destinations
  ];

  // Encode the array using RLP
  return uint8ArrayToHex(rlp.encode(rlpArray));
}

function uint8ArrayToHex(uint8Array: Uint8Array): string {
  return Array.from(uint8Array)
      .map(byte => byte.toString(16).padStart(2, '0'))
      .join('');
}

describe("xcall", () => {
  // Configure the client to use the local cluster.
  let provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const xcall = anchor.workspace.Xcall as Program<Xcall>;

  const connection1 = anchor.workspace.CentralizedConnection as Program<CentralizedConnection>;
  const connection2 = anchor.workspace.CentralizedConnection as Program<CentralizedConnection>;

  let owner: anchor.web3.Keypair;
  let notOwner: anchor.web3.Keypair;
  let feeHandler: anchor.web3.Keypair;

  let temp, xcallStatePda, replyDataPda;
  let conn1StatePda, conn2StatePda;
  let conn1FeePda, conn2FeePda;

  let tx;
  let network_id = "solana";

  let protocol_fee = new anchor.BN(anchor.web3.LAMPORTS_PER_SOL / 100);
  let connection_fee_message = new anchor.BN(anchor.web3.LAMPORTS_PER_SOL / 100);
  let connection_fee_response = new anchor.BN(anchor.web3.LAMPORTS_PER_SOL / 100);

  const airdrop = async (publicKey: anchor.web3.PublicKey) => {
    const airdropSignature = await provider.connection.requestAirdrop(
      publicKey,
      anchor.web3.LAMPORTS_PER_SOL,// Adjust amount as necessary
    );
    await provider.connection.confirmTransaction(airdropSignature);
  }

  const getTxnLogs = async (tx) => {
    const confirmation = await provider.connection.confirmTransaction(tx, "confirmed");
    console.log("Transaction confirmation status:", confirmation.value.err);

    let txDetails = await provider.connection.getTransaction(tx, { commitment: "confirmed" })

    if (txDetails?.meta?.logMessages) {
      txDetails.meta.logMessages.forEach(log => {
        console.log("Log:", log);
      });
    }

  }

  const getBalance = async (acc: PublicKey) => {
    let balance = await provider.connection.getBalance(acc);
    console.log("Account Balance is: ", balance);
  }

  beforeEach(async () => {

    owner = anchor.web3.Keypair.generate();
    notOwner = anchor.web3.Keypair.generate();
    feeHandler = anchor.web3.Keypair.generate();

    await airdrop(owner.publicKey);
    await airdrop(notOwner.publicKey);
    await airdrop(feeHandler.publicKey);

    [xcallStatePda, temp] = await PublicKey.findProgramAddressSync([Buffer.from("xcall")], xcall.programId);
    [replyDataPda, temp] = await PublicKey.findProgramAddressSync([Buffer.from("xcall-reply-data")], xcall.programId);
    [conn1StatePda, temp] = await PublicKey.findProgramAddressSync([Buffer.from("connection_state")], connection1.programId);

    console.log("conn1feepda", conn1FeePda);


    tx = await xcall.methods.initialize(network_id)
      .accounts({
        xcallState: xcallStatePda,
        replyState: replyDataPda,
        owner: owner.publicKey,
        systemProgram: SystemProgram.programId
      }).signers([owner]).rpc();


    tx = await connection1.methods.initialize()
      .accounts({
        user: owner.publicKey,
        systemProgram: SystemProgram.programId,
        connectionAccount: conn1StatePda,
      }).signers([owner]).rpc()
  })

  const setProtocolFee = async () => {
    console.debug("Setting Protocol Fee")
    await xcall.methods.setFee(protocol_fee)
      .accounts({
        xcallState: xcallStatePda,
        owner: owner.publicKey,
        systemProgram: SystemProgram.programId
      }).signers([owner]).rpc();
  }

  const initializeConnection2 = async () => {

    [conn2StatePda, temp] = await PublicKey.findProgramAddressSync([Buffer.from("connection_state")], connection2.programId);
    console.log("conn2feepda", conn2FeePda);
    tx = await connection2.methods.initialize()
      .accounts({
        user: owner.publicKey,
        systemProgram: SystemProgram.programId,
        connectionAccount: conn2StatePda,
      }).signers([owner]).rpc().catch(e => console.error(e))

    let k = await PublicKey.findProgramAddressSync([Buffer.from("connection_fee"), Buffer.from("0x1.icon")], connection2.programId);
    conn2FeePda = k[0];

    await connection2.methods.setFee("0x1.icon", connection_fee_message, connection_fee_response).accounts({
      connectionAccount: conn2StatePda,
      feeAccount: conn2FeePda,
      admin: owner.publicKey,
      systemProgram: SystemProgram.programId
    }).signers([owner]).rpc().catch(e => console.error(e))


  }

  const setConnectionFee = async () => {
    console.debug("Set connection fee")
    let k = await PublicKey.findProgramAddressSync([Buffer.from("connection_fee"), Buffer.from("0x1.icon")], connection1.programId);
    conn1FeePda = k[0];

    console.log(conn1FeePda)
    await connection1.methods.setFee("0x1.icon", connection_fee_message, connection_fee_response).accounts({
      connectionAccount: conn1StatePda,
      feeAccount: conn1FeePda,
      admin: owner.publicKey,
      systemProgram: SystemProgram.programId
    }).signers([owner]).rpc().catch(e => console.error(e))


    let connState = await connection1.account.feesState.fetch(conn1FeePda);
    console.log(connState)
    assert(connState.messageFees, connection_fee_message.toString())
    assert(connState.messageFees, connection_fee_message.toString())
  }


  it("Is initialized!", async () => {
    await setProtocolFee()
    await setConnectionFee()
    // await initializeConnection2()

    let connState = await connection1.account.connectionState.fetch(conn1StatePda);
    assert(connState.admin.toString, owner.publicKey.toString());

    console.log("Send message to connection", connection1.programId.toString())

    let to = "0x1.icon/cx7235a0296f4f0323587c1840181afbee84bbc91a"


    const exampleEnvelope: XCallEnvelope = {
      msg_type: 0,
      message: new Uint8Array([1, 2, 3, 4]),
      sources: [connection1.programId.toString()],
      destinations: ['cx7235a0296f4f0323587c1840181afbee84bbc91a']
    };
    const encoded = encodeXCallEnvelope(exampleEnvelope);

    let msg = Buffer.from(encoded, "hex");

    // console.log("connection program id ", connection1.programId)

    tx = await xcall.methods.sendMessage(to, msg).accounts({
      xcallState: xcallStatePda,
      sender: notOwner.publicKey,
      replyData: replyDataPda,
      feeHandler: feeHandler.publicKey,
      systemProgram: SystemProgram.programId,
    }).remainingAccounts([{
      pubkey: conn1StatePda,
      isWritable: true,
      isSigner: false
    }, {
      pubkey: conn1FeePda,
      isSigner: false,
      isWritable: false
    },
    {
      pubkey: connection1.programId,
      isSigner: false,
      isWritable: false
    }])
      .signers([notOwner]).rpc().catch(e => console.log(e))

    getTxnLogs(tx)

    // console.log(connection_fee_message.toNumber())
  });
});
