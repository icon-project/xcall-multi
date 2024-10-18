# Changes in the Implementation of xCall in Solana

## Overview

This document outlines the major changes in the implementation of xCall on the Solana blockchain and the rationale behind these updates. Key modifications include the removal of reply logic and default connections, improvements in handling cross-contract errors, the introduction of a forced rollback mechanism, and the splitting of message handling into multiple instructions to address stack memory limitations.

## Major Changes

**1. Remove reply state**

**Change**

- The `send_call` and `execute_call` instructions have been simplified by removing the reply logic
- When xCall invokes a dApp's instruction, the dApp is no longer able invoke any further instructions back to xCall within the same transaction

**Rationale**

- Solana's security model restricts cross-program reentrancy, preventing dApps from re-invoking xCall during the same execution flow. This ensures a more secure and predictable execution environment.

**2. Remove default connection**

**Change**

- The default connection state has been removed, requiring dApps to explicitly specify the connections they wish to use for cross-chain messaging.

**Rationale**

- Solana programs are inherently stateless, meaning they do not maintain persistent account states.
- The initial purpose of a default connection was to simplify the process for users, allowing them to avoid managing connections for each chain individually. However, since users must always specify the connection account they intend to use, the default connection became redundant and unnecessary.

**3. Handling cross-contract error**

**Change**

- Introduced the `HandleCallMessageResponse` type for dApps (e.g., Balanced), which will be returned by dApps in both success and failure cases.

**Rationale**

- Solana lacks native exception handling mechanisms like try-catch, making it difficult to roll back every failed message during `execute_call`.
- Instead of throwing an error and causing the entire transaction to fail, the dApp will now return a standardized response type (`HandleCallMessageResponse`). This type will allow xCall to appropriately handle the outcome of the message, whether it succeeds or fails.

**4. Splitting the HandleMessage Instruction**

**Change**

- The `handle_message` instruction has been split into two separate instructions: `handle_request` and `handle_result`. These instructions are invoked internally by xCall after `handle_message` is executed.

**Rationale**

- Solana imposes a strict limit on stack frame size, capped at 4KB. When using the Anchor framework, combining the accounts for both `handle_request` and `handle_result` exceeded this limit, leading to potential execution failures.
- To address this, we split the logic of receiving messages into multiple instructions. The `handle_message` instruction, which is executed for every incoming message, now includes only the common state accounts. This reduces stack memory consumption as only necessary accounts are deserialized in this step.
- After executing `handle_message`, xCall invokes either `handle_request` or `handle_result` based on the message type, passing the remaining accounts specific to each context. This ensures that the stack memory usage stays within limits, resolving the original issue.
- We considered alternative solutions like using `Box` and `Zero_Copy` provided by the Anchor framework. While these approaches worked in the `solana-test-validator` environment, they were not viable on devnet, likely due to the lack of feature activation on that network. Therefore, we opted for the current solution, splitting the instructions in a way that guarantees they are invoked only by xCall.

**5. Handle Force Rollback Implementation**

**Change**

- Introduced the `handle_forced_rollback` instruction to manage forced rollbacks of cross-chain messages when an unknown error occurs after the message is received on the destination chain.

**Rationale**

- The `handle_forced_rollback` function allows dApps to initiate a rollback when a cross-chain message encounters an unknown error post-reception on the destination chain. Instead of letting the message fail silently or causing undefined behavior, this function provides a controlled way to send a failure response back to the source chain, effectively reversing the state to reflect that the message was not successfully processed.

## Conclusion

The updated implementation of xCall on Solana deviates from the original architecture but maintains functional equivalence, ensuring that the protocol remains robust and efficient within Solana's unique constraints.
