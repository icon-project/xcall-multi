// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.0;
import "@iconfoundation/xcall-solidity-library/utils/ParseAddress.sol";

import "./Types.sol";
import "./IPermit2.sol";
import {console} from "forge-std/console.sol";

library Permit2OrderLib {
    using ParseAddress for string;

    bytes internal constant ORDER_TYPE =
        abi.encodePacked(
            "SwapOrder(",
            "uint256 id,",
            "string emitter,",
            "string srcNID,",
            "string dstNID,",
            "string creator,",
            "string destinationAddress,",
            "string token,",
            "uint256 amount,",
            "string toToken,",
            "uint256 toAmount,",
            "bytes data)"
        );
    bytes32 internal constant ORDER_TYPE_HASH = keccak256(ORDER_TYPE);
    string private constant TOKEN_PERMISSIONS_TYPE =
        "TokenPermissions(address token,uint256 amount)";
    string internal constant PERMIT2_ORDER_TYPE =
        string(
            abi.encodePacked(
                "SwapOrder witness)",
                ORDER_TYPE,
                TOKEN_PERMISSIONS_TYPE
            )
        );
    // Hashes an order to get an order hash. Needed for permit2.
    function _hashOrder(
        Types.SwapOrder memory order
    ) internal pure returns (bytes32) {
        bytes memory orderData = abi.encode(
            ORDER_TYPE_HASH,
            order.id,
            keccak256(abi.encodePacked(order.emitter)),
            keccak256(abi.encodePacked(order.srcNID)),
            keccak256(abi.encodePacked(order.dstNID)),
            keccak256(abi.encodePacked(order.creator)),
            keccak256(abi.encodePacked(order.destinationAddress)),
            keccak256(abi.encodePacked(order.token)),
            order.amount,
            keccak256(abi.encodePacked(order.toToken)),
            order.toAmount,
            keccak256(order.data)
        );

        return keccak256(orderData);
    }

    function _processPermit2Order(
        IPermit2 permit2,
        Types.SwapOrder memory order,
        bytes memory signature,
        IPermit2.PermitTransferFrom memory permit
    ) internal {
        permit2.permitWitnessTransferFrom(
            permit,
            IPermit2.SignatureTransferDetails({
                to: address(this),
                requestedAmount: order.amount
            }),
            order.creator.parseAddress("IllegalArgument"),
            _hashOrder(order),
            PERMIT2_ORDER_TYPE,
            signature
        );
    }
}
