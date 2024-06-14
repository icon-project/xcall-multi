# Changes in the Implementation of xCall in Sui

## Overview

This document outlines the major changes in the implementation of xCall in the Sui blockchain and the rationale behind these changes. Key updates include modifications to the execution process of calls and rollbacks, the introduction of forced rollback mechanisms, and changes in connection registration and package upgrades.

## Major Changes

### 1. Execution of Calls and Rollbacks

#### Change
- The `execute_call` and `execute_rollback` functions will now be executed from the dApps rather than xCall.

#### Process

1. **Initiate Call from dApp**:
   - dApp initiates a call to xCall and receives a ticket containing execution data and protocol information.

2. **Retrieve Execution Data**:
   - dApp retrieves execution data from the ticket.

3. **Execute Call within dApp**:
   - dApp executes the call using its own state and data.

4. **Report Execution Result**:
   - If successful, dApp sends `true` to `execute_call_result` in xCall.
   - If failed, dApp sends `false` to `execute_call_result` in xCall.

5. **Execute Rollback if Necessary**:
   - If xCall receives `false`, it triggers the rollback process using the data and protocols specified in the initial ticket and closes the ticket.

#### Rationale
- Sui is a stateless blockchain, meaning it does not maintain the state of every dApp.
- Executing calls from xCall would require accessing the data of each dApp, which is inefficient.
- By executing calls from the dApps, each dApp have their own data and use a common xCall, making the process more efficient and reducing the data management overhead for xCall.

### 2. Handling Rollback Failures

#### Change
- Introduced `execute_forced_rollback` in dApps (e.g., Balanced), which can be executed by an admin in case of a failure in `execute_call`.

#### Rationale
- There is no concept of exception handling in sui such as try-catch, making it impossible to rollback every message that fails in `execute_call`. Instead, it will fail the entire transaction if there is a configuration failure.

### 3. RollbackMessage event Change

#### Additions
- `data` and `dapp` parameters in the `RollbackExecuted` message.

#### Rationale
- `dapp` parameter is needed to recognize the dApp module to call, since they all have same package id so need a indicator to seperate module.
- `data` parameter is required for getting type arguments (e.g., in our case needed to get token type for `asset_manager` module in balanced).

#### CallMessage Event

- Similar to the DApp address in the `CallMessage` event, it now includes the DApp cap ID. The relay tracks this to map to the Balanced module and call execute_call from the specific module.

### 4. Connection Registration

#### Changes
- xCall and centralized connection are now the same entity.
- For multiple connections, admin can register the connection from xCall with a connection ID in the format `centralized-...` and a relayer. The connection address equivalent to other chain is the connection-id here.

#### Connection Cap
- A connection cap is generated and transferred to the relayer.
- The relayer uses this cap to gain admin access to the centralized connection.
- Admin rights are tied to the possession of the cap.

#### Relayer Changes
- To change the relayer, the connection cap can be transferred externally.

## Sui Object ID, Cap ID, Address Format

### External and Internal of Contract ID Format

- **External Format**: IDs in Sui are formatted to start with `0x`.
- **Internal Format**: Internally, IDs are handled without the `0x` prefix.

### Implementation

- To maintain consistency and efficiency within the contract, all IDs are managed as hex strings.
- This dual-format approach ensures that the contract can seamlessly handle both formats(`0x` prefixed and non-prefixed), converting as necessary when sending or receiving IDs externally.

## Deployment Process

### Sui Network Branches

- **Sui maintains different branches for testnet and mainnet.**
- So, packages such as `xcall` and `balanced` need to align with the respective Sui network (testnet or mainnet).

### Contract Branch Management

1. **Main Contract Branch**:
   - The common xcall package will be managed under the branch: `feat/sui-xcall-contracts`.
   - The common balanced package will be managed under the branch: `main`.

2. **Testnet and Mainnet Branches**:
   - Separate branches will be maintained for testnet and mainnet deployments.
   - These branches will reflect the state of the contracts as per the latest deployments and upgrades.

### Deployment Steps

1. **Branch Creation**:
   - Create and maintain `testnet` and `mainnet` branches from `feat/sui-xcall-contracts`.

2. **Deployment and Upgrades**:
   - Deployment is done from specific(testnet/mainnet) branch
   - After each deployment or upgrade, update the `Move.toml` file with the deployed package id.
   - A new field `published-at` will be introduced to package manifests, to designate a package's on-chain address(latest address), while the addresses in the `[addresses]` section of the manifest will continue to represent package addresses for type resolution and access purposes.
   - Push the updated `Move.toml` file to the respective branch (`testnet` or `mainnet`).
   - This ensures the branch is the official release branch for the corresponding network.

### Dependency Management for Balanced

- **Testnet**:
  - While using `xcall` as a dependency in Balanced, the `testnet` branch of `xCall` should be used.
- **Mainnet**:
  - For mainnet deployments, the `mainnet` branch of `xCall` should be utilized.

### Summary

- Maintaining separate branches for testnet and mainnet ensures clear and organized deployment processes.
- Updating the `move.toml` file with deployment details keeps the branches accurate and reflective of the current state.
- Dependency management aligns with the respective network branches, ensuring seamless integration and functionality.

## Upgrading Packages

### Upgrade Compatibility
Upgraded packages must maintain compatibility with all their previous versions, so a package published to run with version V of a dependency can run against some later version, V + k. This property is guaranteed by doing a link and layout compatibility check during publishing, enforcing the following:

#### Compatible Changes
- Adding new types, functions and modules.
- Changing the implementations of existing functions.
- Changing the signatures of non-public functions.
- Removing ability constraints on type parameters.
- Adding abilities to struct definitions.

#### Incompatible Changes
- Changing the signatures of public functions.
- Changing the layout of existing types (adding, removing or modifying field names or types).
- Adding new ability constraints on type parameters.
-Removing abilities from structs.

### Immutability and Versioning

#### Sui Packages
- Sui packages are immutable, meaning upgrading a package retains the original package, allowing both old and new packages to be used.
- New features added in upgrade require the new package, while the old package can still operate using the same storage.

#### Versioning
- Introduced versioning in xcall and balanced to prevent the use of old packages after critical bug fixes or breaking changes.
-  Upgrades involve updating the storage version and the constant version in the package. Each entry function enforces the version to ensure the old package is not used.

### Event Handling

#### Event Immutability
- Events are immutable objects and won't upgrade to a newer package ID.
- Events added in an upgrade must be tracked from the new package ID, while previous events are tracked from the old package ID.

#### Event Listening
- To listen to events added during an upgrade, all package IDs must be tracked.
- Establishing websocket connections with each package ID is cumbersome, so adding events post-deployment is not recommended.

### Dependency Upgrades:
   - **Implementation**: If any dependencies are upgraded, the corresponding packages must also be upgraded to utilize the new features from these dependencies.
   - **Process**: For instance, if the `rlp` package is upgraded, `xcall` should be upgraded to point to the new `rlp`, and then `balanced` should be upgraded to point to the new xCall to incorporate new features. This cascading upgrade ensures all improvements are integrated but increases the complexity of the development process.

### Dapp Package Structure

#### Balanced Package
- **Modules**: Asset Manager, xCall Manager, Balanced Dollar.
- **Identifiers**: Each package has one package ID and three cap IDs (one for each module). Cap IDs are akin to contract addresses in other chains.
- **Usage**: While configuring balanced in other chains, we configure Cap ID for each module. Like sending message from icon would require sending to specific cap id.

## Conclusion

The updated implementation of the xcall deviates from the original xCall architecture but retains functional equivalence. 

