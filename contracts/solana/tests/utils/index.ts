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

export const uint128ToArray = (num: any) => {
  if (typeof num === "string" || typeof num === "number") {
    num = BigInt(num);
  } else if (!(num instanceof BigInt)) {
    throw new Error("Input must be a BigInt or convertible to a BigInt.");
  }

  let buffer = new ArrayBuffer(16);
  let view = new DataView(buffer);

  view.setBigUint64(0, num >> BigInt(64), false);
  view.setBigUint64(8, num & BigInt("0xFFFFFFFFFFFFFFFF"), false);

  return new Uint8Array(buffer);
};

export * from "./transaction";
