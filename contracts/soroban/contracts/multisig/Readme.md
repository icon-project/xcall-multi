# Stellar Multisig Contract Frontend Workflow

This README explains how to build a frontend for interacting with a Stellar Multisig contract. The frontend is provided with:
- **Multisig Contract Address**: Used to call functions like `create_proposal`, `add_approval_signature`, and `execute_proposal`.
- **Multisig Account Address**: Used to fetch and interact with relevant proposals.

---

## **Features Overview**

### 1. **Create Proposal Section**
This section allows users to create proposals to upgrade a Stellar contract.

- **Inputs**:
  - **Contract Address**: Address of the contract to be upgraded.
  - **Contract Hash**: Hash of the new WASM code to deploy.

- **Button**: "Create Proposal".

- **Action Flow**:
  1. Build a transaction to upgrade the contract(Can be taken further reference from https://github.com/icon-project/ICON-Projects-Planning/issues/510).
  2. Call the `create_proposal` method on the Multisig contract, passing:
     - The built transaction in XDR format.
     - The Multisig Account Address for reference.

---

### 2. **Approve Proposal Section**
This section allows users to approve proposals that are in a pending state.

- **Display**:
  - A list of proposals fetched from the Multisig contract, showing:
    - **Proposal ID**
    - **Status** (Pending, Approved, Executed).

- **Button**: "Approve" (enabled for pending proposals).

- **Action Flow**:
  1. Fetch the proposal data (XDR) from the contract(Structure can be taken reference from contract).
  2. Build the transaction and sign it using the signer's private key.
  3. Submit the signature and `proposal_id` to the `add_approval_signature` method to approve the proposal.

---

### 3. **Execute Proposal Section**
This section allows users to execute approved proposals.

- **Display**:
  - A list of approved proposals ready for execution.

- **Button**: "Execute" (enabled for approved proposals).

- **Action Flow**:
  1. Fetch transaction data and signatures from the contract for execution.
  2. Build the transaction using the fetched data.
  3. Submit the transaction to the Stellar network for execution.

---