// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.0;
import "@iconfoundation/xcall-solidity-library/utils/ParseAddress.sol";

import "./Types.sol";
import "./IPermit2.sol";

/**
 * @notice Permit2OrderLib knows how to process a particular type of external Permit2Order so that it can be used in Across.
 * @dev This library is responsible for validating the order and communicating with Permit2 to pull the tokens in.
 * This is a library to allow it to be pulled directly into the SpokePool in a future version.
 */
library Permit2OrderLib {
    using ParseAddress for string;

    // Type strings and hashes
    bytes private constant OUTPUT_TOKEN_TYPE =
        "OutputToken(address recipient,address token,uint256 amount,uint256 chainId)";
    bytes32 private constant OUTPUT_TOKEN_TYPE_HASH = keccak256(OUTPUT_TOKEN_TYPE);

    bytes internal constant ORDER_TYPE =
        abi.encodePacked(
            "SwapOrder(",
            "string emitter,",
            "string srcNID,",
            "string dstNID,",
            "string creator,",
            "string destinationAddress,",
            "string token,",
            "uint256 amount,",
            "string toToken,",
            "uint256 toAmount,",
            "bytes data,"
        );
    bytes32 internal constant ORDER_TYPE_HASH = keccak256(ORDER_TYPE);
    string private constant TOKEN_PERMISSIONS_TYPE = "TokenPermissions(address token,uint256 amount)";
    string internal constant PERMIT2_ORDER_TYPE =
        string(abi.encodePacked("SwapOrder witness)", ORDER_TYPE, TOKEN_PERMISSIONS_TYPE));

    // Hashes an order to get an order hash. Needed for permit2.
    function _hashOrder(Types.SwapOrder memory order) internal pure returns (bytes32) {
        bytes memory orderData = abi.encode(
            ORDER_TYPE_HASH,
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

        return keccak256(orderData);
    }

    function _processPermit2Order(IPermit2 permit2, Types.SwapOrder memory order, bytes memory signature, uint32 deadline)
        internal
    {
        address token = order.token.parseAddress("IllegalArgument");
        IPermit2.PermitTransferFrom memory permit = IPermit2.PermitTransferFrom({
            permitted: IPermit2.TokenPermissions({ token: token , amount: order.amount }),
            nonce: order.id,
            deadline: deadline
        });

        IPermit2.SignatureTransferDetails memory signatureTransferDetails = IPermit2.SignatureTransferDetails({
            to: address(this),
            requestedAmount: order.amount
        });

        // Pull user funds.
        permit2.permitWitnessTransferFrom(
            permit,
            signatureTransferDetails,
            order.creator.parseAddress("IllegalArgument"),
            _hashOrder(order),
            PERMIT2_ORDER_TYPE,
            signature
        );
    }
}