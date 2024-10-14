// SPDX-License-Identifier: Apache-2.0
pragma solidity >=0.8.0;
pragma abicoder v2;

import "@iconfoundation/xcall-solidity-library/utils/RLPEncode.sol";
import "@iconfoundation/xcall-solidity-library/utils/RLPDecode.sol";
import "./Types.sol";

library Encoding {
    using RLPEncode for *;
    using RLPDecode for *;

    function decodeSwapOrder(
        bytes memory _rlp
    ) internal pure returns (Types.SwapOrder memory) {
        RLPDecode.RLPItem[] memory ls = _rlp.toRlpItem().toList();
        return
            Types.SwapOrder(
                ls[0].toUint(), // id
                string(ls[1].toBytes()), // emitter
                string(ls[2].toBytes()), // srcNID
                string(ls[3].toBytes()), // dstNID
                string(ls[4].toBytes()), // creator
                string(ls[5].toBytes()), // destinationAddress
                string(ls[6].toBytes()), // token
                ls[7].toUint(), // amount
                string(ls[8].toBytes()), // toToken
                ls[9].toUint(), // toAmount
                ls[10].toBytes() // data
            );
    }

    function decodeOrderMessage(
        bytes memory _rlp
    ) internal pure returns (Types.OrderMessage memory) {
        RLPDecode.RLPItem[] memory ls = _rlp.toRlpItem().toList();
        return
            Types.OrderMessage(
                ls[0].toUint(), // messageType
                ls[1].toBytes() // message
            );
    }

    function decodeOrderFill(
        bytes memory _rlp
    ) internal pure returns (Types.OrderFill memory) {
        RLPDecode.RLPItem[] memory ls = _rlp.toRlpItem().toList();
        return
            Types.OrderFill(
                ls[0].toUint(), // id
                ls[1].toBytes(), // orderBytes
                string(ls[2].toBytes()) // solver
            );
    }

    function decodeCancel(
        bytes memory _rlp
    ) internal pure returns (Types.Cancel memory) {
        RLPDecode.RLPItem[] memory ls = _rlp.toRlpItem().toList();
        return
            Types.Cancel(
                ls[0].toBytes() // orderBytes
            );
    }

    /// @notice Encodes a `SwapOrder` struct into an RLP-encoded byte array.
    function encode(
        Types.SwapOrder memory order
    ) internal pure returns (bytes memory) {
        bytes memory encoded = abi.encodePacked(
            _encodeSwapOrderPart1(order),
            _encodeSwapOrderPart2(order)
        );
        return encoded.encodeList();
    }

    // Avoid stack to deep error
    function _encodeSwapOrderPart1(
        Types.SwapOrder memory order
    ) internal pure returns (bytes memory) {
        bytes memory encoded = abi.encodePacked(
            order.id.encodeUint(),
            order.emitter.encodeString(),
            order.srcNID.encodeString(),
            order.dstNID.encodeString(),
            order.creator.encodeString(),
            order.destinationAddress.encodeString()
        );
        return encoded;
    }

    function _encodeSwapOrderPart2(
        Types.SwapOrder memory order
    ) internal pure returns (bytes memory) {
        bytes memory encoded = abi.encodePacked(
            order.token.encodeString(),
            order.amount.encodeUint(),
            order.toToken.encodeString(),
            order.toAmount.encodeUint(),
            order.data.encodeBytes()
        );
        return encoded;
    }

    /// @notice Encodes an `OrderMessage` struct into an RLP-encoded byte array.
    function encode(
        Types.OrderMessage memory message
    ) internal pure returns (bytes memory) {
        bytes memory encoded = abi.encodePacked(
            message.messageType.encodeUint(),
            message.message.encodeBytes()
        );
        return encoded.encodeList();
    }

    /// @notice Encodes an `OrderFill` struct into an RLP-encoded byte array.
    function encode(
        Types.OrderFill memory fill
    ) internal pure returns (bytes memory) {
        bytes memory encoded = abi.encodePacked(
            fill.id.encodeUint(),
            fill.orderBytes.encodeBytes(),
            fill.solver.encodeString()
        );
        return encoded.encodeList();
    }

    /// @notice Encodes a `Cancel` struct into an RLP-encoded byte array.
    function encode(
        Types.Cancel memory cancel
    ) internal pure returns (bytes memory) {
        bytes memory encoded = abi.encodePacked(
            cancel.orderBytes.encodeBytes()
        );
        return encoded.encodeList();
    }
}
