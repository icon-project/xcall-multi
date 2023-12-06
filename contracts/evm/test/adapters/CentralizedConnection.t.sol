// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import {Test, console2} from "forge-std/Test.sol";
import {LZEndpointMock} from "@lz-contracts/mocks/LZEndpointMock.sol";
import "@xcall/contracts/adapters/CentralizedConnection.sol";
import "@xcall/contracts/xcall/CallService.sol";
import "@xcall/contracts/mocks/multi-protocol-dapp/MultiProtocolSampleDapp.sol";


contract CentralizedConnectionTest is Test {

    event CallExecuted(
        uint256 indexed _reqId,
        int _code,
        string _msg
    );

    event RollbackExecuted(
        uint256 indexed _sn
    );

    event Message(string targetNetwork,int256 sn,bytes msg);


    event ResponseOnHold(uint256 indexed _sn);

    MultiProtocolSampleDapp dappSource;
    MultiProtocolSampleDapp dappTarget;

    CallService xCallSource;
    CallService xCallTarget;

    CentralizedConnection adapterSource;
    CentralizedConnection adapterTarget;

    address public sourceRelayer;
    address public destinationRelayer;

    string public nidSource = "nid.source";
    string public nidTarget = "nid.target";

    address public owner = address(uint160(uint256(keccak256("owner"))));
    address public admin = address(uint160(uint256(keccak256("admin"))));
    address public user = address(uint160(uint256(keccak256("user"))));

    address public source_relayer = address(uint160(uint256(keccak256("source_relayer"))));
    address public destination_relayer= address(uint160(uint256(keccak256("destination_relayer"))));

    function _setupSource() internal {
        console2.log("------>setting up source<-------");
        xCallSource = new CallService();
        xCallSource.initialize(nidSource);

        dappSource = new MultiProtocolSampleDapp();
        dappSource.initialize(address(xCallSource));

        adapterSource = new CentralizedConnection();
        adapterSource.initialize(source_relayer, address(xCallSource));

        xCallSource.setDefaultConnection(nidTarget, address(adapterSource));

        console2.log(ParseAddress.toString(address(xCallSource)));
        console2.log(ParseAddress.toString(address(user)));
    }

    function _setupTarget() internal {
        console2.log("------>setting up target<-------");

        xCallTarget = new CallService();
        xCallTarget.initialize(nidTarget);

        dappTarget = new MultiProtocolSampleDapp();
        dappTarget.initialize(address(xCallTarget));

        adapterTarget = new CentralizedConnection();
        adapterTarget.initialize(destination_relayer, address(xCallTarget));

        xCallTarget.setDefaultConnection(nidSource, address(adapterTarget));
    }

    /**
     * @dev Sets up the initial state for the test.
     */
    function setUp() public {
        vm.startPrank(owner);
        _setupSource();
        _setupTarget();

        vm.stopPrank();

        // deal some gas
        vm.deal(admin, 10 ether);
        vm.deal(user, 10 ether);
    }


    function testSetAdmin() public {
        vm.prank(source_relayer);
        adapterSource.setAdmin(user);
        assertEq(adapterSource.admin(), user);
    }

    function testSetAdminUnauthorized() public {
        vm.prank(user);
        vm.expectRevert("OnlyRelayer");
        adapterSource.setAdmin(user);
    }

    function testSendMessage() public {

        vm.startPrank(user);
        string memory to = NetworkAddress.networkAddress(nidTarget, ParseAddress.toString(address(dappTarget)));

        uint256 cost = adapterSource.getFee(nidTarget, false);

        bytes memory data = bytes("test");
        bytes memory rollback = bytes("");

        dappSource.sendMessage{value: cost}(to, data, rollback);
        vm.stopPrank();
    }

}