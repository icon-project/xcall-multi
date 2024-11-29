import { PublicKey } from "@solana/web3.js";

import { uint128ToArray } from "../utils";
import { xcallProgram } from "..";

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
      [Buffer.from("proxy"), uint128ToArray(requestId)],
      xcallProgram.programId
    );

    return { pda, bump };
  }

  static successRes(sequenceNo: number) {
    const [pda, bump] = PublicKey.findProgramAddressSync(
      [Buffer.from("success"), uint128ToArray(sequenceNo)],
      xcallProgram.programId
    );

    return { pda, bump };
  }

  static defaultConnection(netId: String) {
    const [pda, bump] = PublicKey.findProgramAddressSync(
      [Buffer.from("conn"), Buffer.from(netId)],
      xcallProgram.programId
    );

    return { pda, bump };
  }

  static pendingRequest(messageBytes: Buffer) {
    const [pda, bump] = PublicKey.findProgramAddressSync(
      [Buffer.from("req"), messageBytes],
      xcallProgram.programId
    );

    return { pda, bump };
  }

  static pendingResponse(messageBytes: Buffer) {
    const [pda, bump] = PublicKey.findProgramAddressSync(
      [Buffer.from("res"), messageBytes],
      xcallProgram.programId
    );

    return { pda, bump };
  }

  static rollback(sequenceNo: number) {
    const [pda, bump] = PublicKey.findProgramAddressSync(
      [Buffer.from("rollback"), uint128ToArray(sequenceNo)],
      xcallProgram.programId
    );

    return { pda, bump };
  }
}
