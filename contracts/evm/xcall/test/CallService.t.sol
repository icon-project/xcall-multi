// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import "forge-std/Test.sol";
import "../contracts/CallService.sol";

import "@iconfoundation/btp2-solidity-library/contracts/utils/NetworkAddress.sol";
import "@iconfoundation/btp2-solidity-library/contracts/utils/ParseAddress.sol";
import "@iconfoundation/btp2-solidity-library/contracts/utils/Integers.sol";
import "@iconfoundation/btp2-solidity-library/contracts/utils/Strings.sol";

import "@iconfoundation/btp2-solidity-library/contracts/interfaces/IConnection.sol";
import "@iconfoundation/btp2-solidity-library/contracts/interfaces/ICallService.sol";

import "../contracts/test/DAppProxySample.sol";


contract CallServiceTest is Test {
    CallService public callService;
    DAppProxySample public dapp;
    IConnection public baseConnection;

    IConnection public connection1;
    IConnection public connection2;

    address public owner = address(0x1111);
    address public user = address(0x1234);

    address public xcall;
    string public iconNid = "0x2.ICON";
    string public ethNid = "0x1.ETH";
    string public iconDapp = NetworkAddress.networkAddress(iconNid, "0xa");

    string public netTo;
    string public dstAccount;
    string public ethDappAddress;

    string public baseIconConnection = "0xb";

    string[] _baseSource;
    string[] _baseDestination;

    event CallMessage(
        string indexed _from,
        string indexed _to,
        uint256 indexed _sn,
        uint256 _reqId,
        bytes _data
    );


    function setUp() public {
        dapp = new DAppProxySample();
        ethDappAddress = NetworkAddress.networkAddress(ethNid, ParseAddress.toString(address(dapp)));

        // mock when call getFee to return 0
        baseConnection = IConnection(address(0x1234));

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
        vm.expectRevert("OnlyAdmin");
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

    function testHandleResponseDefaultProtocol() public {
        bytes memory data = bytes("test");

        callService.setDefaultConnection(netTo, address(baseConnection));

        Types.CSMessageRequest memory request = Types.CSMessageRequest(iconDapp, ParseAddress.toString(address(dapp)), 1, false, data, new string[](0));
        Types.CSMessage memory msg = Types.CSMessage(Types.CS_REQUEST, RLPEncodeStruct.encodeCSMessageRequest(request));

        vm.expectEmit();
        emit CallMessage(iconDapp, ParseAddress.toString(address(dapp)), 1, 1, data);

        vm.prank(address(baseConnection));
        callService.handleMessage(iconNid, RLPEncodeStruct.encodeCSMessage(msg));
    }

    function testHandleResponseDefaultProtocolInvalidSender() public {
        bytes memory data = bytes("test");

        callService.setDefaultConnection(netTo, address(baseConnection));
        Types.CSMessageRequest memory request = Types.CSMessageRequest(iconDapp, ParseAddress.toString(address(dapp)), 1, false, data, new string[](0));
        Types.CSMessage memory msg = Types.CSMessage(Types.CS_REQUEST, RLPEncodeStruct.encodeCSMessageRequest(request));

        vm.prank(address(user));
        vm.expectRevert("NotAuthorized");
        callService.handleMessage(iconNid, RLPEncodeStruct.encodeCSMessage(msg));
    }

    function testHandleResponseSingleProtocol() public {
        bytes memory data = bytes("test");

        string[] memory sources = new string[](1);
        sources[0] = ParseAddress.toString(address(baseConnection));

        Types.CSMessageRequest memory request = Types.CSMessageRequest(iconDapp, ParseAddress.toString(address(dapp)), 1, false, data, sources);
        Types.CSMessage memory msg = Types.CSMessage(Types.CS_REQUEST, RLPEncodeStruct.encodeCSMessageRequest(request));
        vm.prank(address(baseConnection));

        vm.expectEmit();
        emit CallMessage(iconDapp, ParseAddress.toString(address(dapp)), 1, 1, data);

        callService.handleMessage(iconNid, RLPEncodeStruct.encodeCSMessage(msg));
    }

    function testHandleResponseSingleProtocolInvalidSender() public {
        bytes memory data = bytes("test");

        string[] memory sources = new string[](1);
        sources[0] = ParseAddress.toString(address(baseConnection));

        Types.CSMessageRequest memory request = Types.CSMessageRequest(iconDapp, ParseAddress.toString(address(dapp)), 1, false, data, sources);
        Types.CSMessage memory msg = Types.CSMessage(Types.CS_REQUEST, RLPEncodeStruct.encodeCSMessageRequest(request));

        vm.prank(address(user));
        vm.expectRevert("NotAuthorized");

        callService.handleMessage(iconNid, RLPEncodeStruct.encodeCSMessage(msg));
    }

    function testHandleResponseMultiProtocol() public {
        bytes memory data = bytes("test");

        connection1 = IConnection(address(0x0000000000000000000000000000000000000011));
        connection2 = IConnection(address(0x0000000000000000000000000000000000000012));

        vm.mockCall(address(connection1), abi.encodeWithSelector(connection1.getFee.selector), abi.encode(0));
        vm.mockCall(address(connection2), abi.encodeWithSelector(connection2.getFee.selector), abi.encode(0));

        string[] memory connections = new string[](2);
        connections[0] = ParseAddress.toString(address(connection1));
        connections[1] = ParseAddress.toString(address(connection2));

        Types.CSMessageRequest memory request = Types.CSMessageRequest(iconDapp, ParseAddress.toString(address(dapp)), 1, false, data, connections);
        Types.CSMessage memory msg = Types.CSMessage(Types.CS_REQUEST, RLPEncodeStruct.encodeCSMessageRequest(request));

        vm.prank(address(connection1));
        callService.handleMessage(iconNid, RLPEncodeStruct.encodeCSMessage(msg));

        vm.expectEmit();
        emit CallMessage(iconDapp, ParseAddress.toString(address(dapp)), 1, 1, data);
        vm.prank(address(connection2));
        callService.handleMessage(iconNid, RLPEncodeStruct.encodeCSMessage(msg));
    }
}