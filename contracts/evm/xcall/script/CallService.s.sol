// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import {Script, console2} from "forge-std/Script.sol";
import "../contracts/CallService.sol";

contract CallServiceScript is Script {
    function setUp() public {}

    function run() public {
        uint256 deployerPrivateKey = vm.envUint("PRIVATE_KEY");
        string memory nid = vm.envString("NID");

        vm.startBroadcast(deployerPrivateKey);
        CallService xcall = new CallService();
        xcall.initialize(nid);
        vm.stopBroadcast();
        
    }
}
