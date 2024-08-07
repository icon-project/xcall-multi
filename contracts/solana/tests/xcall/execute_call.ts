import * as anchor from "@coral-xyz/anchor";
import { describe } from "mocha";
import { SYSTEM_PROGRAM_ID } from "@coral-xyz/anchor/dist/cjs/native/system";
import { expect } from "chai";

import { TxnHelpers, hash, sleep } from "../utils";
import { Xcall } from "../../target/types/xcall";
import { MockDappMulti } from "../../target/types/mock_dapp_multi";
import { TestContext, XcallPDA } from "./setup";
import {
  CSMessage,
  CSMessageRequest,
  CSMessageType,
  MessageType,
} from "./types";

describe("xcall- execute message", () => {
  const provider = anchor.AnchorProvider.env();
  const connection = provider.connection;
  const wallet = provider.wallet as anchor.Wallet;

  const txnHelpers = new TxnHelpers(connection, wallet.payer);
  const ctx = new TestContext(connection, txnHelpers, wallet.payer);

  const xcallProgram: anchor.Program<Xcall> = anchor.workspace.Xcall;

  it("[execute call] - should execute call", async () => {});
});
