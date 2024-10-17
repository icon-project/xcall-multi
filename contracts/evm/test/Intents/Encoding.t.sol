// SPDX-License-Identifier: MIT
pragma solidity ^0.8.17;

import "forge-std/Test.sol";
import "@intents/contracts/Intents/Intents.sol";
import "@intents/contracts/Intents/Types.sol";
import "@intents/contracts/Intents/Encoding.sol";
import {console} from "forge-std/console.sol";

contract EncodingTest is Test {
    using Types for *; // Enables use of the Types library in this contract
    using Encoding for *;

    function testSwapOrder() public {
        // Create multiple SwapOrder structs with different values
        Types.SwapOrder memory order1 = Types.SwapOrder({
            id: 1,
            emitter: "0xbe6452d4d6c61cee97d3",
            srcNID: "Ethereum",
            dstNID: "Polygon",
            creator: "0x3e36eddd65e239222e7e67",
            destinationAddress: "0xd2c6218b875457a41b6fb7964e",
            token: "0x14355340e857912188b7f202d550222487",
            amount: 1000,
            toToken:"0x91a4728b517484f0f610de7b",
            toAmount: 900,
            data: ""
        });

        bytes memory expectedBytes = hex"f8a601963078626536343532643464366336316365653937643388457468657265756d87506f6c79676f6e983078336533366564646436356532333932323265376536379c30786432633632313862383735343537613431623666623739363465a43078313433353533343065383537393132313838623766323032643535303232323438378203e89a307839316134373238623531373438346630663631306465376282038480";
        assertEq(order1.encode(), expectedBytes);
        Types.SwapOrder memory order2 = Types.SwapOrder({
                id: 1,
                emitter: "0xbe6452d4d6c61cee97d3",
                srcNID: "Ethereum",
                dstNID: "Polygon",
                creator: "0x3e36eddd65e239222e7e67",
                destinationAddress: "0xd2c6218b875457a41b6fb7964e",
                token: "0x14355340e857912188b7f202d550222487",
                amount: 100000*10**22,
                toToken:"0x91a4728b517484f0f610de7b",
                toAmount: 900*10**7,
                data: hex"6c449988e2f33302803c93f8287dc1d8cb33848a"
            });
        expectedBytes = hex"f8c701963078626536343532643464366336316365653937643388457468657265756d87506f6c79676f6e983078336533366564646436356532333932323265376536379c30786432633632313862383735343537613431623666623739363465a43078313433353533343065383537393132313838623766323032643535303232323438378c033b2e3c9fd0803ce80000009a3078393161343732386235313734383466306636313064653762850218711a00946c449988e2f33302803c93f8287dc1d8cb33848a";
        assertEq(order2.encode(), expectedBytes);

    }

    function testOrderMessage() public {
        Types.OrderMessage memory fillMessage = Types.OrderMessage({
            messageType: Types.FILL,
            message: hex"6c449988e2f33302803c93f8287dc1d8cb33848a"
        });

        bytes memory expectedBytes = hex"d601946c449988e2f33302803c93f8287dc1d8cb33848a";
        assertEq(fillMessage.encode(), expectedBytes);

        Types.OrderMessage memory cancelMessage = Types.OrderMessage({
            messageType: Types.CANCEL,
            message: hex"6c449988e2f33302803c93f8287dc1d8cb33848a"
        });

        expectedBytes = hex"d602946c449988e2f33302803c93f8287dc1d8cb33848a";
        assertEq(cancelMessage.encode(), expectedBytes);

    }

    function testOrderFill() public {
        Types.OrderFill memory fill1 = Types.OrderFill({
            id: 1,
            orderBytes: hex"6c449988e2f33302803c93f8287dc1d8cb33848a",
            solver: "0xcb0a6bbccfccde6be9f10ae781b9d9b00d6e63"
        });

        bytes memory expectedBytes = hex"f83f01946c449988e2f33302803c93f8287dc1d8cb33848aa830786362306136626263636663636465366265396631306165373831623964396230306436653633";
        assertEq(fill1.encode(), expectedBytes);

        Types.OrderFill memory fill2 = Types.OrderFill({
            id: 2,
            orderBytes: hex"cb0a6bbccfccde6be9f10ae781b9d9b00d6e63",
            solver: "0x6c449988e2f33302803c93f8287dc1d8cb33848a"
        });

        expectedBytes = hex"f8400293cb0a6bbccfccde6be9f10ae781b9d9b00d6e63aa307836633434393938386532663333333032383033633933663832383764633164386362333338343861";
        assertEq(fill2.encode(), expectedBytes);
    }

    function testCancel() public {
        // Create Cancel struct
        Types.Cancel memory cancel = Types.Cancel({
            orderBytes: hex"6c449988e2f33302803c93f8287dc1d8cb33848a"
        });

        bytes memory expectedBytes = hex"d5946c449988e2f33302803c93f8287dc1d8cb33848a";
        assertEq(cancel.encode(), expectedBytes);
    }
}