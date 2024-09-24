import { PublicKey } from "@solana/web3.js";

import { mockDappProgram } from "..";

export class DappPDA {
  constructor() {}

  static config() {
    let [pda, bump] = PublicKey.findProgramAddressSync(
      [Buffer.from("config")],
      mockDappProgram.programId
    );

    return { bump, pda };
  }

  static connections(networkId: string) {
    const buffer1 = Buffer.from("connections");
    const buffer2 = Buffer.from(networkId);
    const seed = [buffer1, buffer2];

    const [pda, bump] = PublicKey.findProgramAddressSync(
      seed,
      mockDappProgram.programId
    );

    return { pda, bump };
  }

  static authority() {
    let [pda, bump] = PublicKey.findProgramAddressSync(
      [Buffer.from("dapp_authority")],
      mockDappProgram.programId
    );

    return { bump, pda };
  }
}
