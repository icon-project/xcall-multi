import os from "os";

import * as anchor from "@coral-xyz/anchor";
import { Connection } from "@solana/web3.js";

import { loadKeypairFromFile } from "./utils";

/** PROGRAMS TYPE */
import { Xcall } from "../target/types/xcall";
import { CentralizedConnection } from "../target/types/centralized_connection";
import { MockDappMulti } from "../target/types/mock_dapp_multi";

/** PROGRAMS IDL */
import dappIdl from "../target/idl/mock_dapp_multi.json";
import connectionIdl from "../target/idl/centralized_connection.json";
import xcallIdl from "../target/idl/xcall.json";

/** RPC PROVIDER */
export const RPC_URL = "http://127.0.0.1:8899";
export const connection = new Connection(RPC_URL, "confirmed");

/** WALLET KEYPAIR */
let keypairFilePath = os.homedir + "/.config/solana/id.json";
export const keypair = loadKeypairFromFile(keypairFilePath);
export const wallet = new anchor.Wallet(keypair);

/** PROVIDER FOR CLIENT */
export const provider = new anchor.AnchorProvider(connection, wallet);
anchor.setProvider(provider);

export const mockDappProgram: anchor.Program<MockDappMulti> =
  new anchor.Program(
    dappIdl as anchor.Idl,
    provider
  ) as unknown as anchor.Program<MockDappMulti>;
export const connectionProgram: anchor.Program<CentralizedConnection> =
  new anchor.Program(
    connectionIdl as anchor.Idl,
    provider
  ) as unknown as anchor.Program<CentralizedConnection>;
export const xcallProgram: anchor.Program<Xcall> = new anchor.Program(
  xcallIdl as anchor.Idl,
  provider
) as unknown as anchor.Program<Xcall>;
