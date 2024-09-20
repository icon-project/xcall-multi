// SPDX-License-Identifier: MIT
pragma solidity ^0.8.22;

import "forge-std/Test.sol";
import "@xcall/contracts/Intents/GeneralizedConnection.sol";
import "@xcall/contracts/Intents/Intents.sol";
import "@xcall/contracts/Intents/Types.sol";
import "@xcall/contracts/Intents/Encoding.sol";
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
    string public srcNID = "Network-1";

    function setUp() public {
        token = new MockERC20();
        feeHandler = address(0x123);
        relayer = address(0x234);
        intents = new Intents(srcNID, 50, feeHandler, relayer);
        user = address(0x456);
    }

    function testSwapInitiation() public {
        vm.startPrank(user);

        // Call swap function
        string memory destinationNetwork = "Network-2";
        string memory toToken = "0x7891";
        string memory toAddress = "0x789";
        uint256 amount = 500 * 10 ** token.decimals();
        uint256 minReceive = 400 * 10 ** token.decimals();
        bytes memory data = "";
        token.mint(user, 1_000 * 10 ** token.decimals());
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
            minReceive,
            data
        );

        // Execute the swap
        intents.swap(
            destinationNetwork,
            address(token),
            amount,
            toToken,
            toAddress,
            minReceive,
            data
        );

        // Validate that tokens have been transferred to the contract
        assertEq(token.balanceOf(address(intents)), amount);

        vm.stopPrank();
    }

    function testFillOrder() public {
        // Arrange
        string memory destinationNetwork = "Network-2";
        string memory toToken = address(token).toString();
        address toAddress = address(0x789);
        uint256 amount = 400 * 10 ** token.decimals();
        uint256 minReceive = 500 * 10 ** token.decimals();
        bytes memory data = "";
        uint256 fillAmount = 250 * 10 ** token.decimals();
        uint256 payout = 200 * 10 ** token.decimals();

        address solver = address(0x9999);
        string memory solverAddress = solver.toString();
        token.mint(solver, 1_000 * 10 ** token.decimals());
        vm.startPrank(solver);
        token.approve(address(intents), fillAmount*2);

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
            minReceive: minReceive,
            data: data
        });

        // Expect
        vm.expectEmit(address(intents));
        Types.OrderFill memory orderFill = Types.OrderFill({
            id: order.id,
            orderBytes: order.encode(),
            solver:  solverAddress,
            amount: payout,
            closeOrder: false
        });
        Types.OrderMessage memory message = Types.OrderMessage({
            messageType: Types.FILL,
            message: orderFill.encode()
        });
        emit GeneralizedConnection.Message(order.srcNID, 1, message.encode());

        // Act
        intents.fill(order, fillAmount, solverAddress);

        // Assert
        assertEq(intents.pendingFills(keccak256(order.encode())), payout);
        uint256 fee = fillAmount*intents.protocolFee() / 10_000;
        assertEq(token.balanceOf(toAddress), fillAmount - fee);
        assertEq(token.balanceOf(feeHandler), fee);

        // Expect
        vm.expectEmit(address(intents));
        orderFill = Types.OrderFill({
            id: order.id,
            orderBytes: order.encode(),
            solver:  solverAddress,
            amount: payout,
            closeOrder: true
        });
        message = Types.OrderMessage({
            messageType: Types.FILL,
            message: orderFill.encode()
        });
        emit GeneralizedConnection.Message(order.srcNID, 2, message.encode());

         // Act
        intents.fill(order, fillAmount, solverAddress);

        // Assert
        assertEq(intents.pendingFills(keccak256(order.encode())), 0);
        assertTrue(intents.finishedOrders(keccak256(order.encode())));
        fee = minReceive*intents.protocolFee() / 10_000;
        assertEq(token.balanceOf(toAddress), minReceive - fee);
        assertEq(token.balanceOf(feeHandler), fee);
    }

    function testFillOrdersameChain() public {
        // Arrange
        MockERC20 token2 = new MockERC20();
        string memory toToken = address(token2).toString();
        address toAddress = address(0x789);
        uint256 amount = 400 * 10 ** token.decimals();
        uint256 minReceive = 500 * 10 ** token.decimals();
        bytes memory data = "";
        token.mint(user, 1_000 * 10 ** token.decimals());

        address solver = address(0x9999);
        string memory solverAddress = solver.toString();
        token2.mint(solver, 1_000 * 10 ** token.decimals());

        // Execute the swap
         vm.startPrank(user);
        token.approve(address(intents), amount);
        intents.swap(
            srcNID,
            address(token),
            amount,
            toToken,
            toAddress.toString(),
            minReceive,
            data
        );

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
            minReceive: minReceive,
            data: data
        });

        // Act
        vm.startPrank(solver);
        token2.approve(address(intents), minReceive);
        intents.fill(order, minReceive, solverAddress);

        // Assert
        assertTrue(intents.finishedOrders(keccak256(order.encode())));
        order = intents.getOrder(0);

        // order has been removed
        assertEq(order.srcNID, "");
        uint256 fee = minReceive*intents.protocolFee() / 10_000;
        assertEq(token2.balanceOf(toAddress), minReceive - fee);
        assertEq(token2.balanceOf(feeHandler), fee);
        assertEq(token.balanceOf(solver), amount);
    }

    function testResolveOrder() public {
        // Arrange
        string memory destinationNetwork = "Network-2";
        string memory toToken = address(token).toString();
        string memory toAddress = address(0x789).toString();
        address solver = address(0x891);
        uint256 amount = 500 * 10 ** token.decimals();
        uint256 minReceive = 400 * 10 ** token.decimals();
        bytes memory data = "";
        token.mint(user, 1_000 * 10 ** token.decimals());

        // Execute the swap
        vm.startPrank(user);
        token.approve(address(intents), amount);
        intents.swap(
            destinationNetwork,
            address(token),
            amount,
            toToken,
            toAddress,
            minReceive,
            data
        );
        Types.SwapOrder memory order = intents.getOrder(0);
        Types.OrderFill memory orderFill = Types.OrderFill({
            id: 0,
            orderBytes: order.encode(),
            solver:  solver.toString(),
            amount: amount,
            closeOrder: true
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

        // order has been removed
        assertEq(order.srcNID, "");
        assertEq(token.balanceOf(solver), amount);
    }

    function testCancelOrder() public {
        // Arrange
        string memory destinationNetwork = "Network-2";
        string memory toToken = address(token).toString();
        string memory toAddress = address(0x789).toString();
        uint256 amount = 500 * 10 ** token.decimals();
        uint256 minReceive = 400 * 10 ** token.decimals();
        bytes memory data = "";
        token.mint(user, amount);

        vm.startPrank(user);
        token.approve(address(intents), amount);
        intents.swap(destinationNetwork, address(token), amount, toToken, toAddress, minReceive, data);
        Types.SwapOrder memory order = intents.getOrder(0);

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
}
