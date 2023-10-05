// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.0;

import "@iconfoundation/btp2-solidity-library/utils/RLPEncode.sol";
import "@iconfoundation/btp2-solidity-library/utils/RLPDecode.sol";

library Rollback {
    using RLPDecode for RLPDecode.RLPItem;
    using RLPDecode for RLPDecode.Iterator;
    using RLPDecode for bytes;
    using RLPEncode for bytes;
    using RLPEncode for uint256;

    struct RollbackData {
        uint256 id;
        bytes rollback;
        uint256 ssn;
    }

    function encodeRollbackData(RollbackData memory rbdata)
    internal
    pure
    returns (bytes memory)
    {
        bytes memory _rlp =
        abi.encodePacked(
            rbdata.id.encodeUint(),
            rbdata.rollback.encodeBytes(),
            rbdata.ssn.encodeUint()
        );
        return _rlp.encodeList();
    }

    function decodeRollbackData(bytes memory _rlp)
        internal
        pure
        returns (RollbackData memory)
    {
        RLPDecode.RLPItem[] memory rbdata = _rlp.toRlpItem().toList();
        return
            RollbackData(
                rbdata[0].toUint(),
                rbdata[1].toBytes(),
                rbdata[2].toUint()
            );
    }
}
