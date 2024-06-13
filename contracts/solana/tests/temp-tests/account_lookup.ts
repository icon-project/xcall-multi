import {
    Connection,
    PublicKey,
    Keypair,
    AddressLookupTableProgram,
    TransactionMessage,
    VersionedTransaction,
    TransactionInstruction
  } from "@solana/web3.js";
  
  
  const secret = [110, 188, 83, 176, 53, 152, 30, 254, 45, 198, 203, 236, 145, 52, 62, 108, 79, 202, 41, 191, 118, 8, 116, 219, 105, 31, 65, 163, 104, 177, 77, 86, 20, 111, 102, 232, 160, 83, 119, 224, 107, 179, 85, 73, 63, 176, 95, 14, 57, 49, 26, 154, 219, 178, 174, 152, 168, 162, 206, 37, 84, 161, 108, 40]; //  Replace with your secret
  const signer = Keypair.fromSecretKey(new Uint8Array(secret));
  const connection = new Connection('https://api.devnet.solana.com', 'confirmed')
  
  export async function createV0Transaction(inst: TransactionInstruction[]) {
  
    const slot = await connection.getSlot('max');
    const block = await connection.getBlock(slot, {
      maxSupportedTransactionVersion: 0,
    });
    // lets create a versioned tranaction
    const messageV0 = new TransactionMessage({
      payerKey: signer.publicKey,
      recentBlockhash: block.blockhash,
      instructions: inst,
    }).compileToV0Message()
  
    const transaction = new VersionedTransaction(messageV0);
  
    transaction.sign([signer]);
  
    // posting the transaction on-chain
    const sx = await connection.sendTransaction(transaction);
  
    console.log('Transaction size is: ', transaction.serialize().length)
    console.log('The transaction is posted', '\n', `https://explorer.solana.com/tx/${sx}?cluster=devnet`);
  
  
  }
  
  export async function creatLUT() {
  
    
    const slot = await connection.getSlot('max');
  
  
    // creates the LUT address here
    const [lookupTableInst, lookupTableAddress] =
      AddressLookupTableProgram.createLookupTable({
        authority: signer.publicKey,
        payer: signer.publicKey,
        recentSlot: slot,
      });
  
    console.log("lookup table address:", lookupTableAddress.toBase58());
  
    createV0Transaction([lookupTableInst])
  
  
  }
  
  
  export async function extendLookupTable(LOOKUP_TABLE_ADDRESS: PublicKey) {
  
    const addAddressInstruction = AddressLookupTableProgram.extendLookupTable({
      payer :signer.publicKey,
      authority: signer.publicKey,
      lookupTable: LOOKUP_TABLE_ADDRESS,
      addresses : [
          Keypair.generate().publicKey,
          Keypair.generate().publicKey,
          Keypair.generate().publicKey,
          Keypair.generate().publicKey,
      ]});
  
     createV0Transaction([addAddressInstruction])
    
  }
  
  export async function createV0TransactionWithLUT(inst: [any], lookupTablePubKey: PublicKey) {
  
    const slot = await connection.getSlot('max');
    const block = await connection.getBlock(slot, {
      maxSupportedTransactionVersion: 0,
    });
  
    const lookupTableAccount = await connection.getAddressLookupTable(lookupTablePubKey);
    // lets create a versioned tranaction
    const messageV0 = new TransactionMessage({
      payerKey: signer.publicKey,
      recentBlockhash: block.blockhash,
      instructions: inst,
    }).compileToV0Message([lookupTableAccount.value])
  
    const transaction = new VersionedTransaction(messageV0);
  
    transaction.sign([signer]);
  
    // posting the transaction on-chain
    const sx = await connection.sendTransaction(transaction);
  
    // console.log('Transaction size with address lookup table: ', transaction.serialize().length)
    console.log('The transaction is posted', '\n', `https://explorer.solana.com/tx/${sx}?cluster=devnet`);
  
  
  }
  
  /// TESTING PART WITH ANCHOR 
  // UNCOMMENT THIS PART
  // describe("tests", () => {
  // it ("versioned transaction sent to program ", async() => {
  
  //   const LOOKUP_TABLE_ADDRESS = new PublicKey("GtRKcsrwcdtXuGgZpwipUZzAfJ5ryZMYP96i6uTSu1VH");
  
  
  //   const ix =  await program.methods // program should be setup
  //   .anInstruction()
  //   .accounts({
  //       someAccount: new PublicKey('8DDqsxJuLiU6dJGfdKaPb8J9hnsuF1rTMASiVYim6UeF'),
  //       signer: signer.publicKey,
    
  //   })
  //   .instruction();
  
  //   createV0TransactionWithLUT([ix],LOOKUP_TABLE_ADDRESS)
  // })
  // })
  