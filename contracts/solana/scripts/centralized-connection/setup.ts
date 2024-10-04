import { PublicKey } from "@solana/web3.js";

import { uint128ToArray } from "../utils";
import { connectionProgram } from "..";

export class ConnectionPDA {
  constructor() {}

  static config() {
    let [pda, bump] = PublicKey.findProgramAddressSync(
      [Buffer.from("config")],
      connectionProgram.programId
    );

    return { bump, pda };
  }

  static network_fee(networkId: string) {
    const [pda, bump] = PublicKey.findProgramAddressSync(
      [Buffer.from("fee"), Buffer.from(networkId)],
      connectionProgram.programId
    );

    return { pda, bump };
  }

  static receipt(networkId: string, sn: number) {
    const [pda, bump] = PublicKey.findProgramAddressSync(
      [Buffer.from("receipt"), Buffer.from(networkId), uint128ToArray(sn)],
      connectionProgram.programId
    );

    return { pda, bump };
  }

  static authority() {
    let [pda, bump] = PublicKey.findProgramAddressSync(
      [Buffer.from("connection_authority")],
      connectionProgram.programId
    );

    return { bump, pda };
  }
}
