import fs from "fs";
import { Keypair } from "@solana/web3.js";

export const loadKeypariFromFile = (path: string) => {
  return Keypair.fromSecretKey(
    Buffer.from(JSON.parse(fs.readFileSync(path, "utf-8")))
  );
};

export const sleep = (seconds: number) => {
  return new Promise((resolve) => setTimeout(resolve, seconds * 1000));
};

export * from "./transaction";
