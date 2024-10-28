// SPDX-License-Identifier: Apache-2.0
pragma solidity >=0.8.2;
pragma abicoder v2;

import "openzeppelin-contracts/contracts/token/ERC20/utils/SafeERC20.sol";
import "openzeppelin-contracts/contracts/utils/Strings.sol";
import "@iconfoundation/xcall-solidity-library/utils/ParseAddress.sol";
import "openzeppelin-contracts/contracts/access/Ownable.sol";


import "./Types.sol";
import "./Encoding.sol";
import "./GeneralizedConnection.sol";
import "./Permit2OrderLib.sol";
import "./IPermit2.sol";

import {console} from "forge-std/console.sol";

/// @title ICONIntents
/// @notice Implements the intent-based swapping protocol for cross-chain swaps.
contract Intents is GeneralizedConnection, Ownable {
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
    IPermit2 public immutable permit2;

    mapping(uint256 => Types.SwapOrder) public orders; // Mapping of deposit ID to SwapOrder
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
    /// @param toAmount The minimum amount of tokens to receive after the swap.
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
        uint256 toAmount,
        bytes data
    );

    event OrderFilled(uint256 indexed  id, string indexed srcNID);
    event OrderCancelled(uint256 indexed  id, string indexed srcNID);
    event OrderClosed(uint256 indexed id);

    constructor(string memory _nid, uint16 _protocolFee, address _feeHandler, address _relayer, address _premit2) {
        nid = _nid;
        relayAdress = _relayer;
        protocolFee = _protocolFee;
        feeHandler = _feeHandler;
        permit2 = IPermit2(_premit2);
    }

    function setFeeHandler(address _feeHandler) external onlyOwner {
        feeHandler = _feeHandler;
    }

    function setProtocolFee(uint16 _protocolFee) external onlyOwner {
        protocolFee = _protocolFee;
    }

    function swap(
        Types.SwapOrder memory order
    ) public payable {
        // Escrows amount from user
        address token = order.token.parseAddress("IllegalArgument");
        if (token == NATIVE_ADDRESS) {
            require(msg.value == order.amount, "Deposit amount not equal to order amount");
        } else {
            IERC20(token).safeTransferFrom(msg.sender, address(this), order.amount);
        }

        require(msg.sender == order.creator.parseAddress("IllegalArgument"), "Creator must be sender");

        _swap(order);
    }

    function swapPermit2(
        Types.SwapOrder memory order,
        bytes memory signature,
        IPermit2.PermitTransferFrom calldata _permit
    ) public {
        order.id = 0;
        Permit2OrderLib._processPermit2Order(permit2, order, signature, _permit);
         _swap(order);
    }

    function _swap(
        Types.SwapOrder memory order
    ) private {
        // Create unique deposit ID
        uint256 id = depositId++;
        order.id = id;
        require(order.srcNID.equal(nid), "NID is misconfigured");
        require(order.emitter.equal(address(this).toString()), "Emitter specified is not this");
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
            order.toAmount,
            order.data
        );
    }

    /// @notice Fills an order for a cross-chain swap.
    /// @param order The SwapOrder object.
    /// @param solverAddress The address of the solver filling the order.
    function fill(
        Types.SwapOrder memory order,
        string memory solverAddress
    ) external payable {
        // Compute the hash of the order
        bytes memory orderBytes = order.encode();
        bytes32 orderHash = keccak256(orderBytes);

        // Check if the order has been finished
        require(!finishedOrders[orderHash], "Order has already been filled");
        finishedOrders[orderHash] = true;

        // Transfer tokens
        uint256 fee = (order.toAmount * protocolFee) / 10_000;
        uint256 toAmount = order.toAmount -fee;
        _transferResult(order.destinationAddress, order.toToken, toAmount, fee);

        // Create and send the order message
        Types.OrderFill memory orderFill = Types.OrderFill({
            id: order.id,
            orderBytes: orderBytes,
            solver: solverAddress
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
        emit OrderFilled(order.id, order.srcNID);
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
            "Only creator cancel this order"
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
        _recvMessage(srcNetwork, _connSn);

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
            "Invalid network"
        );

        delete orders[_fill.id];
        emit OrderClosed(_fill.id);

        address tokenAddress = order.token.parseAddress("IllegalArgument");
        if (tokenAddress == NATIVE_ADDRESS) {
            _nativeTransfer(_fill.solver.parseAddress("IllegalArgument"), order.amount);
        } else {
            IERC20(tokenAddress).safeTransfer(
                _fill.solver.parseAddress("IllegalArgument"),
                order.amount
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
            "Invalid network"
        );

        // Load the pending amount if available
        finishedOrders[orderHash] = true;

        Types.OrderFill memory _fill = Types.OrderFill({
            id: order.id,
            orderBytes: orderBytes,
            solver: order.creator
        });

        Types.OrderMessage memory _msg = Types.OrderMessage({
            messageType: Types.FILL,
            message: _fill.encode()
        });

        _sendMessage(order.srcNID, _msg.encode());
        emit OrderCancelled(order.id, order.srcNID);
    }

    function _nativeTransfer(address to, uint256 amount) internal {
        bool sent = payable(to).send(amount);
        require(sent, "Failed to send tokens");
    }

}
