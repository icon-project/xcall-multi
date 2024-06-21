import {
  Connection,
  Keypair,
  AddressLookupTableProgram,
  TransactionMessage,
  TransactionInstruction,
  PublicKey,
  VersionedTransaction,
} from "@solana/web3.js";

import { sleep } from ".";

export class TxnHelpers {
  connection: Connection;
  payer: Keypair;
  lookupTable: PublicKey;

  constructor(connection: Connection, payer: Keypair) {
    this.connection = connection;
    this.payer = payer;
  }

  async createAddressLookupTable() {
    let recentSlot = await this.connection.getSlot("max");

    let [createLookupTableIx, lookupTable] =
      AddressLookupTableProgram.createLookupTable({
        authority: this.payer.publicKey,
        payer: this.payer.publicKey,
        recentSlot,
      });

    const tx = await this.buildV0Txn([createLookupTableIx], [this.payer]);

    await this.connection.sendTransaction(tx);
    return (this.lookupTable = lookupTable);
  }

  async extendAddressLookupTable(addresses: PublicKey[]) {
    await sleep(2);

    let extendLookupTableIx = AddressLookupTableProgram.extendLookupTable({
      addresses,
      authority: this.payer.publicKey,
      lookupTable: this.lookupTable,
      payer: this.payer.publicKey,
    });

    const tx = await this.buildV0Txn([extendLookupTableIx], [this.payer]);
    await this.connection.sendTransaction(tx);
  }

  async getAddressLookupTable() {
    return await this.connection
      .getAddressLookupTable(this.lookupTable)
      .then((res) => res.value);
  }

  async printAddressLookupTable() {
    await sleep(2);

    const lookupTableAccount = await this.getAddressLookupTable();
    console.log(`Lookup Table: ${this.lookupTable}`);

    for (let i = 0; i < lookupTableAccount.state.addresses.length; i++) {
      const address = lookupTableAccount.state.addresses[i];
      console.log(
        `Index: ${i.toString().padEnd(2)} Address: ${address.toBase58()}`
      );
    }
  }

  async buildV0Txn(instructions: TransactionInstruction[], signers: Keypair[]) {
    let blockHash = await this.connection
      .getLatestBlockhash()
      .then((res) => res.blockhash);

    const messageV0 = new TransactionMessage({
      payerKey: this.payer.publicKey,
      recentBlockhash: blockHash,
      instructions,
    }).compileToV0Message();

    const tx = new VersionedTransaction(messageV0);
    signers.forEach((s) => tx.sign([s]));
    return tx;
  }

  async buildTxnWithLookupTable(
    instructions: TransactionInstruction[],
    signers: Keypair[]
  ) {
    await sleep(2);

    const lookupTableAccount = await this.connection
      .getAddressLookupTable(this.lookupTable)
      .then((res) => res.value);

    let blockhash = await this.connection
      .getLatestBlockhash()
      .then((res) => res.blockhash);

    let messageV0 = new TransactionMessage({
      payerKey: this.payer.publicKey,
      recentBlockhash: blockhash,
      instructions,
    }).compileToV0Message([lookupTableAccount]);

    const tx = new VersionedTransaction(messageV0);
    signers.forEach((s) => tx.sign([s]));
    return tx;
  }
}
