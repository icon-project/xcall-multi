// SPDX-License-Identifier: Apache-2.0
pragma solidity >=0.8.0;
pragma abicoder v2;

import "openzeppelin-contracts/contracts/token/ERC20/utils/SafeERC20.sol";
import "openzeppelin-contracts/contracts/utils/Strings.sol";
import "./Types.sol";
import "./Encoding.sol";
/// @title ICONIntents
/// @notice Implements the intent-based swapping protocol for cross-chain swaps.
contract Intents {
    using Encoding for *;
    using Strings for string;

    uint256 public depositId;                        // Deposit ID counter
    string public nid;                               // Network Identifier

    address public relayer;                         // Address of the relayer
    uint256 public connSn;                           // Connection serial number

    mapping(string => mapping(uint256 => bool)) public receipts;  // Mapping of receipts for tracking

    mapping(uint256 => Types.SwapOrder) public orders;      // Mapping of deposit ID to SwapOrder
    mapping(bytes32 => Types.SwapOrder) public pendingFills;  // Mapping of order hash to pending SwapOrder fills
    mapping(bytes32 => bool) public finishedOrders;     // Mapping of order hash to bool, for all finished orders

    modifier onlyRelayer() {
        require(msg.sender == this.relayer(), "OnlyRelayer");
        _;
    }

    /// @dev Emitted when a new swap intent is created.
    /// @param id The ID of the swap order.
    /// @param order The SwapOrder object.
    event SwapIntent(uint256 indexed id, Types.SwapOrder order);

    event Message(string targetNetwork, uint256 sn, bytes _msg);

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
        uint256 id = depositId++;

        Types.SwapOrder memory order = Types.SwapOrder({
            id: id,
            srcNID: nid,
            dstNID: to,
            creator: abi.encode(msg.sender),
            destinationAddress: toAddress,
            token: token,
            amount: amount,
            toToken: toToken,
            minReceive: minReceive,
            data: data
        });

        orders[id] = order;
        emit SwapIntent(id, order);
    }

    /// @notice Handles incoming messages from the relayer.
    /// @param srcNetwork The source network identifier.
    /// @param _connSn The connection serial number.
    /// @param _msg The message payload.
    function recvMessage(
        string memory srcNetwork,
        uint256 _connSn,
        bytes memory _msg
    ) external onlyRelayer {
        require(!receipts[srcNetwork][_connSn], "Duplicate Message");
        receipts[srcNetwork][_connSn] = true;

        Types.OrderMessage memory orderMessage = _msg.decodeOrderMessage();
        if (orderMessage.messageType == Types.FILL) {
            Types.OrderFill memory _fill = orderMessage.message.decodeOrderFill();
            _resolveFill(srcNetwork,_fill);
        } else if (orderMessage.messageType == Types.CANCEL) {
            Types.Cancel memory _cancel = orderMessage.message.decodeCancel();
            _resolveCancel(_cancel.orderBytes);
        }
    }

    function _resolveFill(
        string memory srcNetwork,
        Types.OrderFill memory _fill
    ) internal {
        Types.SwapOrder memory order = orders[_fill.id];
        require(string(order.encode()).equal(string(_fill.orderBytes)), "Mismatched order");
        require(order.dstNID.equal(srcNetwork), "Invalid source network");
        require(order.amount >= _fill.amount, "Fill amount exceeds order amount");

        order.amount -= _fill.amount;
        if (order.amount == 0) {
            delete orders[_fill.id];
        }

        IERC20(order.token).transfer(_bytesToAddress(_fill.solver), _fill.amount);
    }



    function _resolveCancel(
        bytes memory orderBytes
    ) internal {
        bytes32 orderHash = keccak256(orderBytes);
        if (finishedOrders[orderHash]) {
            return;
        }

        Types.SwapOrder storage pendingFill = pendingFills[orderHash];
        delete pendingFills[orderHash];
        finishedOrders[orderHash] = true;

        Types.OrderFill memory _fill = Types.OrderFill({
            id: pendingFill.id,
            orderBytes: orderBytes,
            solver: pendingFill.creator,
            amount: pendingFill.amount
        });

        Types.OrderMessage memory _msg = Types.OrderMessage({
            messageType: Types.FILL,
            message: _fill.encode()
        });

        _sendMessage(pendingFill.srcNID, _msg.encode());
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
        require(!finishedOrders[orderHash], "Hash has already been used");

        // Load the pending fill if available
        Types.SwapOrder storage pendingFill = pendingFills[orderHash];
        if (pendingFill.amount > 0) {
            order = pendingFill;
        }

        // Ensure the amount to fill is valid
        require(amount <= order.minReceive, "Cannot fill more than remaining ask");

        // Calculate the payout
        uint256 payout = (order.amount * amount) / order.minReceive;

        // Transfer tokens
        address toAddress = _bytesToAddress(order.destinationAddress);
        IERC20(_bytesToAddress(order.toToken)).transferFrom(msg.sender, toAddress, amount);

        // Update order state
        order.minReceive -= amount;
        order.amount -= payout;

        // Finalize the order if fully filled
        if (order.minReceive == 0) {
            delete pendingFills[orderHash];
            finishedOrders[orderHash] = true;
        }

        // Create and send the order message
        Types.OrderFill memory orderFill = Types.OrderFill({
            id: id,
            orderBytes: orderBytes,
            solver: solverAddress,
            amount: amount
        });

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
        require(_bytesToAddress(order.creator) == msg.sender, "Cannot cancel this order");

        Types.OrderMessage memory _msg = Types.OrderMessage({
            messageType: Types.CANCEL,
            message: abi.encode(order)
        });
        _sendMessage(order.srcNID, _msg.encode());
    }

    function _sendMessage(string memory targetNetwork, bytes memory _msg) internal {
        connSn++;
        emit Message(targetNetwork, connSn, _msg);
    }

    function _bytesToAddress(bytes memory b) private pure returns (address) {
        return address(uint160(bytes20(b)));
    }

//    function _bytesToAddress(bytes memory bys) private pure returns (address addr) {
//         assembly {
//             addr := mload(add(bys,20))
//         }
//     }
}
