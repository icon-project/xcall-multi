import fs from "fs";
import { createHash } from "crypto";
import { Keypair, Connection, PublicKey } from "@solana/web3.js";

export const loadKeypariFromFile = (path: string) => {
  return Keypair.fromSecretKey(
    Buffer.from(JSON.parse(fs.readFileSync(path, "utf-8")))
  );
};

export const sleep = (seconds: number) => {
  return new Promise((resolve) => setTimeout(resolve, seconds * 1000));
};

export const hash = (message: Uint8Array) => {
  return createHash("sha256").update(message).digest("hex");
};

export * from "./transaction";
