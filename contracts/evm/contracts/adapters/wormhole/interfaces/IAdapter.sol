// SPDX-License-Identifier: Apache-2.0
pragma solidity >=0.8.0;

/**
 * @title IAdapter - Interface for Wormhole-xCall Adapter
 * @dev This interface defines the functions and events for a Wormhole-xCall adapter,
 * allowing communication and message transfer between xCall on different blockchain networks.
 */
interface IAdapter {
    /**
     * @notice Emitted when a response is put on hold.
     * @param _sn The serial number of the response.
     */
    event ResponseOnHold(uint256 indexed _sn);

    /**
     * @notice Configure connection settings for a destination chain.
     * @param networkId The network ID of the destination chain.
     * @param chainId The chain ID of the destination chain.
     * @param endpoint The endpoint or address of the destination chain.
     * @param gasLimit The gas limit for transactions on the destination chain.
     */
    function configureConnection(
        string calldata networkId,
        uint16 chainId,
        bytes32 endpoint,
        uint256 gasLimit
    ) external;

    /**
     * @notice Pay and trigger the execution of a stored response to be sent back.
     * @param _sn The serial number of the message for which the response is being triggered.
     */
    function triggerResponse(uint256 _sn) external payable;
}
