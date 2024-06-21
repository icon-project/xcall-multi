import * as anchor from "@coral-xyz/anchor";

import { PublicKey } from "@solana/web3.js";
import { Xcall } from "../../target/types/xcall";

const xcallProgram: anchor.Program<Xcall> = anchor.workspace.Xcall;
export class TestContext {
  nid: String;

  constructor() {}
}
export class XcallPDA {
  constructor() {}

  static config() {
    let [pda, bump] = PublicKey.findProgramAddressSync(
      [Buffer.from("config")],
      xcallProgram.programId
    );

    return { bump, pda };
  }

  static proxyRequest(requestId: number) {
    const [pda, bump] = PublicKey.findProgramAddressSync(
      [Buffer.from("proxy"), Buffer.from(requestId.toString())],
      xcallProgram.programId
    );

    return { pda, bump };
  }

  successRes(sequenceNumber: number) {
    const [pda, bump] = PublicKey.findProgramAddressSync(
      [Buffer.from("succes"), Buffer.from(sequenceNumber.toString())],
      xcallProgram.programId
    );

    return { pda, bump };
  }

  defaultConnection(netId: String) {
    const [pda, bump] = PublicKey.findProgramAddressSync(
      [Buffer.from("conn"), Buffer.from(netId)],
      xcallProgram.programId
    );

    return { pda, bump };
  }

  pendingRequest(messageBytes: Buffer) {
    const [pda, bump] = PublicKey.findProgramAddressSync(
      [messageBytes],
      xcallProgram.programId
    );

    return { pda, bump };
  }

  pendingResponse(messageBytes: Buffer) {
    const [pda, bump] = PublicKey.findProgramAddressSync(
      [messageBytes],
      xcallProgram.programId
    );

    return { pda, bump };
  }

  rollback(sequenceNumber: number) {
    const [pda, bump] = PublicKey.findProgramAddressSync(
      [Buffer.from("rollback"), Buffer.from(sequenceNumber.toString())],
      xcallProgram.programId
    );

    return { pda, bump };
  }
}
