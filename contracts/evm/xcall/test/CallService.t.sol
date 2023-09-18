// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import "forge-std/Test.sol";
import "../contracts/CallService.sol";
import "../contracts/libraries/Types.sol";
import "../contracts/test/DAppProxySample.sol";

import "@iconfoundation/btp2-solidity-library/contracts/utils/NetworkAddress.sol";
import "@iconfoundation/btp2-solidity-library/contracts/utils/ParseAddress.sol";
import "@iconfoundation/btp2-solidity-library/contracts/utils/Integers.sol";
import "@iconfoundation/btp2-solidity-library/contracts/utils/Strings.sol";

import "@iconfoundation/btp2-solidity-library/contracts/interfaces/IConnection.sol";
import "@iconfoundation/btp2-solidity-library/contracts/interfaces/ICallServiceReceiver.sol";


contract CallServiceTest is Test {
    CallService public callService;
    DAppProxySample public dapp;
    IConnection public baseConnection;
    IConnection public connection1;
    IConnection public connection2;
    ICallServiceReceiver public receiver;

    address public owner = address(0x1111);
    address public user = address(0x1234);

    address public xcall;
    // address public xcallSpy;
    string public iconNid = "0x2.ICON";
    string public ethNid = "0x1.ETH";
    string public iconDapp = NetworkAddress.networkAddress(iconNid, "0xa");
    string public netTo;
    string public dstAccount;
    string public ethDappAddress;

    string public baseIconConnection = "0xb";

    string[] _baseSource;
    string[] _baseDestination;

    string constant xcallMulti = "xcall-multi";

    function setUp() public {
        dapp = new DAppProxySample();
        ethDappAddress = NetworkAddress.networkAddress(ethNid, ParseAddress.toString(address(dapp)));

        // mock when call getFee to return 0
        baseConnection = IConnection(address(0x0000000000000000000000000000000000000000));

        _baseSource = new string[](1);
        _baseSource[0] = ParseAddress.toString(address(baseConnection));
        _baseDestination = new string[](1);
        _baseDestination[0] = baseIconConnection;
        vm.mockCall(address(baseConnection), abi.encodeWithSelector(baseConnection.getFee.selector), abi.encode(0));

        callService = new CallService();
        callService.initialize(ethNid);

    }

    function testSetAdmin() public {
        callService.setAdmin(user);
        assertEq(callService.admin(), user);
    }

    function testSetAdminUnauthorized() public {
        vm.prank(user);
        vm.expectRevert("OnlyOwner");
        callService.setAdmin(user);
    }

    function testSetProtocolFees() public {
        callService.setProtocolFee(10);
        assertEq(callService.getProtocolFee(), 10);
    }

    function testSetProtocolFeesAdmin() public {
        callService.setAdmin(user);
        vm.prank(user);
        callService.setProtocolFee(10);

        assertEq(callService.getProtocolFee(), 10);
    }

    function testSetProtocolFeesUnauthorized() public {
        vm.prank(user);
        vm.expectRevert("OnlyAdmin");
        callService.setProtocolFee(10);
    }

    function testSetProtocolFeeFeeHandler() public {
        callService.setProtocolFeeHandler(user);
        assertEq(callService.getProtocolFeeHandler(), user);
    }

    function testSetProtocolFeeHandlerUnauthorized() public {
        vm.prank(user);
        vm.expectRevert("OnlyAdmin");
        callService.setProtocolFeeHandler(user);
    }

    function testSendMessage() public {
        bytes memory data = bytes("test");
        bytes memory rollbackData =bytes("");
        uint256 sn = callService.sendCallMessage{value: 0 ether}(iconDapp, data, rollbackData,_baseSource,_baseDestination);
        assertEq(sn, 1);

        Types.CSMessageRequest memory request = Types.CSMessageRequest(ethDappAddress, iconDapp, 1, false, data, _baseSource);
        Types.CSMessage memory msg = Types.CSMessage(Types.CS_REQUEST, RLPEncodeStruct.encodeCSMessageRequest(request));
        vm.mockCall(address(baseConnection), abi.encodeWithSelector(baseConnection.sendMessage.selector, msg), abi.encode(1));
    }

    function testSendMessageMultiProtocol() public {
        bytes memory data = bytes("test");
        bytes memory rollbackData =bytes("");

        connection1 = IConnection(address(0x0000000000000000000000000000000000000001));
        connection1 = IConnection(address(0x0000000000000000000000000000000000000002));

        vm.mockCall(address(connection1), abi.encodeWithSelector(connection1.getFee.selector), abi.encode(0));
        vm.mockCall(address(connection2), abi.encodeWithSelector(connection2.getFee.selector), abi.encode(0));

        string[] memory destinations = new string[](2);
        destinations[0] = "0x1icon";
        destinations[1] = "0x2icon";

        string[] memory sources = new string[](2);
        sources[0] = ParseAddress.toString(address(connection1));
        sources[1] = ParseAddress.toString(address(connection2));

        uint256 sn = callService.sendCallMessage{value: 0 ether}(iconDapp, data, rollbackData,sources,destinations);
        assertEq(sn, 1);

        Types.CSMessageRequest memory request = Types.CSMessageRequest(ethDappAddress, iconDapp, 1, false, data, _baseSource);
        Types.CSMessage memory msg = Types.CSMessage(Types.CS_REQUEST, RLPEncodeStruct.encodeCSMessageRequest(request));
        vm.mockCall(address(connection1), abi.encodeWithSelector(connection1.sendMessage.selector, msg), abi.encode(1));
        vm.mockCall(address(connection2), abi.encodeWithSelector(connection2.sendMessage.selector, msg), abi.encode(1));
    }
}
