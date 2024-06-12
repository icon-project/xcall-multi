import * as anchor from "@coral-xyz/anchor";
import { assert, expect } from "chai";
import { Keypair } from "@solana/web3.js";

import { TestContext } from "./setup";

describe("CentralizedConnection", () => {
  let ctx = new TestContext();

  it("[initialize]: should initialize the program", async () => {
    await ctx.initialize();

    let data = await ctx.getConfigAccount();

    assert.equal(data.admin.toString(), ctx.signer.publicKey.toString());
    assert.equal(data.xcall.toString(), ctx.signer.publicKey.toString());
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
    await ctx.program.methods
      .setAdmin(ctx.admin.publicKey)
      .accounts({})
      .signers([ctx.signer.payer])
      .rpc();

    let { admin } = await ctx.getConfigAccount();
    assert.equal(ctx.admin.publicKey.toString(), admin.toString());
  });

  it("[set_admin]: should fail if not called by admin", async () => {
    try {
      await ctx.program.methods
        .setAdmin(ctx.admin.publicKey)
        .accounts({})
        .signers([ctx.signer.payer])
        .rpc();
    } catch (err) {
      expect(err.message).to.includes("Only admin");
    }
  });

  it("[set_fee]: should set the fee for network ID", async () => {
    let msg_fee = 50;
    let res_fee = 100;

    await ctx.program.methods
      .setFee(ctx.networkId, new anchor.BN(msg_fee), new anchor.BN(res_fee))
      .accounts({})
      .signers([ctx.admin.payer])
      .rpc();

    let fee = await ctx.getFeeAccount();
    assert.equal(fee.messageFee.toNumber(), msg_fee);
    assert.equal(fee.responseFee.toNumber(), res_fee);
  });

  it("[claim_fees]: should claim fee stored in PDA account", async () => {
    let claimFees = ctx.getClaimFeesAddress();

    let transfer_amount = 500_000;
    ctx.transferLamports(claimFees, transfer_amount);
    await ctx.sleep(3000);

    const min_rent_exempt_balance =
      await ctx.connection.getMinimumBalanceForRentExemption(8);
    const before_pda_balance = (await ctx.connection.getAccountInfo(claimFees))
      .lamports;
    assert.equal(min_rent_exempt_balance + transfer_amount, before_pda_balance);

    await ctx.program.methods
      .claimFees()
      .accounts({ claimFees })
      .signers([ctx.signer.payer])
      .rpc();

    const after_pda_balance = (await ctx.connection.getAccountInfo(claimFees))
      .lamports;
    assert.equal(min_rent_exempt_balance, after_pda_balance);
  });

  it("[claim_fees]: should fail if not called by admin", async () => {
    let new_admin = Keypair.generate();
    await ctx.setAdmin(new_admin.publicKey);

    let claimFees = ctx.getClaimFeesAddress();

    try {
      await ctx.program.methods
        .claimFees()
        .accounts({
          claimFees,
        })
        .signers([ctx.signer.payer])
        .rpc();
    } catch (err) {
      expect(err.message).includes("Only admin");
    }
  });
});
