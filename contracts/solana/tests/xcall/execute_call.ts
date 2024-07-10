import * as anchor from "@coral-xyz/anchor";
import { describe } from "mocha";
import { TxnHelpers, hash, sleep } from "../utils";
import { Xcall } from "../../target/types/xcall";
import { MockDapp } from "../../target/types/mock_dapp";
import { TestContext, XcallPDA } from "./setup";
import { CSMessage, CSMessageRequest, CSMessageType, MessageType } from "./types";
import { SYSTEM_PROGRAM_ID } from "@coral-xyz/anchor/dist/cjs/native/system";
import { expect } from "chai";

describe("xcall- execute message", () => {
    const provider = anchor.AnchorProvider.env()
    const connection = provider.connection;
    const wallet = provider.wallet as anchor.Wallet;

    const txnHelpers = new TxnHelpers(connection, wallet.payer);
    const ctx = new TestContext(connection, txnHelpers, wallet.payer);

    const xcallProgram: anchor.Program<Xcall> = anchor.workspace.Xcall;
    const mockDapp: anchor.Program<MockDapp> = anchor.workspace.MockDapp;

    before(async () => {
        let defaultConnectionPDA = XcallPDA.defaultConnection("icx").pda;
        await ctx.setDefaultConnection("icx", defaultConnectionPDA);

    })


    it("[execute call] - sending invalid dapp address ", async () => {
        /* create proxy request from handle message
        execute call with that proxy request
        */
        let netId = "icx"
        let payload = new CSMessageRequest(
            "icx/abc",
            "icon",
            2,
            MessageType.CallMessage,
            new Uint8Array([0, 1, 2, 3]),
            [wallet.publicKey.toString()]
        )
        let cs_message = new CSMessage(
            CSMessageType.CSMessageRequest,
            payload.encode()
        ).encode()
        let message_seed = Buffer.from(hash(cs_message), "hex");
        let sequenceNo = new anchor.BN(2);

        let handleMessageIx = await xcallProgram.methods
            .handleMessage(netId, Buffer.from(cs_message), sequenceNo)
            .accounts({
                signer: wallet.payer.publicKey,
                systemProgram: SYSTEM_PROGRAM_ID,
                config: XcallPDA.config().pda,
                pendingRequest: XcallPDA.pendingRequest(message_seed).pda,
                defaultConnection: XcallPDA.defaultConnection("icx").pda,
                rollbackAccount: null,
                pendingResponse: null,
                successfulResponse: null,
                proxyRequest: XcallPDA.proxyRequest(1).pda,
            })
            // .instruction();

        // let handleMessageTx = await txnHelpers.buildV0Txn([handleMessageIx], [wallet.payer])
        // let handle  = await connection.sendTransaction(handleMessageTx);
        // await txnHelpers.logParsedTx(handle);

        await sleep(3);
        
        // proxy request -> account initialized at req_id: 1
        let req_id = new anchor.BN(1);
        let from_nid = "icx"
        let data = payload.data;
        let executeCallIX = await xcallProgram.methods
            .executeCall(req_id, Buffer.from(data), from_nid)
            .accountsStrict({
                signer: wallet.payer.publicKey,
                systemProgram: SYSTEM_PROGRAM_ID,  
                proxyRequests: XcallPDA.proxyRequest(1).pda,
                replyState: null,
                defaultConnection: XcallPDA.defaultConnection("icx").pda,
            })
            // .instruction();


        // let executeCallTx = await txnHelpers.buildV0Txn([executeCallIX], [wallet.payer])
        // try {

            // await connection.sendTransaction(executeCallTx);
        // } catch (err) {
            // expect(err.message).to.include("Invalid pubkey")
        // }

    })

    it("[execute call] - sending wrong account of proxy request ", async () => {
        let netId = "icx"
        let payload = new CSMessageRequest(
            "icx/abc",
            "icon",
            2,
            MessageType.CallMessage,
            new Uint8Array([0, 1, 2, 3]),
            [wallet.publicKey.toString()]
        )
        let cs_message = new CSMessage(
            CSMessageType.CSMessageRequest,
            payload.encode()
        ).encode()
        let message_seed = Buffer.from(hash(cs_message), "hex");
        let sequenceNo = new anchor.BN(2);

        let handleMessageIx = await xcallProgram.methods
            .handleMessage(netId, Buffer.from(cs_message), sequenceNo)
            .accounts({
                signer: wallet.payer.publicKey,
                systemProgram: SYSTEM_PROGRAM_ID,
                config: XcallPDA.config().pda,
                pendingRequest: XcallPDA.pendingRequest(message_seed).pda,
                defaultConnection: XcallPDA.defaultConnection("icx").pda,
                rollbackAccount: null,
                pendingResponse: null,
                successfulResponse: null,
                proxyRequest: XcallPDA.proxyRequest(2).pda,
            })
            // .instruction();
        
            // let handleMessageTx = await txnHelpers.buildV0Txn([handleMessageIx], [wallet.payer])
            // let handle  = await connection.sendTransaction(handleMessageTx);
            // await txnHelpers.logParsedTx(handle);
    
            await sleep(3);

        // proxy request -> account initialized at req_id: 2 but will be sending for 1
        let req_id = new anchor.BN(3);
        let from_nid = "icx"
        let data = payload.data;
        let executeCallIX = await xcallProgram.methods
            .executeCall(req_id, Buffer.from(data), from_nid)
            .accountsStrict({
                signer: wallet.payer.publicKey,
                systemProgram: SYSTEM_PROGRAM_ID,  
                proxyRequests: XcallPDA.proxyRequest(2).pda,
                replyState: null,
                defaultConnection: XcallPDA.defaultConnection("icx").pda,
            })
            // .instruction();


        // let executeCallTx = await txnHelpers.buildV0Txn([executeCallIX], [wallet.payer])
        // await connection.sendTransaction(executeCallTx);
        // try {

        //     await connection.sendTransaction(executeCallTx);
        // } catch (err) {
        //     expect(err.message).to.include("Invalid pubkey")
        // }
    } )

   
})
