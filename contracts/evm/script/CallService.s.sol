// SPDX-License-Identifier: MIT
pragma solidity 0.8.18;
import {Script} from "forge-std/Script.sol";
import {UUPSProxy} from "@xcall/contracts/upgradeable/UUPSProxy.sol";
import {console2} from "forge-std/console2.sol";

import "@xcall/contracts/xcall/CallService.sol";

contract DeployCallService is Script {
    CallService private proxyXcall;
    CallService private wrappedProxy;

    using Strings for string;

    UUPSProxy internal proxyContract;

    uint256 internal deployerPrivateKey;
    address internal ownerAddress;
    address internal proxyXcallAddress;

    string internal nid;
    string internal iconNid;
    address internal connection;
    uint256 internal protocolFee;

    constructor() {
        deployerPrivateKey = vm.envUint("PRIVATE_KEY");
        ownerAddress = vm.envAddress("OWNER_ADDRESS");
    }

    modifier broadcast(uint256 privateKey) {
        vm.startBroadcast(privateKey);

        _;

        vm.stopBroadcast();
    }

    function capitalizeString(
        string memory input
    ) public pure returns (string memory) {
        bytes memory inputBytes = bytes(input);
        for (uint i = 0; i < inputBytes.length; i++) {
            if (uint8(inputBytes[i]) >= 97 && uint8(inputBytes[i]) <= 122) {
                inputBytes[i] = bytes1(uint8(inputBytes[i]) - 32);
            }
        }
        return string(inputBytes);
    }

    function deployContract(
        string memory env,
        string memory chain
    ) external broadcast(deployerPrivateKey) {
        env = capitalizeString(env);
        chain = capitalizeString(chain);
        iconNid = vm.envString(env.concat("_ICON_NID"));
        connection = vm.envAddress(env.concat("_BMC_ADDRESS"));
        nid = vm.envString(chain.concat("_NID"));

        CallService xcall = new CallService();
        proxyContract = new UUPSProxy(
            address(xcall),
            abi.encodeWithSelector(CallService.initialize.selector, nid)
        );
        console2.log("CallService address:", address(xcall), "\n");
        console2.log(
            "CallService Proxy address:",
            address(proxyContract),
            "\n"
        );

        proxyXcall = CallService(address(proxyContract));
        proxyXcall.setProtocolFee(protocolFee);
        proxyXcall.setProtocolFeeHandler(ownerAddress);
        proxyXcall.setDefaultConnection(iconNid, connection);
    }

    function upgradeContract(
        string memory chain
    ) external broadcast(deployerPrivateKey) {
        proxyXcallAddress = vm.envAddress(
            capitalizeString(chain).concat("_PROXY")
        );
        CallService xcall = new CallService();
        wrappedProxy = CallService(proxyXcallAddress);
        wrappedProxy.upgradeTo(address(xcall));
    }
}
