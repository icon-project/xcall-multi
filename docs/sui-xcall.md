# Changes in the Implementation of xCall in Sui

## Overview

This document outlines the major changes in the implementation of xCall in the Sui blockchain and the rationale behind these changes. Key updates include modifications to the execution process of calls and rollbacks, the introduction of forced rollback mechanisms, and changes in connection registration and package upgrades.

## Major Changes

### 1. Execution of Calls and Rollbacks

#### Change
- The `execute_call` and `execute_rollback` functions will now be executed from the dApps rather than xCall.

#### Rationale
- Sui is a stateless blockchain, meaning it does not maintain the state of every dApp.
- Executing calls from xCall would require accessing the data of each dApp, which is inefficient.
- By executing calls from the dApps, each dApp have their own data and use a common xCall, making the process more efficient and reducing the data management overhead for xCall.

### 2. Handling Rollback Failures

#### Change
- Introduced `execute_forced_rollback` in dApps (e.g., Balanced), which can be executed by an admin in case of a failure in `execute_call`.

#### Rationale
- Sui lacks try-catch mechanisms in xCall, making it impossible to rollback every message that fails in `execute_call`. Instead, it will fail the entire transaction if there is a configuration failure.

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

## Upgrading Packages

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
- **Usage**: Cap IDs are used for configuring Balanced in other chains, while the package ID is used for function calls from the Sui side.

## Conclusion

The updated implementation of the xcall deviates from the original xCall architecture but retains functional equivalence. 

