import * as anchor from "@coral-xyz/anchor";
import { assert, expect } from "chai";
import { Keypair } from "@solana/web3.js";

import { TestContext, connectionPDA } from "./setup";
import { TxnHelpers, sleep } from "../utils";
import { SYSTEM_PROGRAM_ID } from "@coral-xyz/anchor/dist/cjs/native/system";
import { Xcall } from "../../target/types/xcall";

const xcallProgram: anchor.Program<Xcall> = anchor.workspace.Xcall;

describe("CentralizedConnection", () => {
  let ctx = new TestContext();
  let txnHelpers = new TxnHelpers(ctx.connection, ctx.signer.payer);

  it("[initialize]: should initialize the program", async () => {
    await ctx.initialize();
    await sleep(3);

    let data = await ctx.getConfig();

    assert.equal(data.admin.toString(), ctx.signer.publicKey.toString());
    assert.equal(data.xcall.toString(), xcallProgram.programId.toString());
    assert.equal(data.sn.toString(), new anchor.BN(0).toString());
  });

  it("[initialize]: should fail on double initialize", async () => {
    try {
      await ctx.initialize();
    } catch (err) {
      expect(err.message).to.includes(
        "Error processing Instruction 0: custom program error: 0x0"
      );
    }
  });

  it("[set_admin]: should set the new admin", async () => {
    let newAdmin = Keypair.generate();
    await ctx.setAdmin(newAdmin);

    await sleep(3);

    let { admin } = await ctx.getConfig();
    assert.equal(ctx.admin.publicKey.toString(), admin.toString());
  });

  it("[set_admin]: should fail if not called by admin", async () => {
    let non_admin = Keypair.generate();

    try {
      await ctx.program.methods
        .setAdmin(Keypair.generate().publicKey)
        .accountsStrict({
          admin: non_admin.publicKey,
          config: connectionPDA.config().pda,
        })
        .signers([non_admin])
        .rpc();
    } catch (err) {
      expect(err.message).to.includes("Only admin");
    }
  });

  it("[set_fee]: should set the fee for network ID", async () => {
    let msg_fee = 50;
    let res_fee = 100;

    await txnHelpers.airdrop(ctx.admin.publicKey, 1e9);
    await sleep(3);

    await ctx.program.methods
      .setFee(ctx.networkId, new anchor.BN(msg_fee), new anchor.BN(res_fee))
      .accountsStrict({
        config: connectionPDA.config().pda,
        fee: connectionPDA.fee(ctx.networkId).pda,
        admin: ctx.admin.publicKey,
        systemProgram: SYSTEM_PROGRAM_ID,
      })
      .signers([ctx.admin])
      .rpc();

    await sleep(3);

    let fee = await ctx.getFee(ctx.networkId);
    assert.equal(fee.messageFee.toNumber(), msg_fee);
    assert.equal(fee.responseFee.toNumber(), res_fee);
  });

  it("[claim_fees]: should claim fee stored in PDA account", async () => {
    let claimFees = connectionPDA.claimFees().pda;

    let transfer_amount = 500_000;
    await txnHelpers.airdrop(claimFees, transfer_amount);
    await sleep(3);

    const min_rent_exempt_balance =
      await ctx.connection.getMinimumBalanceForRentExemption(9);
    const before_pda_balance = (await ctx.connection.getAccountInfo(claimFees))
      .lamports;
    assert.equal(min_rent_exempt_balance + transfer_amount, before_pda_balance);

    await ctx.program.methods
      .claimFees()
      .accountsStrict({
        admin: ctx.admin.publicKey,
        config: connectionPDA.config().pda,
        claimFees,
      })
      .signers([ctx.admin])
      .rpc();

    const after_pda_balance = (await ctx.connection.getAccountInfo(claimFees))
      .lamports;
    assert.equal(min_rent_exempt_balance, after_pda_balance);
  });

  it("[claim_fees]: should fail if not called by admin", async () => {
    let new_admin = Keypair.generate();

    try {
      await ctx.program.methods
        .claimFees()
        .accountsStrict({
          admin: new_admin.publicKey,
          config: connectionPDA.config().pda,
          claimFees: connectionPDA.claimFees().pda,
        })
        .signers([new_admin])
        .rpc();
    } catch (err) {
      expect(err.message).includes("OnlyAdmin");
    }
  });
});
