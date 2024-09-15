// SPDX-License-Identifier: Apache-2.0
pragma solidity >=0.8.2;
pragma abicoder v2;

import "openzeppelin-contracts/contracts/token/ERC20/utils/SafeERC20.sol";
import "openzeppelin-contracts/contracts/utils/Strings.sol";
import "./Types.sol";
import "./Encoding.sol";
import "./GeneralizedConnection.sol";

/// @title ICONIntents
/// @notice Implements the intent-based swapping protocol for cross-chain swaps.
contract Intents is GeneralizedConnection {
    using Encoding for *;
    using Strings for string;

    uint256 public depositId; // Deposit ID counter
    string public nid; // Network Identifier
    uint16 public protocolFee; //  ProtocolFee in basis points taken on outgoing transfersr
    address public feeHandler;  // Receiver of protocol fees

    mapping(uint256 => Types.SwapOrder) public orders; // Mapping of deposit ID to SwapOrder
    mapping(bytes32 => uint256) public pendingFills; // Mapping of order hash to pending amount to fill
    mapping(bytes32 => bool) public finishedOrders; // Mapping of order hash to bool, for all finished orders

    /// @dev Emitted when a new swap intent is created.
    /// @param id The ID of the swap order.
    /// @param emitter Address of emitter contract
    /// @param srcNID The source network ID.
    /// @param dstNID The destination network ID.
    /// @param creator The address of the creator of the swap order.
    /// @param destinationAddress The address where the swapped tokens will be sent.
    /// @param token The address of the token being swapped.
    /// @param amount The amount of tokens being swapped.
    /// @param toToken The token to be received after the swap (if applicable).
    /// @param minReceive The minimum amount of tokens to receive after the swap.
    /// @param data Additional arbitrary data for the swap.
    event SwapIntent(
        uint256 indexed id,
        bytes emitter,
        string srcNID,
        string dstNID,
        bytes creator,
        bytes destinationAddress,
        bytes token,
        uint256 amount,
        bytes toToken,
        uint256 minReceive,
        bytes data
    );

    constructor(string memory _nid, uint16 _protocolFee, address _feeHandler) {
        nid = _nid;
        protocolFee = _protocolFee;
        feeHandler = _feeHandler;
    }

    /// @notice Initiates a swap by escrowing the tokens and emitting a SwapIntent event.
    /// @param to The destination network identifier.
    /// @param token The address of the token to swap.
    /// @param amount The amount of the token to swap.
    /// @param toToken The token to receive on the destination network.
    /// @param toAddress The receiving address on the destination network.
    /// @param minReceive The minimum amount of toToken to receive.
    /// @param data Additional data for future parameters.
    function swap(
        string memory to,
        address token,
        uint256 amount,
        bytes memory toToken,
        bytes memory toAddress,
        uint256 minReceive,
        bytes memory data
    ) public {
        // Escrows amount from user
        IERC20(token).transferFrom(msg.sender, address(this), amount);

        // Create unique deposit ID
        uint256 id = depositId++;

        Types.SwapOrder memory order = Types.SwapOrder({
            id: id,
            emitter: abi.encode(address(this)),
            srcNID: nid,
            dstNID: to,
            creator: abi.encode(msg.sender),
            destinationAddress: toAddress,
            token: abi.encode(token),
            amount: amount,
            toToken: toToken,
            minReceive: minReceive,
            data: data
        });

        orders[id] = order;
        emit SwapIntent(
            order.id,
            order.emitter,
            order.srcNID,
            order.dstNID,
            order.creator,
            order.destinationAddress,
            order.token,
            order.amount,
            order.toToken,
            order.minReceive,
            order.data
        );
    }

    /// @notice Fills an order for a cross-chain swap.
    /// @param id The order ID.
    /// @param order The SwapOrder object.
    /// @param amount The amount to fill.
    /// @param solverAddress The address of the solver filling the order.
    function fill(
        uint256 id,
        Types.SwapOrder memory order,
        uint256 amount,
        bytes memory solverAddress
    ) external {
        // Compute the hash of the order
        bytes memory orderBytes = order.encode();
        bytes32 orderHash = keccak256(orderBytes);

        // Check if the order has been finished
        require(!finishedOrders[orderHash], "Order has already been filled");

        // Load the pending amount if available
        uint256 remaningAmount = pendingFills[orderHash];
        if (remaningAmount == 0) {
            remaningAmount = order.amount;
        }

        // Ensure the amount to fill is valid
        require(
            amount <= remaningAmount,
            "Cannot fill more than remaining ask"
        );

        // Calculate the payout
        uint256 payout = (order.amount * amount) / order.minReceive;
        remaningAmount -= payout;

        // Update order state
        if (order.minReceive == 0) {
            // Finalize the order if fully filled
            delete pendingFills[orderHash];
            finishedOrders[orderHash] = true;
        } else {
            pendingFills[orderHash] = remaningAmount;
        }

        // Transfer tokens
        uint256 fee = (payout * protocolFee) / 10_000;
        payout -= fee;
        address toAddress = _bytesToAddress(order.destinationAddress);
        address toTokenAddress = _bytesToAddress(order.toToken);
        IERC20(toTokenAddress).transferFrom(
            msg.sender,
            toAddress,
            payout
        );
        IERC20(toTokenAddress).transferFrom(
            msg.sender,
            feeHandler,
            fee
        );

        // Create and send the order message
        Types.OrderFill memory orderFill = Types.OrderFill({
            id: id,
            orderBytes: orderBytes,
            solver: solverAddress,
            amount: amount
        });

        if (order.srcNID.equal(order.dstNID)) {
            _resolveFill(orderFill);
            return;
        }

        Types.OrderMessage memory orderMessage = Types.OrderMessage({
            messageType: Types.FILL,
            message: orderFill.encode()
        });

        _sendMessage(order.srcNID, orderMessage.encode());
    }

    /// @notice Cancels a cross-chain order.
    /// @param id The order ID to cancel.
    function cancel(uint256 id) external {
        Types.SwapOrder storage order = orders[id];
        require(
            _bytesToAddress(order.creator) == msg.sender,
            "Cannot cancel this order"
        );

        if (order.srcNID.equal(order.dstNID)) {
            _resolveCancel(order.encode());
            return;
        }

        Types.OrderMessage memory _msg = Types.OrderMessage({
            messageType: Types.CANCEL,
            message: order.encode()
        });
        _sendMessage(order.dstNID, _msg.encode());
    }

    /// @notice Handles incoming messages from the relayer.
    /// @param srcNetwork The source network identifier.
    /// @param _connSn The connection serial number.
    /// @param _msg The message payload.
    function recvMessage(
        string memory srcNetwork,
        uint256 _connSn,
        bytes calldata _msg
    ) external {
        // Handle incoming messages from the relayer
        _recvMessage(srcNetwork, _connSn);

        Types.OrderMessage memory orderMessage = _msg.decodeOrderMessage();
        if (orderMessage.messageType == Types.FILL) {
            Types.OrderFill memory _fill = orderMessage
                .message
                .decodeOrderFill();
            _resolveFill(_fill);
        } else if (orderMessage.messageType == Types.CANCEL) {
            Types.Cancel memory _cancel = orderMessage.message.decodeCancel();
            _resolveCancel(_cancel.orderBytes);
        }
    }

    function _resolveFill(
        Types.OrderFill memory _fill
    ) internal {
        Types.SwapOrder memory order = orders[_fill.id];
        require(
            string(order.encode()).equal(string(_fill.orderBytes)),
            "Mismatched order"
        );
        require(
            order.amount >= _fill.amount,
            "Fill amount exceeds order amount"
        );

        order.amount -= _fill.amount;
        if (order.amount == 0) {
            delete orders[_fill.id];
        }

        IERC20(_bytesToAddress(order.token)).transfer(
            _bytesToAddress(_fill.solver),
            _fill.amount
        );
    }

    function _resolveCancel(bytes memory orderBytes) internal {
        bytes32 orderHash = keccak256(orderBytes);
        if (finishedOrders[orderHash]) {
            return;
        }

        Types.SwapOrder memory order = orderBytes.decodeSwapOrder();

        // Load the pending amount if available
        uint256 remaningAmount = pendingFills[orderHash];
        if (remaningAmount == 0) {
            remaningAmount = order.amount;
        } else {
            delete pendingFills[orderHash];
        }

        finishedOrders[orderHash] = true;

        Types.OrderFill memory _fill = Types.OrderFill({
            id: order.id,
            orderBytes: orderBytes,
            solver: order.creator,
            amount: remaningAmount
        });

        Types.OrderMessage memory _msg = Types.OrderMessage({
            messageType: Types.FILL,
            message: _fill.encode()
        });

        _sendMessage(order.srcNID, _msg.encode());
    }

    function _bytesToAddress(bytes memory b) private pure returns (address) {
        return address(uint160(bytes20(b)));
    }
}
