// SPDX-License-Identifier: MIT
pragma solidity >=0.8.18;
import {Script} from "forge-std/Script.sol";
import {console2} from "forge-std/console2.sol";
import {Upgrades} from "openzeppelin-foundry-upgrades/Upgrades.sol";

import "@xcall/contracts/xcall/CallService.sol";
import "@xcall/contracts/mocks/multi-protocol-dapp/MultiProtocolSampleDapp.sol";

contract DeployCallService is Script {
    CallService private proxyXcall;
    CallService private wrappedProxy;

    using Strings for string;

    uint256 internal deployerPrivateKey;
    address internal ownerAddress;

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

        address proxy = Upgrades.deployTransparentProxy(
            "CallService.sol",
            msg.sender,
            abi.encodeCall(CallService.initialize, "abc")
        );
        console2.log("CallService address:", proxy, "\n");

        proxyXcall = CallService(proxy);
        proxyXcall.setProtocolFee(protocolFee);
        proxyXcall.setProtocolFeeHandler(ownerAddress);
        proxyXcall.setDefaultConnection(iconNid, connection);
    }

    function upgradeContract(
        string memory chain,
        string memory contractName
    ) external broadcast(deployerPrivateKey) {
        address proxy = vm.envAddress(
            capitalizeString(chain).concat("_XCALL")
        );
        Upgrades.upgradeProxy(proxy, contractName,"");
    }
}