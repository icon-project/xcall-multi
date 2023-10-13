// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import "wormhole-solidity-sdk/testing/WormholeRelayerTest.sol";
import "@xcall/contracts/adapters/wormhole/WormholeAdapter.sol";
import "@xcall/contracts/xcall/CallService.sol";
import "@xcall/contracts/mocks/multi-protocol-dapp/MultiProtocolSampleDapp.sol";

contract WormholeAdapterTest is WormholeRelayerBasicTest {

    event CallExecuted(
        uint256 indexed _reqId,
        int _code,
        string _msg
    );

    MultiProtocolSampleDapp dappSource;
    MultiProtocolSampleDapp dappTarget;

    CallService xCallSource;
    CallService xCallTarget;

    WormholeAdapter adapterSource;
    WormholeAdapter adapterTarget;

    string public nidSource = "nid.source";
    string public nidTarget = "nid.target";


    function setUpSource() public override {
        console2.log("------>setting up source<-------");
        xCallSource = new CallService();
        xCallSource.initialize(nidSource);

        dappSource = new MultiProtocolSampleDapp();
        dappSource.initialize(address(xCallSource));

        adapterSource = new WormholeAdapter();
        adapterSource.initialize(address(relayerSource), address(xCallSource));

        xCallSource.setDefaultConnection(nidTarget, address(adapterSource));
    }

    function setUpTarget() public override {
        console2.log("------>setting up target<-------");

        xCallTarget = new CallService();
        xCallTarget.initialize(nidTarget);

        dappTarget = new MultiProtocolSampleDapp();
        dappTarget.initialize(address(xCallTarget));

        adapterTarget = new WormholeAdapter();
        adapterTarget.initialize(address(relayerTarget), address(xCallTarget));

        xCallTarget.setDefaultConnection(nidSource, address(adapterTarget));
        toWormholeFormat(address(xCallTarget));
    }

    function setUpGeneral() public override {
        console2.log("------>setting up connections<-------");

        string memory adapterSourceAdr = ParseAddress.toString(
            address(adapterSource)
        );
        string memory adapterTargetAdr = ParseAddress.toString(
            address(adapterTarget)
        );


        dappSource.addConnection(nidTarget, adapterSourceAdr, adapterTargetAdr);

        adapterSource.configureConnection(
            nidTarget,
            targetChain,
            toWormholeFormat(address(adapterTarget)),
            5_000_000
        );
        vm.selectFork(targetFork);
        dappTarget.addConnection(nidSource, adapterTargetAdr, adapterSourceAdr);

        adapterTarget.configureConnection(
            nidSource,
            sourceChain,
            toWormholeFormat(address(adapterSource)),
            5_000_000
        );
    }


    function testSendMessage() public {
        vm.recordLogs();
        vm.selectFork(sourceFork);

        string memory to = NetworkAddress.networkAddress(nidTarget, ParseAddress.toString(address(dappTarget)));

        uint256 cost = adapterSource.getFee(nidTarget, false);

        bytes memory data = bytes("test");
        bytes memory rollback = bytes("");
        dappSource.sendMessage{value: cost}(to, data, rollback);

        performDelivery();

        vm.selectFork(targetFork);
        vm.expectEmit();
        emit CallExecuted(1, 1, "");
        xCallTarget.executeCall(1, data);

    }

}
