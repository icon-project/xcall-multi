// SPDX-License-Identifier: Apache-2.0
pragma solidity >=0.8.2;
pragma abicoder v2;

import "openzeppelin-contracts/contracts/token/ERC20/utils/SafeERC20.sol";
import "openzeppelin-contracts/contracts/utils/Strings.sol";
import "@iconfoundation/xcall-solidity-library/utils/ParseAddress.sol";

import "./Types.sol";
import "./Encoding.sol";
import "./GeneralizedConnection.sol";

import {console} from "forge-std/console.sol";

/// @title ICONIntents
/// @notice Implements the intent-based swapping protocol for cross-chain swaps.
contract Intents is GeneralizedConnection {
    using Encoding for *;
    using Strings for string;
    using SafeERC20 for IERC20;
    using ParseAddress for address;
    using ParseAddress for string;

    uint256 public depositId; // Deposit ID counter
    string public nid; // Network Identifier
    uint16 public protocolFee; //  ProtocolFee in basis points taken on outgoing transfersr
    address public feeHandler;  // Receiver of protocol fees
    address public constant NATIVE_ADDRESS = address(0);

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
        string emitter,
        string srcNID,
        string dstNID,
        string creator,
        string destinationAddress,
        string token,
        uint256 amount,
        string toToken,
        uint256 minReceive,
        bytes data
    );

    event OrderFilled(uint256 indexed  id, string indexed srcNID, bytes32 indexed orderHash, string solverAddres, uint256 fillAmount, uint256 fee, uint256 solverPayout, uint256 remaningAmount);
    event OrderCancelled(uint256 indexed id, string indexed srcNID, bytes32 indexed orderHash, uint256 userReturn);
    event OrderClosed(uint256 indexed id);

    constructor(string memory _nid, uint16 _protocolFee, address _feeHandler, address _relayer) {
        nid = _nid;
        relayAdress = _relayer;
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
        string memory toToken,
        string memory toAddress,
        uint256 minReceive,
        bytes memory data
    ) public payable {
        // Escrows amount from user
        if (token == NATIVE_ADDRESS) {
            require(msg.value == amount, "Deposit amount not equal to order amount");
        } else {
            IERC20(token).safeTransferFrom(msg.sender, address(this), amount);
        }

        // Create unique deposit ID
        uint256 id = depositId++;

        Types.SwapOrder memory order = Types.SwapOrder({
            id: id,
            emitter: address(this).toString(),
            srcNID: nid,
            dstNID: to,
            creator: msg.sender.toString(),
            destinationAddress: toAddress,
            token: token.toString(),
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
    /// @param order The SwapOrder object.
    /// @param amount The amount to fill.
    /// @param solverAddress The address of the solver filling the order.
    function fill(
        Types.SwapOrder memory order,
        uint256 amount,
        string memory solverAddress
    ) external payable {
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

        // Calculate the payout
        uint256 payout = (order.amount * amount) / order.minReceive;
        // Ensure the amount to fill is valid
        require(
            payout <= remaningAmount,
            "Cannot fill more than remaining ask"
        );

        remaningAmount -= payout;

        // Update order state
        bool closeOrder = false;
        if (remaningAmount == 0) {
            // Finalize the order if fully filled
            delete pendingFills[orderHash];
            finishedOrders[orderHash] = true;
            closeOrder = true;
        } else {
            pendingFills[orderHash] = remaningAmount;
        }

        // Transfer tokens
        uint256 fee = (amount * protocolFee) / 10_000;
        amount -= fee;
        _transferResult(order.destinationAddress, order.toToken, amount, fee);

        // Create and send the order message
        Types.OrderFill memory orderFill = Types.OrderFill({
            id: order.id,
            orderBytes: orderBytes,
            solver: solverAddress,
            amount: payout,
            closeOrder: closeOrder
        });

        if (order.srcNID.equal(order.dstNID)) {
            _resolveFill(nid, orderFill);
            return;
        }

        Types.OrderMessage memory orderMessage = Types.OrderMessage({
            messageType: Types.FILL,
            message: orderFill.encode()
        });

        _sendMessage(order.srcNID, orderMessage.encode());
        emit OrderFilled(order.id, order.srcNID, orderHash, solverAddress, amount, fee, payout, remaningAmount);
    }

    function _transferResult(string memory _toAddress, string memory _toToken, uint256 amount, uint256 fee) internal {
        address toAddress = _toAddress.parseAddress("IllegalArgument");
        address toTokenAddress = _toToken.parseAddress("IllegalArgument");
        if (toTokenAddress == NATIVE_ADDRESS) {
            require(msg.value == amount+fee, "Deposit amount not equal to order amount");
            _nativeTransfer(toAddress, amount);
            _nativeTransfer(feeHandler, fee);
        } else {
            IERC20(toTokenAddress).safeTransferFrom(
                msg.sender,
                toAddress,
                amount
            );
            IERC20(toTokenAddress).safeTransferFrom(
                msg.sender,
                feeHandler,
                fee
            );
        }
    }

    /// @notice Cancels a cross-chain order.
    /// @param id The order ID to cancel.
    function cancel(uint256 id) external {
        Types.SwapOrder storage order = orders[id];
        require(
            order.creator.parseAddress("IllegalArgument") == msg.sender,
            "Cannot cancel this order"
        );

        if (order.srcNID.equal(order.dstNID)) {
            _resolveCancel(nid, order.encode());
            return;
        }

        Types.OrderMessage memory _msg = Types.OrderMessage({
            messageType: Types.CANCEL,
            message: Types.Cancel({orderBytes:order.encode()}).encode()
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
        // _recvMessage(srcNetwork, _connSn);

        Types.OrderMessage memory orderMessage = _msg.decodeOrderMessage();
        if (orderMessage.messageType == Types.FILL) {
            Types.OrderFill memory _fill = orderMessage
                .message
                .decodeOrderFill();
            _resolveFill(srcNetwork, _fill);
        } else if (orderMessage.messageType == Types.CANCEL) {
            Types.Cancel memory _cancel = orderMessage.message.decodeCancel();
            _resolveCancel(srcNetwork, _cancel.orderBytes);
        }
    }

    function getOrder(uint256 id) external view returns (Types.SwapOrder memory) {
        return orders[id];
    }

    function _resolveFill(
        string memory srcNetwork,
        Types.OrderFill memory _fill
    ) internal {
        Types.SwapOrder memory order = orders[_fill.id];
        require(
            keccak256(order.encode()) == keccak256(_fill.orderBytes),
            "Mismatched order"
        );

        require(
            order.dstNID.equal(srcNetwork),
            "Mismatched order"
        );

        if (_fill.closeOrder) {
            delete orders[_fill.id];
            emit OrderClosed(_fill.id);
        }

        address tokenAddress = order.token.parseAddress("IllegalArgument");
        if (tokenAddress == NATIVE_ADDRESS) {
            _nativeTransfer(_fill.solver.parseAddress("IllegalArgument"), _fill.amount);
        } else {
            IERC20(tokenAddress).safeTransfer(
                _fill.solver.parseAddress("IllegalArgument"),
                _fill.amount
            );
        }
    }

    function _resolveCancel(string memory srcNetwork, bytes memory orderBytes) internal {
        bytes32 orderHash = keccak256(orderBytes);
        if (finishedOrders[orderHash]) {
            return;
        }

        Types.SwapOrder memory order = orderBytes.decodeSwapOrder();

        require(
            order.srcNID.equal(srcNetwork),
            "Mismatched order"
        );

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
            amount: remaningAmount,
            closeOrder: true
        });

        Types.OrderMessage memory _msg = Types.OrderMessage({
            messageType: Types.FILL,
            message: _fill.encode()
        });

        _sendMessage(order.srcNID, _msg.encode());
        emit OrderCancelled(order.id, order.srcNID, orderHash, remaningAmount);
    }

    function _nativeTransfer(address to, uint256 amount) internal {
        bool sent = payable(to).send(amount);
        require(sent, "Failed to send tokens");
    }

}
