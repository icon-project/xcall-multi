# Changes in the Implementation of xCall in Stellar

## Overview

This document outlines the major changes in the xCall implementation on the Stellar blockchain and the reasoning behind these updates. Key modifications include a new fee deduction mechanism where xCall and each connection directly deduct fees from the user’s account, and an updated storage management approach utilizing instance and persistent storage while extending rent periodically to comply with Stellar's requirements.

## Major Changes

**1. Fee Deduction Mechanism**

**Change**

- The `send_call` function of xCall now directly deducts fees from the user's account.
- Similarly, `send_message` function of each connection also deducts fees directly from the user.

**Rationale**

- On Stellar, native token transfers (XLM) aren’t managed directly by contracts. Instead, transfers are handled through specific operations within transactions.
- Because of this, we can’t have users send fees to a contract. Instead, xCall directly deducts the fees from the user’s account, and each connection also takes its fees directly from the user. This method fits Stellar’s way of handling transactions.

**2. Storage Management and Rent Extension**

**Change**

- Configuration data is stored in instance storage for efficient access and management throughout the contract's lifecycle.
- Long-term and persistent data are stored in persistent storage to ensure durability and retention.
- Storage rent is extended periodically based on the frequency and volume of data usage to maintain compliance.

**Rationale**

- Stellar’s storage model has distinct types such as temporary, persistent, and instance storage, each with different cost and management implications. We use these types based on their suitability and extend the rent periodically as needed to ensure compliance with Stellar's requirements and maintain data accessibility.

## Conclusion

The updated implementation of xCall on Stellar deviates from the original architecture but maintains functional equivalence, ensuring that the protocol remains robust and efficient within Stellar's unique constraints.
