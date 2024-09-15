// SPDX-License-Identifier: MIT
pragma solidity ^0.8.2;

import "forge-std/Test.sol";
import "@xcall/contracts/Intents/Intents.sol";
import "@xcall/contracts/Intents/Types.sol";
import "@xcall/contracts/Intents/Encoding.sol";
import "openzeppelin-contracts/contracts/token/ERC20/ERC20.sol";
import "openzeppelin-contracts/contracts/token/ERC20/extensions/ERC20Burnable.sol";
import "openzeppelin-contracts/contracts/token/ERC20/extensions/ERC20Capped.sol";

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

    Intents public intents;
    MockERC20 public token;
    address public user;
    address public solver;
    address public feeHandler;

    function setUp() public {
        // Deploy a mock ERC20 token
        token = new MockERC20();

        // Set fee handler address
        feeHandler = address(0x123);

        // Deploy the Intents contract
        intents = new Intents("Network-1", 50, feeHandler);

        // Assign test user address
        user = address(0x456);

        // Transfer some tokens to the user for testing
        token.mint(user, 1_000 * 10 ** token.decimals());

        // Prank as the user to approve the tokens
        vm.prank(user);
        token.approve(address(intents), 500 * 10 ** token.decimals());
    }

    // Test: Swap initiation
    function testSwapInitiation() public {
        vm.startPrank(user);

        // Call swap function
        string memory destinationNetwork = "Network-2";
        bytes memory toToken = abi.encode(address(token));
        bytes memory toAddress = abi.encode(address(0x789));
        uint256 amount = 500 * 10 ** token.decimals();
        uint256 minReceive = 400 * 10 ** token.decimals();
        bytes memory data = "";

        // Expect the SwapIntent event to be emitted
        // vm.expectEmit();
        // emit Intents.SwapIntent(
        //     0,
        //     abi.encode(address(intents)),
        //     "Network-1",
        //     destinationNetwork,
        //     abi.encode(user),
        //     toAddress,
        //     abi.encode(address(token)),
        //     amount,
        //     toToken,
        //     minReceive,
        //     data
        // );

        // Execute the swap
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

        // Validate that tokens have been transferred to the contract
        assertEq(token.balanceOf(address(intents)), amount);

        vm.stopPrank();
    }

    // // Test: Fill function
    // function testFillOrder() public {
    //     vm.startPrank(user);

    //     // Initiate swap order
    //     string memory destinationNetwork = "Network-2";
    //     bytes memory toToken = abi.encode(address(token));
    //     bytes memory toAddress = abi.encode(address(0x789));
    //     uint256 amount = 500 * 10 ** token.decimals();
    //     uint256 minReceive = 400 * 10 ** token.decimals();
    //     bytes memory data = "";
    //     intents.swap(destinationNetwork, address(token), amount, toToken, toAddress, minReceive, data);

    //     // Stop pranking as user, and start as a solver
    //     vm.stopPrank();
    //     address solver = address(0x999);
    //     vm.startPrank(solver);

    //     // Call fill function
    //     uint256 fillAmount = 250 * 10 ** token.decimals();
    //     bytes memory solverAddress = abi.encode(solver);

    //     // Perform the fill action
    //     intents.fill(0, order, fillAmount, solverAddress);

    //     // Verify that the remaining amount is correctly updated
    //     assertEq(intents.pendingFills(keccak256(order.encode())), 250 * 10 ** token.decimals());

    //     vm.stopPrank();
    // }

    // // Test: Swap and fill should revert on invalid amounts
    // function testRevertOnInvalidAmounts() public {
    //     vm.startPrank(user);

    //     // Initiate swap order
    //     string memory destinationNetwork = "Network-2";
    //     bytes memory toToken = abi.encode(address(token));
    //     bytes memory toAddress = abi.encode(address(0x789));
    //     uint256 amount = 500 * 10 ** token.decimals();
    //     uint256 minReceive = 400 * 10 ** token.decimals();
    //     bytes memory data = "";
    //     intents.swap(destinationNetwork, address(token), amount, toToken, toAddress, minReceive, data);

    //     // Stop pranking as user, and start as solver
    //     vm.stopPrank();
    //     address solver = address(0x999);
    //     vm.startPrank(solver);

    //     // Call fill function with invalid amount
    //     Types.SwapOrder memory order = intents.orders(0);
    //     uint256 invalidFillAmount = 600 * 10 ** token.decimals();  // Greater than order amount
    //     bytes memory solverAddress = abi.encode(solver);

    //     // Expect a revert due to invalid fill amount
    //     vm.expectRevert("Cannot fill more than remaining ask");
    //     intents.fill(0, order, invalidFillAmount, solverAddress);

    //     vm.stopPrank();
    // }

    // // Test: Cancelling an order
    // function testCancelOrder() public {
    //     vm.startPrank(user);

    //     // Initiate swap order
    //     string memory destinationNetwork = "Network-2";
    //     bytes memory toToken = abi.encode(address(token));
    //     bytes memory toAddress = abi.encode(address(0x789));
    //     uint256 amount = 500 * 10 ** token.decimals();
    //     uint256 minReceive = 400 * 10 ** token.decimals();
    //     bytes memory data = "";
    //     intents.swap(destinationNetwork, address(token), amount, toToken, toAddress, minReceive, data);

    //     // Cancel the order
    //     intents.cancel(0);

    //     // Validate that the order is cancelled and no longer available
    //     assertEq(intents.finishedOrders(keccak256(intents.orders(0).encode())), true);

    //     vm.stopPrank();
    // }
}
