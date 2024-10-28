// SPDX-License-Identifier: MIT
pragma solidity ^0.8.22;

import "forge-std/Test.sol";
import "@intents/contracts/Intents/GeneralizedConnection.sol";
import "@intents/contracts/Intents/Intents.sol";
import "@intents/contracts/Intents/Types.sol";
import "@intents/contracts/Intents/Encoding.sol";
import "@iconfoundation/xcall-solidity-library/utils/ParseAddress.sol";
import "openzeppelin-contracts/contracts/token/ERC20/ERC20.sol";
import "openzeppelin-contracts/contracts/token/ERC20/extensions/ERC20Burnable.sol";
import "openzeppelin-contracts/contracts/token/ERC20/extensions/ERC20Capped.sol";
import "forge-std/console.sol";

contract MockERC20 is ERC20 {
    constructor() ERC20("MockToken", "MTK") {
        _mint(msg.sender, 1_000_000 * 10 ** decimals());
    }

    function mint(address to, uint256 amount) public {
        _mint(to, amount);
    }
}

contract IntentsTest is Test {
    using Encoding for *;
    using Strings for string;
    using ParseAddress for address;
    using ParseAddress for string;

    Intents public intents;
    MockERC20 public token;
    address public user;
    address public feeHandler;
    address public relayer;
    address public permit2;
    string public srcNID = "Network-1";

    function setUp() public {
        token = new MockERC20();
        feeHandler = address(0x123);
        relayer = address(0x234);
        permit2 = address(0x345);
        intents = new Intents(srcNID, 50, feeHandler, relayer, permit2);
        user = address(0x456);
    }

    function testSwap() public {
        // Arrange
        string memory destinationNetwork = "Network-2";
        string memory toToken = "0x7891";
        string memory toAddress = "0x789";
        uint256 amount = 500 * 10 ** token.decimals();
        uint256 toAmount = 400 * 10 ** token.decimals();
        bytes memory data = "";
        token.mint(user, 1_000 * 10 ** token.decimals());
        vm.startPrank(user);
        token.approve(address(intents), amount);

        // Expect
        vm.expectEmit(address(intents));
        emit Intents.SwapIntent(
            0,
            address(intents).toString(),
            srcNID,
            destinationNetwork,
            user.toString(),
            toAddress,
            address(token).toString(),
            amount,
            toToken,
            toAmount,
            data
        );

        // Act
        Types.SwapOrder memory order = Types.SwapOrder({
            id: 1,
            emitter:address(intents).toString(),
            srcNID: srcNID,
            dstNID: destinationNetwork,
            creator:user.toString(),
            destinationAddress: toAddress,
            token: address(token).toString(),
            amount: amount,
            toToken: toToken,
            toAmount: toAmount,
            data: data
        });
        intents.swap(order);

        // Assert
        assertEq(token.balanceOf(address(intents)), amount);
    }

     function testSwapNative() public {
        // Arrange
        string memory destinationNetwork = "Network-2";
        string memory toToken = "0x7891";
        string memory toAddress = "0x789";
        uint256 amount = 500 * 10 ** token.decimals();
        uint256 toAmount = 400 * 10 ** token.decimals();
        bytes memory data = "";
        vm.startPrank(user);
        vm.deal(user, amount);

        // Expect
        vm.expectEmit(address(intents));
        emit Intents.SwapIntent(
            0,
            address(intents).toString(),
            srcNID,
            destinationNetwork,
            user.toString(),
            toAddress,
            address(0).toString(),
            amount,
            toToken,
            toAmount,
            data
        );

        // Act
        Types.SwapOrder memory order = Types.SwapOrder({
            id: 1,
            emitter:address(intents).toString(),
            srcNID: srcNID,
            dstNID: destinationNetwork,
            creator:user.toString(),
            destinationAddress: toAddress,
            token: address(0).toString(),
            amount: amount,
            toToken: toToken,
            toAmount: toAmount,
            data: data
        });
        intents.swap{value: amount}(order);

        // Assert
        assertEq(address(intents).balance, amount);
    }

    function testSwapInvalidOrder() public {
        // Arrange
        string memory destinationNetwork = "Network-2";
        string memory toToken = "0x7891";
        string memory toAddress = "0x789";
        uint256 amount = 500 * 10 ** token.decimals();
        uint256 toAmount = 400 * 10 ** token.decimals();
        bytes memory data = "";
        token.mint(user, 1_000 * 10 ** token.decimals());
        vm.startPrank(user);
        token.approve(address(intents), amount);

        // Expect
        vm.expectRevert("Creator must be sender");

        // Act
        Types.SwapOrder memory order = Types.SwapOrder({
            id: 1,
            emitter:address(intents).toString(),
            srcNID: srcNID,
            dstNID: destinationNetwork,
            creator: address(intents).toString(),
            destinationAddress: toAddress,
            token: address(token).toString(),
            amount: amount,
            toToken: toToken,
            toAmount: toAmount,
            data: data
        });
        intents.swap(order);

        // Expect
        vm.expectRevert("NID is misconfigured");

        // Act
        order.creator = address(user).toString();
        order.srcNID = "dummy";
        intents.swap(order);

        // Expect
        vm.expectRevert("Emitter specified is not this");

        // Act
        order.srcNID = srcNID;
        order.emitter = address(user).toString();
        intents.swap(order);
    }


    function testFillOrder() public {
        // Arrange
        string memory destinationNetwork = "Network-2";
        string memory toToken = address(token).toString();
        address toAddress = address(0x789);
        uint256 amount = 400 * 10 ** token.decimals();
        uint256 toAmount = 500 * 10 ** token.decimals();
        bytes memory data = "";

        address solver = address(0x9999);
        string memory solverAddress = solver.toString();
        token.mint(solver, 1_000 * 10 ** token.decimals());
        vm.startPrank(solver);
        token.approve(address(intents), toAmount);

        Types.SwapOrder memory order = Types.SwapOrder({
            id: 1,
            emitter:address(intents).toString(),
            srcNID: srcNID,
            dstNID: destinationNetwork,
            creator: address(0x1234).toString(),
            destinationAddress: toAddress.toString(),
            token: address(token).toString(),
            amount: amount,
            toToken: toToken,
            toAmount: toAmount,
            data: data
        });

        // Expect
        vm.expectEmit(address(intents));
        Types.OrderFill memory orderFill = Types.OrderFill({
            id: order.id,
            orderBytes: order.encode(),
            solver:  solverAddress
        });
        Types.OrderMessage memory message = Types.OrderMessage({
            messageType: Types.FILL,
            message: orderFill.encode()
        });
        emit GeneralizedConnection.Message(order.srcNID, 1, message.encode());

        // Act
        intents.fill(order, solverAddress);

        // Assert
        uint256 fee = toAmount*intents.protocolFee() / 10_000;
        assertEq(token.balanceOf(toAddress), toAmount - fee);
        assertEq(token.balanceOf(feeHandler), fee);
        assertTrue(intents.finishedOrders(keccak256(order.encode())));
    }


    function testFillOrderNative() public {
        // Arrange
        string memory destinationNetwork = "Network-2";
        address toAddress = address(0x789);
        uint256 amount = 400 * 10 ** token.decimals();
        uint256 toAmount = 500 * 10 ** token.decimals();
        bytes memory data = "";

        address solver = address(0x9999);
        string memory solverAddress = solver.toString();
        vm.deal(solver, toAmount);

        Types.SwapOrder memory order = Types.SwapOrder({
            id: 1,
            emitter:address(intents).toString(),
            srcNID: srcNID,
            dstNID: destinationNetwork,
            creator: address(0x1234).toString(),
            destinationAddress: toAddress.toString(),
            token: address(token).toString(),
            amount: amount,
            toToken: address(0).toString(),
            toAmount: toAmount,
            data: data
        });

        // Expect
        vm.expectEmit(address(intents));
        Types.OrderFill memory orderFill = Types.OrderFill({
            id: order.id,
            orderBytes: order.encode(),
            solver:  solverAddress
        });
        Types.OrderMessage memory message = Types.OrderMessage({
            messageType: Types.FILL,
            message: orderFill.encode()
        });
        emit GeneralizedConnection.Message(order.srcNID, 1, message.encode());

        // Act
        vm.startPrank(solver);
        intents.fill {value: toAmount}(order, solverAddress);

        // Assert
        uint256 fee = toAmount*intents.protocolFee() / 10_000;
        assertEq(toAddress.balance, toAmount - fee);
        assertEq(feeHandler.balance, fee);
        assertTrue(intents.finishedOrders(keccak256(order.encode())));
    }


    function testFillOrderAlreadyFilled() public {
        // Arrange
        string memory destinationNetwork = "Network-2";
        string memory toToken = address(token).toString();
        address toAddress = address(0x789);
        uint256 amount = 400 * 10 ** token.decimals();
        uint256 toAmount = 500 * 10 ** token.decimals();
        bytes memory data = "";

        address solver = address(0x9999);
        string memory solverAddress = solver.toString();
        token.mint(solver, 1_000 * 10 ** token.decimals());
        vm.startPrank(solver);
        token.approve(address(intents), toAmount);

        Types.SwapOrder memory order = Types.SwapOrder({
            id: 1,
            emitter:address(intents).toString(),
            srcNID: srcNID,
            dstNID: destinationNetwork,
            creator: address(0x1234).toString(),
            destinationAddress: toAddress.toString(),
            token: address(token).toString(),
            amount: amount,
            toToken: toToken,
            toAmount: toAmount,
            data: data
        });
        intents.fill(order, solverAddress);

        // Expect
        vm.expectRevert("Order has already been filled");

        // Act
        intents.fill(order, solverAddress);
    }

    function testFillOrdersameChain() public {
        // Arrange
        MockERC20 token2 = new MockERC20();
        string memory toToken = address(token2).toString();
        address toAddress = address(0x789);
        uint256 amount = 400 * 10 ** token.decimals();
        uint256 toAmount = 500 * 10 ** token.decimals();
        bytes memory data = "";
        token.mint(user, 1_000 * 10 ** token.decimals());

        address solver = address(0x9999);
        string memory solverAddress = solver.toString();
        token2.mint(solver, 1_000 * 10 ** token.decimals());

        // Execute the swap
         vm.startPrank(user);
        token.approve(address(intents), amount);
        Types.SwapOrder memory order = Types.SwapOrder({
            id: 0,
            emitter: address(intents).toString(),
            srcNID: srcNID,
            dstNID: srcNID,
            creator: user.toString(),
            destinationAddress: toAddress.toString(),
            token: address(token).toString(),
            amount: amount,
            toToken: toToken,
            toAmount: toAmount,
            data: data
        });

        vm.startPrank(user);
        intents.swap(order);

        // Act
        vm.startPrank(solver);
        token2.approve(address(intents), toAmount);
        intents.fill(order, solverAddress);

        // Assert
        assertTrue(intents.finishedOrders(keccak256(order.encode())));
        order = intents.getOrder(0);
        assertEq(order.srcNID, "");

        uint256 fee = toAmount*intents.protocolFee() / 10_000;
        assertEq(token2.balanceOf(toAddress), toAmount - fee);
        assertEq(token2.balanceOf(feeHandler), fee);
        assertEq(token.balanceOf(solver), amount);
    }

    function testCancelOrder() public {
        // Arrange
        string memory destinationNetwork = "Network-2";
        string memory toToken = address(token).toString();
        string memory toAddress = address(0x789).toString();
        uint256 amount = 500 * 10 ** token.decimals();
        uint256 toAmount = 400 * 10 ** token.decimals();
        bytes memory data = "";
        token.mint(user, amount);

        vm.startPrank(user);
        token.approve(address(intents), amount);
        Types.SwapOrder memory order = Types.SwapOrder({
            id: 0,
            emitter:address(intents).toString(),
            srcNID: srcNID,
            dstNID: destinationNetwork,
            creator: user.toString(),
            destinationAddress: toAddress,
            token: address(token).toString(),
            amount: amount,
            toToken: toToken,
            toAmount: toAmount,
            data: data
        });
        intents.swap(order);
        order = intents.getOrder(0);

        // Expect
        vm.expectEmit(address(intents));
        Types.Cancel memory cancel = Types.Cancel({
            orderBytes: order.encode()
        });

        Types.OrderMessage memory message = Types.OrderMessage({
            messageType: Types.CANCEL,
            message: cancel.encode()
        });

        emit GeneralizedConnection.Message(destinationNetwork, 1, message.encode());

        // Act
        intents.cancel(0);
    }

      function testCancelOrderInvalidSender() public {
        // Arrange
        string memory destinationNetwork = "Network-2";
        string memory toToken = address(token).toString();
        string memory toAddress = address(0x789).toString();
        uint256 amount = 500 * 10 ** token.decimals();
        uint256 toAmount = 400 * 10 ** token.decimals();
        bytes memory data = "";
        token.mint(user, amount);

        vm.startPrank(user);
        token.approve(address(intents), amount);
        Types.SwapOrder memory order = Types.SwapOrder({
            id: 0,
            emitter:address(intents).toString(),
            srcNID: srcNID,
            dstNID: destinationNetwork,
            creator: user.toString(),
            destinationAddress: toAddress,
            token: address(token).toString(),
            amount: amount,
            toToken: toToken,
            toAmount: toAmount,
            data: data
        });
        intents.swap(order);
        order = intents.getOrder(0);

        // Expect
        vm.expectRevert("Only creator cancel this order");

        // Act
        vm.startPrank(relayer);
        intents.cancel(0);
    }

    function testResolveOrder() public {
        // Arrange
        string memory destinationNetwork = "Network-2";
        string memory toToken = address(token).toString();
        string memory toAddress = address(0x789).toString();
        address solver = address(0x891);
        uint256 amount = 500 * 10 ** token.decimals();
        uint256 toAmount = 400 * 10 ** token.decimals();
        bytes memory data = "";
        token.mint(user, 1_000 * 10 ** token.decimals());

        // Execute the swap
        vm.startPrank(user);
        token.approve(address(intents), amount);
        Types.SwapOrder memory order = Types.SwapOrder({
            id: 0,
            emitter:address(intents).toString(),
            srcNID: srcNID,
            dstNID: destinationNetwork,
            creator: user.toString(),
            destinationAddress: toAddress,
            token: address(token).toString(),
            amount: amount,
            toToken: toToken,
            toAmount: toAmount,
            data: data
        });
        intents.swap(order);
        order = intents.getOrder(0);
        Types.OrderFill memory orderFill = Types.OrderFill({
            id: 0,
            orderBytes: order.encode(),
            solver:  solver.toString()
        });

        Types.OrderMessage memory orderMessage = Types.OrderMessage({
            messageType: Types.FILL,
            message: orderFill.encode()
        });

        // Act
        vm.startPrank(relayer);
        intents.recvMessage(destinationNetwork, 1, orderMessage.encode());

        // Assert
        order = intents.getOrder(0);
        assertEq(order.srcNID, "");
        assertEq(token.balanceOf(solver), amount);
    }

    function testResolveMismatchedOrder() public {
        // Arrange
        string memory destinationNetwork = "Network-2";
        string memory toToken = address(token).toString();
        string memory toAddress = address(0x789).toString();
        address solver = address(0x891);
        uint256 amount = 500 * 10 ** token.decimals();
        uint256 toAmount = 400 * 10 ** token.decimals();
        bytes memory data = "";
        token.mint(user, 1_000 * 10 ** token.decimals());

        // Execute the swap
        vm.startPrank(user);
        token.approve(address(intents), amount);
        Types.SwapOrder memory order = Types.SwapOrder({
            id: 0,
            emitter:address(intents).toString(),
            srcNID: srcNID,
            dstNID: destinationNetwork,
            creator: user.toString(),
            destinationAddress: toAddress,
            token: address(token).toString(),
            amount: amount,
            toToken: toToken,
            toAmount: toAmount,
            data: data
        });
        intents.swap(order);
        order = intents.getOrder(0);

        Types.OrderFill memory orderFill = Types.OrderFill({
            id: 0,
            orderBytes: order.encode(),
            solver:  solver.toString()
        });

        Types.OrderMessage memory orderMessage = Types.OrderMessage({
            messageType: Types.FILL,
            message: orderFill.encode()
        });

        // Expect
        vm.expectRevert("Invalid network");

        // Act
        vm.startPrank(relayer);
        intents.recvMessage("Network-3", 1, orderMessage.encode());

        // Arrange
        order.srcNID = "test";
        orderFill = Types.OrderFill({
            id: 0,
            orderBytes: order.encode(),
            solver:  solver.toString()
        });

        orderMessage = Types.OrderMessage({
            messageType: Types.FILL,
            message: orderFill.encode()
        });

        // Expect
        vm.expectRevert("Mismatched order");

        // Act
        intents.recvMessage(destinationNetwork, 1, orderMessage.encode());
    }


    function testResolveCancel() public {
        // Arrange
        string memory destinationNetwork = "Network-2";
        string memory toToken = address(token).toString();
        string memory toAddress = address(0x789).toString();
        uint256 amount = 500 * 10 ** token.decimals();
        uint256 toAmount = 400 * 10 ** token.decimals();
        bytes memory data = "";
        token.mint(user, 1_000 * 10 ** token.decimals());


        Types.SwapOrder memory order = Types.SwapOrder({
            id: 0,
            emitter:address(intents).toString(),
            srcNID: srcNID,
            dstNID: destinationNetwork,
            creator: user.toString(),
            destinationAddress: toAddress,
            token: address(token).toString(),
            amount: amount,
            toToken: toToken,
            toAmount: toAmount,
            data: data
        });

        Types.Cancel memory cancel = Types.Cancel({
            orderBytes: order.encode()
        });

        Types.OrderMessage memory message = Types.OrderMessage({
            messageType: Types.CANCEL,
            message: cancel.encode()
        });

        vm.startPrank(relayer);

        // Assert
        // Invalid network check
        vm.expectRevert("Invalid network");
        intents.recvMessage("Network-3", 1, message.encode());

        // Expect
        vm.expectEmit(address(intents));
        Types.OrderFill memory orderFill = Types.OrderFill({
            id: order.id,
            orderBytes: order.encode(),
            solver:  order.creator
        });
        Types.OrderMessage memory fillMessage = Types.OrderMessage({
            messageType: Types.FILL,
            message: orderFill.encode()
        });
        emit GeneralizedConnection.Message(order.srcNID, 1, fillMessage.encode());
        emit Intents.OrderCancelled(order.id, order.srcNID);

        // Act

        intents.recvMessage(srcNID, 1, message.encode());

        // Assert
        assertTrue(intents.finishedOrders(keccak256(order.encode())));
    }

    function testSetFeeHandler() public {
        address newFeeHandler = address(0x891);
        intents.setFeeHandler(newFeeHandler);
        assertEq(intents.feeHandler(), newFeeHandler);

        address nonOwner = address(0x892);
        vm.startPrank(nonOwner);

        vm.expectRevert("Ownable: caller is not the owner");
        intents.setFeeHandler(newFeeHandler);
    }

    function testSetProtocolFee() public {
        uint16 newFee = 13;
        intents.setProtocolFee(newFee);
        assertEq(intents.protocolFee(), newFee);

        address nonOwner = address(0x892);
        vm.startPrank(nonOwner);

        vm.expectRevert("Ownable: caller is not the owner");
        intents.setProtocolFee(newFee);
    }
}
