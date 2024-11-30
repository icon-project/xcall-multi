# Multisig Contract on Sui

This README provides an overview of the Multisig Contract on Sui, which serves as a signature manager rather than a traditional multisig contract. Unlike multisig implementations on EVM or CosmWasm, Sui has a built-in multisig mechanism accessible via the CLI. However, since using the CLI may not be practical for everyone, this contract simplifies multisig management, ensuring a secure and efficient way to handle multisig transactions.

## Features

- **Signature Management**: Add members to create a multisig address. The address remains consistent across contracts.
- **Tamper-Proof**: The contract is designed solely as a manager, ensuring no vulnerabilities even if tampered with.

## Contract Overview

The contract comprises several core functions essential for creating and managing multisig transactions:

- **`create_multi_signature`**: Initializes a multisig instance with the required signatures and signers.
- **`create_proposal`**: Creates a proposal with the serialized transaction bytes.
- **`approve_proposal`**: Allows members to approve a transaction by submitting their signatures.
- **`get_execute_command`**: Retrieves the command to execute a multisig transaction once the threshold is met.

### Function Signatures

- `create_multi_signature(raw_signatures:&vector<vector<u8>>, signers:&vector<Signer>, threshold:u16)`
- `create_proposal(storage:&mut Storage,title:String,tx_bytes_64:String,multisig_address:address,ctx:&TxContext)`
- `approve_proposal(storage:&mut Storage, proposal_id:u64, raw_signature_64:String, ctx:&TxContext)`
- `get_execute_command(storage:&Storage, proposal_id:u64): String`

## Step-by-Step Guide

### 1. Publishing the Contract

First, deploy the multisig contract to the Sui network. This contract will act as a manager for your multisig addresses and will not hold any funds or perform any transactions on its own.

### 2. Adding Members and Creating a Multisig Address

To create a multisig address:

- **Register Wallet**: You'll need the public key, not the address, to register a wallet in Sui. Refer to the [Sui Cryptography Documentation](https://docs.sui.io/concepts/cryptography/transaction-auth/keys-addresses) for more details on keys and addresses.
- **Frontend Interaction**: Since the Sui wallet does not directly expose public keys, when you connect your wallet to the multisig frontend, it will fetch the public key for you.
- **Setup**: After obtaining public keys, register each wallet and set the threshold for the number of required signatures.

### 3. Creating and Proposing Transactions

To propose a transaction:

- **Serialize Transaction**: Generate serialized bytes of the transaction you wish to execute from the multisig.
- **Handle Large Transactions**: If the transaction bytes are too large, save them to a file and create a proposal with the digest of the intent. This digest can be obtained by signing the transaction bytes using the Sui CLI. When signing and executing later, you must provide the original transaction bytes, not the digest.
    ```bash
    digest=$(sui keytool sign --data $result --address $active_address --json | jq '.digest')
    echo $digest
    ```
- **Create Proposal**: Use the `create_proposal` function to initiate the multisig process by creating a proposal with the serialized transaction bytes.

  **Important Note:** Ensure to provide a gas object that is held by the multisig address when creating the proposal. This is crucial because the transaction will need to be executed by the multisig, and therefore it requires its own gas object to complete the transaction.


### 4. Approving a Transaction

To approve a transaction:

- **Sign with Member**: Each member of the multisig must sign the serialized transaction bytes.
- **Submit Signature**: Members submit their signatures by calling the `approve_proposal` function with their signature and the proposal id.

### 5. Executing the Transaction

Once the required number of signatures (as per the threshold) is collected:

- **Retrieve Execute Command**: Call the `get_execute_command` function to get the command required to execute the signed transaction.
    ```bash
    sui client execute-signed-tx --tx-bytes ${ORIGINAL_TX_BYTES} --signatures <serialized-signatures>
    ```
- **Execute via CLI**: Replace ${ORIGINAL_TX_BYTES} with the original transaction bytes. Run the command retrieved from the previous step on the Sui CLI to execute the multisig transaction.

And that's it! The transaction will be signed by multiple members and executed successfully.

## Frontend Integration Guide for Multisig Contract on Sui

This guide is for the frontend development team to integrate the multisig contract functionalities into the user interface. The backend handles wallet registration and proposal creation, while the frontend focuses on the signing, approving, and executing stages.

### Workflow Overview

1. **Wallet Registration and Proposal Creation**: Handled by the backend team.
2. **Signing and Approving a Proposal**: Handled by the frontend.
3. **Retrieving the Execute Command**: Handled by the frontend and provided to the user for execution via the CLI.

### 1. Signing and Approving a Proposal

#### Retrieving and Displaying Proposals

To enable users to view the current proposals:

- **Retrieve Proposals**: The frontend should call the `get_proposals` function to fetch the list of proposals stored in the contract.

- The function returns a reference to a table containing the proposals, which can be displayed in the frontend for users to view and interact with.

```javascript
// Fetch proposals from the contract
const proposals = await getProposalsFunction(storage_id: storage_id);
```

#### Step 1: User Provides Transaction Bytes

- The frontend should allow users to input or upload the serialized transaction bytes that need to be signed.

#### Step 2: Signing the Transaction Bytes

- Once the user provides the transaction bytes, the frontend should facilitate the signing process using the connected wallet. 

- You can use the following code snippet to sign the transaction bytes:

```javascript
// Assume 'kp' is the keypair obtained from the connected wallet
const message = transactionBytes; // This is the serialized transaction bytes provided by the user
const signature = (await kp.signPersonalMessage(message)).signature;
```

#### Step 3: Calling the Approve Function

- After signing the transaction bytes, the frontend should call the contract's `approve_proposal` function with the obtained signature.

- The function call would look something like this:

```javascript
// Approve proposal using the obtained signature
await approveProposalFunction({
  storage_id: storage_id,
  proposalId: proposalId,
  rawSignature64: signature,
});
```

- Here, `proposalId` is the ID of the proposal, `rawSignature64` is the signature obtained from the signing process.

### 2. Retrieving and Presenting the Execute Command

#### Step 1: Calling the Get Execute Command Function

- Once the necessary number of approvals (as per the threshold) is met, the frontend should call the contract's `get_execute_command` function to retrieve the execute command.

- The function call could be structured as follows:

```javascript
// Retrieve the execute command
const executeCommand = await getExecuteCommandFunction({
  storage_id: storage_id,
  proposalId: proposalId
});
```
- Here, `proposalId` is the ID of the proposal.

#### Step 2: Presenting the Execute Command to the User

- The frontend should then display the execute command to the user (likely the proposal creator or designated executor) so that they can copy, fill the transaction bytes blank space and run it in their CLI to execute the multisig transaction.

- You can provide a simple UI element like a copy button next to the command to facilitate easy copying.
