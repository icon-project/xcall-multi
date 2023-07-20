// SPDX-License-Identifier: MIT
pragma solidity ^0.8.12;

import "../utils/RLPDecode.sol";
import "../utils/RLPEncode.sol";

contract RLPCodecMock {
    using RLPDecode for bytes;
    using RLPDecode for RLPDecode.RLPItem;

    constructor() {
    }

    function rlpToInt(bytes memory rlp) public pure returns (int256) {
        return rlp.toRlpItem().toInt();
    }

    function intToRLP(int256 v) public pure returns (bytes memory) {
        return RLPEncode.encodeInt(v);
    }

    function rlpToBytes(bytes memory rlp) public pure returns (bytes memory) {
        return rlp.toRlpItem().toBytes();
    }

    function bytesToRLP(bytes memory data) public pure returns (bytes memory) {
        return RLPEncode.encodeBytes(data);
    }

    function intToBytes(int256 v) public pure returns (bytes memory) {
        return RLPEncode.intToBytes(v);
    }

    function uintToBytes(uint256 v) public pure returns (bytes memory) {
        return RLPEncode.uintToBytes(v);
    }
}
