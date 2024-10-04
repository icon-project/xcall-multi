# Sui DApp Integration Guide for XCall Interface

This guide outlines the key considerations and steps necessary for integrating your decentralized application (DApp) on Sui with the `xcall` cross-chain transfer interface. Sui's stateless architecture requires specific adjustments compared to other blockchains, especially when handling cross-chain function calls like `execute_call` and `execute_rollback`.

## Key Differences on Sui: Stateless Execution

Unlike other blockchains, Sui is a stateless chain. This means that when calling functions such as `execute_call` or `execute_rollback`, all required object IDs and parameters must be provided upfront. This is a significant deviation from stateful blockchains, where some state information might be implicitly available.

### `ExecuteParams` Struct Requirements

The `ExecuteParams` struct must be constructed with precision as it will be integrated by the centralized relay. The struct should include:

```typescript
public struct ExecuteParams has drop {
    type_args: vector<String>, 
    args: vector<String>,
}
// type_args: A vector of strings representing the type arguments required by the execute_call or execute_rollback function.
// args: A vector of strings representing the execution or rollback arguments. These typically include object IDs, request IDs, and other necessary data.
```

### Required Getter Functions

To handle the stateless nature of Sui, you must use the following getter functions:

#### `get_execute_params` Function

This function retrieves the parameters required to execute a cross-chain call.

```typescript
entry fun get_execute_params(config: &DappState, _msg: vector<u8>): ExecuteParams
// config: Reference to the DApp's state.
// _msg: The crosschain data for the call.
// Returns: An ExecuteParams struct containing type arguments and execution arguments.

```

#### `get_rollback_params` Function

This function retrieves the parameters needed to execute a rollback in case of a failed or cancelled transaction.

```typescript
entry fun get_rollback_params(config: &DappState, _msg: vector<u8>): ExecuteParams
// config: Reference to the DApp's state.
// _msg: The crosschain data for the call.
// Returns: An ExecuteParams struct containing type arguments and rollback arguments.

```

These functions are designed to return all necessary parameters required by `execute_call` and `execute_rollback` in the same order as they are used in the `execute_call` and `execute_rollback` functions. These parameters are encapsulated in the `ExecuteParams` struct, which ensures that the centralized relay can correctly interpret and utilize the provided data.

#### Returning Object IDs

When returning object IDs within the `ExecuteParams`, ensure they are in string format with a `0x` prefix. This format is crucial for correct interpretation by the centralized relay. The `0x` prefix indicates that the string is a hexadecimal representation of the object ID, which the relay uses to correctly process the cross-chain transactions.

#### Handling Arbitrary Strings

In some cases, your contract may need to return arbitrary strings that cannot be derived directly from the contract (e.g., certain system-generated data like a `clock`). If such a situation arises, you should:

- **Relay Configuration**: Configure your centralized relay to recognize and use these arbitrary strings. For example, if your contract returns a string such as "clock" for a specific type, the relay must have a corresponding configuration to handle this "clock" string appropriately. This allows the relay to map the string to the correct object or data needed for the cross-chain transaction.

### Compulsory and Config-Free Parameters

- **`request_id` and `data` in `execute_call`**: These are compulsory parameters that doesn’t require additional configuration in the relay, uniquely identifies the cross-chain request and data, must be included in the `args` of `ExecuteParams`.

- **`fee` in `execute_call`**: The fee represents sui token, which will be handled by relayer itself, so fee in `execute_call` is mandatory and it is returned as `coin` must be included in the `args` of `ExecuteParams`. You can look on examples for more understanding.

- **`sn` in `execute_rollback`**: Similarly, the sequence number (`sn`) in `execute_rollback` is mandatory and must be included in the `args` of `ExecuteParams`. This parameter identifies the specific transaction rollback request.

### `execute_call` Function

This function handles the execution of cross-chain messages with basic arguments needed, more than this if you need any arguments, you will also return them from `get_execute_params` function in the same order as in `execute_call` function.

```typescript
entry public fun execute_call(
    state: &mut DappState, // state: Mutable reference to the DApp's state.
    xcall: &mut XCallState, // xcall: Mutable reference to the XCall state.
    mut fee: Coin<SUI>, // fee: Coin object representing the transaction fee in SUI.
    request_id: u128, // request_id: Unique identifier for the cross-chain request.
    data: vector<u8>, // data: The crosschain data for the call.
    ctx: &mut TxContext
)
```

### `execute_rollback` Function

This function handles the rollback of a cross-chain transaction, with basic arguments needed, more than this if you need any arguments, you will also return them from `get_rollback_params` function in the same order as in `execute_rollback` function.

```move
entry public fun execute_rollback(
    state: &mut DappState, // state: Mutable reference to the DApp's state.
    xcall: &mut XCallState, // xcall: Mutable reference to the XCall state.
    sn: u128, // sn: Sequence number (identifier) for the rollback request.
    ctx: &mut TxContext
)
```
This function, in combination with the `get_rollback_params` function, helps in preparing the necessary parameters and executing the rollback seamlessly.

### Key Points to Remember:
  
- **ExecuteParams Structure**: The `ExecuteParams` struct must be returned by the getter functions (`get_execute_params` and `get_rollback_params`) to provide all necessary information for the relay to process the cross-chain transactions.

- **Returning Object IDs**: Always return object IDs in string format with a `0x` prefix to ensure they are correctly interpreted by the relay.

- **Handling Arbitrary Strings**: If your contract returns arbitrary strings (e.g., `clock`), ensure that these strings are properly configured in the relay so they can be mapped to the correct data or objects.

- **Compulsory Parameters**: Ensure that required parameters like `request_id`, `data`, and `coin`  for `execute_call` and `sn` for `execute_rollback` are included in the `args` of `ExecuteParams` and do not require additional configuration in the relay.

### Integration Summary

- Use `get_execute_params` to gather all necessary parameters for executing cross-chain calls.
- Use `get_rollback_params` to prepare for potential rollbacks.
- Implement the `execute_call` function to handle the execution of cross-chain messages.
- Implement the `execute_rollback` function to manage transaction rollbacks when needed.

These steps will ensure that your DApp can effectively interact with the `xcall` and our centralized_relay interface on Sui, enabling smooth cross-chain transactions.

## Example DApp Contracts

For more detailed examples and practical implementations, refer to the [Balanced Move Contracts on Sui](https://github.com/balancednetwork/balanced-move-contracts). These contracts demonstrate how to structure your DApp and integrate with the `xcall` interface effectively.

## Note
This specification is essential for integration with our centralized relay binary. If your use case involves utilizing our `xcall` interface but with a custom relayer, you have the flexibility to modify these parameters within your DApp and are not required to adhere strictly to this specification.










