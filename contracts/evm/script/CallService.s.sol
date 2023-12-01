// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import {Script, console2} from "forge-std/Script.sol";
import "@xcall/contracts/xcall/CallService.sol";

//forge create --rpc-url http://127.0.0.1:8545 --private-key 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80 <path>:XCallCentralizeConnection

contract CallServiceScript is Script {
    function setUp() public {}

    function run() public {
        uint256 deployerPrivateKey = vm.envUint("PRIVATE_KEY");
        string memory nid = vm.envString("BSC_NID");
        string memory iconNid = vm.envString("ICON_NID");
        address connection = vm.envAddress("BMC_ADDRESS");

        vm.startBroadcast(deployerPrivateKey);
        CallService xcall = new CallService();
        xcall.initialize(nid);

        xcall.setProtocolFee(vm.envUint("PROTOCOL_FEE"));
        xcall.setProtocolFeeHandler(vm.envAddress("OWNER_ADDRESS"));

        xcall.setDefaultConnection(iconNid, connection);
        vm.stopBroadcast();

    }
}
