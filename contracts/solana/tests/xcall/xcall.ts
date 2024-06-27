import * as anchor from "@coral-xyz/anchor";

import { TxnHelpers } from "../utils/transaction";
import { Xcall } from "../../target/types/xcall";
import { expect } from "chai";

describe("Xcall", async () => {
  const provider = anchor.AnchorProvider.env();
  const program: anchor.Program<Xcall> = anchor.workspace.Xcall;

  let connection = provider.connection;
  let wallet = provider.wallet as anchor.Wallet;

  const txnHelpers = new TxnHelpers(connection, wallet.payer);
});