// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import "forge-std/Test.sol";
import "@xcall/contracts/xcall/CallService.sol";
import "@xcall/contracts/xcall/interfaces/IConnection.sol";
import "@xcall/utils/Types.sol";
import "@xcall/contracts/mocks/dapp/DAppProxySample.sol";

import "@iconfoundation/xcall-solidity-library/utils/NetworkAddress.sol";
import "@iconfoundation/xcall-solidity-library/utils/ParseAddress.sol";
import "@iconfoundation/xcall-solidity-library/utils/Integers.sol";
import "@iconfoundation/xcall-solidity-library/utils/Strings.sol";

import "@iconfoundation/xcall-solidity-library/interfaces/ICallServiceReceiver.sol";
import "@iconfoundation/xcall-solidity-library/interfaces/IDefaultCallServiceReceiver.sol";
import "@iconfoundation/xcall-solidity-library/interfaces/ICallService.sol";


contract ResponseContract {
    string public to;
    bytes public data;

    function setData(string memory _to, bytes memory _data) public {
        to = _to;
        data = _data;
    }

    function handleCallMessage(string memory _from, bytes memory _data, string[] memory protocols) public {
        console.log("handleCallMessage");
        ICallService(msg.sender).sendCall(to, data);
    }
}

contract CallServiceTest is Test {
    CallService public callService;
    DAppProxySample public dapp;
    ResponseContract public responseContract;

    IConnection public baseConnection;
    IConnection public connection1;
    IConnection public connection2;

    ICallServiceReceiver public receiver;
    IDefaultCallServiceReceiver public defaultServiceReceiver;

    using Strings for string;
    using Integers for uint;
    using ParseAddress for address;
    using ParseAddress for string;
    using NetworkAddress for string;
    using RLPEncodeStruct for Types.CSMessage;
    using RLPEncodeStruct for Types.CSMessageRequestV2;
    using RLPEncodeStruct for Types.CSMessageResult;
    using RLPEncodeStruct for Types.XCallEnvelope;
    using RLPDecodeStruct for bytes;

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

    event CallExecuted(
        uint256 indexed _reqId,
        int _code,
        string _msg
    );

    event CallMessageSent(
        address indexed _from,
        string indexed _to,
        uint256 indexed _sn
    );

    event ResponseMessage(
        uint256 indexed _sn,
        int _code
    );

    event RollbackMessage(
        uint256 indexed _sn
    );

    event RollbackExecuted(
        uint256 indexed _sn
    );

    function setUp() public {
        dapp = new DAppProxySample();
        ethDappAddress = NetworkAddress.networkAddress(ethNid, ParseAddress.toString(address(dapp)));
        (netTo, dstAccount) = NetworkAddress.parseNetworkAddress(iconDapp);

        baseConnection = IConnection(address(0x01));

        _baseSource = new string[](1);
        _baseSource[0] = ParseAddress.toString(address(baseConnection));
        _baseDestination = new string[](1);
        _baseDestination[0] = baseIconConnection;
        vm.mockCall(address(baseConnection), abi.encodeWithSelector(baseConnection.getFee.selector), abi.encode(0));

        callService = new CallService();
        callService.initialize(ethNid);

        responseContract = new ResponseContract();
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

    function testGetNetworkId() public {
        assertEq(callService.getNetworkId(), ethNid);
    }

    function testGetDefaultConnection() public {
        callService.setDefaultConnection(iconNid, address(baseConnection));

        address defaultConnection = callService.getDefaultConnection(iconNid);
        assertEq(defaultConnection, address(baseConnection));
    }

    function testGetConnectionFee() public {
        connection1 = IConnection(address(0x0000000000000000000000000000000000000011));

        callService.setDefaultConnection(iconNid, address(connection1));

        vm.mockCall(address(connection1), abi.encodeWithSelector(connection1.getFee.selector), abi.encode(30));

        uint256 fee = callService.getFee(iconNid, true);
        assertEq(fee, 30);
    }

    function testGetFeeMultipleProtocols() public {
        connection1 = IConnection(address(0x0000000000000000000000000000000000000011));
        connection2 = IConnection(address(0x0000000000000000000000000000000000000012));

        vm.mockCall(address(connection1), abi.encodeWithSelector(connection1.getFee.selector), abi.encode(10));
        vm.mockCall(address(connection2), abi.encodeWithSelector(connection2.getFee.selector), abi.encode(20));

        string[] memory sources = new string[](2);
        sources[0] = ParseAddress.toString(address(connection1));
        sources[1] = ParseAddress.toString(address(connection2));

        uint256 fee = callService.getFee(iconNid, true, sources);
        assertEq(fee, 30);
    }

    function testHandleMessageUnknownMsgType() public {
        bytes memory data = bytes("data");

        Types.CSMessage memory message = Types.CSMessage(3, data);

        vm.expectRevert("UnknownMsgType(3)");
        callService.handleMessage(iconNid, message.encodeCSMessage());
    }

    function testSendMessageSingleProtocol() public {
        bytes memory data = bytes("test");
        bytes memory rollbackData = bytes("");
        receiver = ICallServiceReceiver(address(0x02));

        vm.prank(address(dapp));
        vm.expectEmit();
        emit CallMessageSent(address(dapp), iconDapp, 1);

        Types.CSMessageRequestV2 memory request = Types.CSMessageRequestV2(ethDappAddress, dstAccount, 1, 0, data, _baseDestination);
        Types.CSMessage memory message = Types.CSMessage(Types.CS_REQUEST, request.encodeCSMessageRequestV2());

        vm.expectCall(address(baseConnection), abi.encodeCall(baseConnection.sendMessage, (iconNid, Types.NAME, 0, message.encodeCSMessage())));

        uint256 sn = callService.sendCallMessage{value: 0 ether}(iconDapp, data, rollbackData, _baseSource, _baseDestination);
        assertEq(sn, 1);
    }

    function testSendMessageMultiProtocol() public {
        bytes memory data = bytes("test");
        bytes memory rollbackData = bytes("");

        connection1 = IConnection(address(0x0000000000000000000000000000000000000011));
        connection2 = IConnection(address(0x0000000000000000000000000000000000000012));

        vm.mockCall(address(connection1), abi.encodeWithSelector(connection1.getFee.selector), abi.encode(0));
        vm.mockCall(address(connection2), abi.encodeWithSelector(connection2.getFee.selector), abi.encode(0));

        string[] memory destinations = new string[](2);
        destinations[0] = "0x1icon";
        destinations[1] = "0x2icon";

        string[] memory sources = new string[](2);
        sources[0] = ParseAddress.toString(address(connection1));
        sources[1] = ParseAddress.toString(address(connection2));

        vm.expectEmit();
        emit CallMessageSent(address(dapp), iconDapp, 1);

        Types.CSMessageRequestV2 memory request = Types.CSMessageRequestV2(ethDappAddress, dstAccount, 1, 0, data, destinations);
        Types.CSMessage memory message = Types.CSMessage(Types.CS_REQUEST,request.encodeCSMessageRequestV2());

        vm.expectCall(address(connection1), abi.encodeCall(connection1.sendMessage, (iconNid, Types.NAME, 0, message.encodeCSMessage())));
        vm.expectCall(address(connection2), abi.encodeCall(connection2.sendMessage, (iconNid, Types.NAME, 0, message.encodeCSMessage())));

        vm.prank(address(dapp));
        uint256 sn = callService.sendCallMessage{value: 0 ether}(iconDapp, data, rollbackData, sources, destinations);
        assertEq(sn, 1);
    }

    function testHandleReply() public {
        bytes memory data = bytes("test");

        callService.setDefaultConnection(iconNid, address(baseConnection));

        vm.expectEmit();
        emit CallMessageSent(address(dapp), iconDapp, 1);

        Types.CSMessageRequestV2 memory request = Types.CSMessageRequestV2(iconDapp, address(dapp).toString(), 1, Types.PERSISTENT_MESSAGE_TYPE, data, _baseSource);
        Types.CSMessageResult memory result = Types.CSMessageResult(1, Types.CS_RESP_SUCCESS,request.encodeCSMessageRequestV2());
        Types.CSMessage memory message = Types.CSMessage(Types.CS_RESULT,result.encodeCSMessageResult());

        vm.prank(address(dapp));
        uint256 sn = callService.sendCallMessage{value: 0 ether}(iconDapp, data, data, _baseSource, _baseDestination);
        assertEq(sn, 1);

        vm.expectEmit();
        emit ResponseMessage(1, Types.CS_RESP_SUCCESS);
        emit CallMessage(iconDapp, address(dapp).toString(), 1, 1, data);
        vm.prank(address(baseConnection));
        callService.handleMessage(iconNid, RLPEncodeStruct.encodeCSMessage(message));
    }

    function testHandleReplyInvalidTo() public {
        bytes memory data = bytes("test");

        callService.setDefaultConnection(iconNid, address(baseConnection));

        vm.expectEmit();
        emit CallMessageSent(address(dapp), iconDapp, 1);

        Types.CSMessageRequestV2 memory request = Types.CSMessageRequestV2("otherNid/0x1", address(dapp).toString(), 1, Types.PERSISTENT_MESSAGE_TYPE, data, _baseSource);
        Types.CSMessageResult memory result = Types.CSMessageResult(1, Types.CS_RESP_SUCCESS,request.encodeCSMessageRequestV2());
        Types.CSMessage memory message = Types.CSMessage(Types.CS_RESULT,result.encodeCSMessageResult());

        vm.prank(address(dapp));
        uint256 sn = callService.sendCallMessage{value: 0 ether}(iconDapp, data, data, _baseSource, _baseDestination);
        assertEq(sn, 1);

        vm.expectRevert("Invalid Reply");
        vm.prank(address(baseConnection));
        callService.handleMessage(iconNid, RLPEncodeStruct.encodeCSMessage(message));
    }

    function testSendMessageDefaultProtocol() public {
        bytes memory data = bytes("test");
        bytes memory rollbackData = bytes("rollback");

        callService.setDefaultConnection(iconNid, address(baseConnection));

        vm.expectEmit();
        emit CallMessageSent(address(dapp), iconDapp, 1);

        Types.CSMessageRequestV2 memory request = Types.CSMessageRequestV2(ethDappAddress, dstAccount, 1, 1, data, new string[](0));
        Types.CSMessage memory message = Types.CSMessage(Types.CS_REQUEST,request.encodeCSMessageRequestV2());
        vm.expectCall(address(baseConnection), abi.encodeCall(baseConnection.sendMessage, (iconNid, Types.NAME, 1, message.encodeCSMessage())));

        vm.prank(address(dapp));
        uint256 sn = callService.sendCallMessage{value: 0 ether}(iconDapp, data, rollbackData);
        assertEq(sn, 1);
    }

    function testSendMessagePersistent() public {
        bytes memory data = bytes("test");

        bytes memory _msg = Types.createPersistentMessage(data, new string[](0), new string[](0));

        callService.setDefaultConnection(iconNid, address(baseConnection));

        vm.expectEmit();
        emit CallMessageSent(address(dapp), iconDapp, 1);

        Types.CSMessageRequestV2 memory request = Types.CSMessageRequestV2(ethDappAddress, dstAccount, 1, Types.PERSISTENT_MESSAGE_TYPE, data, new string[](0));
        Types.CSMessage memory message = Types.CSMessage(Types.CS_REQUEST,request.encodeCSMessageRequestV2());
        vm.expectCall(address(baseConnection), abi.encodeCall(baseConnection.sendMessage, (iconNid, Types.NAME, 0, message.encodeCSMessage())));

        vm.prank(address(dapp));
        uint256 sn = callService.sendCall{value: 0 ether}(iconDapp, _msg);
        assertEq(sn, 1);
    }

    function testSendInvalidMessageType() public {
        bytes memory data = bytes("test");

        bytes memory _msg = Types.XCallEnvelope(4, data, new string[](0), new string[](0)).encodeXCallEnvelope();

        vm.expectRevert("Message type is not supported");
        vm.prank(address(dapp));
        uint256 sn = callService.sendCall{value: 0 ether}(iconDapp, _msg);
    }

    function testSendMessageResponse() public {
        bytes memory data = bytes("test");
        bytes memory data2 = bytes("test2");

        bytes memory _msg = Types.createPersistentMessage(data2, _baseSource, _baseDestination);

        callService.setDefaultConnection(iconNid, address(baseConnection));

        Types.CSMessageRequestV2 memory request = Types.CSMessageRequestV2(iconDapp, address(responseContract).toString(), 1, Types.CALL_MESSAGE_ROLLBACK_TYPE, data, _baseSource);
        Types.CSMessage memory message = Types.CSMessage(Types.CS_REQUEST,request.encodeCSMessageRequestV2());

        vm.prank(address(baseConnection));
        callService.handleMessage(iconNid, message.encodeCSMessage());

        (string memory nid, string memory iconDappAddress) = iconDapp.parseNetworkAddress();

        Types.CSMessageRequestV2 memory expectedRequest = Types.CSMessageRequestV2(NetworkAddress.networkAddress(ethNid, address(responseContract).toString()), iconDappAddress, 1, Types.PERSISTENT_MESSAGE_TYPE, data2, _baseDestination);

        responseContract.setData(iconDapp, _msg);

        Types.CSMessageResult memory result = Types.CSMessageResult(1, Types.CS_RESP_SUCCESS,expectedRequest.encodeCSMessageRequestV2());
        Types.CSMessage memory response = Types.CSMessage(Types.CS_RESULT,result.encodeCSMessageResult());

        vm.expectEmit();
        emit CallMessageSent(address(responseContract), iconDapp, 1);

        vm.expectCall(address(baseConnection), abi.encodeCall(baseConnection.sendMessage, (iconNid, Types.NAME, -1, response.encodeCSMessage())));
        callService.executeCall(1, data);
    }

    function testSendMessageResponseAnotherNetwork() public {
        bytes memory data = bytes("test");
        bytes memory data2 = bytes("test2");

        string memory bscNid = "0x61.bsc";
        string memory bscDapp = "bscaddress";

        callService.setDefaultConnection(iconNid, address(baseConnection));
        callService.setDefaultConnection(bscNid, address(baseConnection));

        bytes memory _msg = Types.createPersistentMessage(data2, _baseSource, _baseDestination);

        Types.CSMessageRequestV2 memory request = Types.CSMessageRequestV2(iconDapp, address(responseContract).toString(), 1, Types.CALL_MESSAGE_ROLLBACK_TYPE, data, _baseSource);
        Types.CSMessage memory message = Types.CSMessage(Types.CS_REQUEST,request.encodeCSMessageRequestV2());

        vm.prank(address(baseConnection));
        callService.handleMessage(iconNid, message.encodeCSMessage());

        (string memory nid, string memory iconDappAddress) = iconDapp.parseNetworkAddress();

        Types.CSMessageRequestV2 memory expectedRequest = Types.CSMessageRequestV2(NetworkAddress.networkAddress(ethNid, address(responseContract).toString()), bscDapp, 1, Types.PERSISTENT_MESSAGE_TYPE, data2, _baseDestination);

        responseContract.setData(NetworkAddress.networkAddress(bscNid, bscDapp), _msg);

        Types.CSMessageResult memory result = Types.CSMessageResult(1, Types.CS_RESP_SUCCESS,bytes(""));
        Types.CSMessage memory response = Types.CSMessage(Types.CS_RESULT,result.encodeCSMessageResult());

        Types.CSMessage memory message2 = Types.CSMessage(Types.CS_REQUEST,expectedRequest.encodeCSMessageRequestV2());

        vm.expectEmit();
        emit CallMessageSent(address(responseContract), responseContract.to(), 1);

        vm.expectCall(address(baseConnection), abi.encodeCall(baseConnection.sendMessage, (iconNid, Types.NAME, -1, response.encodeCSMessage())));
        vm.expectCall(address(baseConnection), abi.encodeCall(baseConnection.sendMessage, (bscNid, Types.NAME, 0, message2.encodeCSMessage())));
        callService.executeCall(1, data);
    }

    function testSendMessageResponseTwoWayMessage() public {

        callService.setDefaultConnection(iconNid, address(baseConnection));

        bytes memory data1 = bytes("test1");
        bytes memory data2 = bytes("test2");

        bytes memory _msg = Types.createCallMessageWithRollback(data2, data2, _baseSource, _baseDestination);

        Types.CSMessageRequestV2 memory request = Types.CSMessageRequestV2(iconDapp, address(responseContract).toString(), 1, Types.CALL_MESSAGE_ROLLBACK_TYPE, data1, _baseSource);
        Types.CSMessage memory message = Types.CSMessage(Types.CS_REQUEST,request.encodeCSMessageRequestV2());

        vm.prank(address(baseConnection));
        callService.handleMessage(iconNid, message.encodeCSMessage());

        (string memory nid, string memory iconDappAddress) = iconDapp.parseNetworkAddress();

        Types.CSMessageRequestV2 memory expectedRequest = Types.CSMessageRequestV2(NetworkAddress.networkAddress(ethNid, address(responseContract).toString()), iconDappAddress, 1, Types.CALL_MESSAGE_ROLLBACK_TYPE, data2, _baseDestination);

        responseContract.setData(iconDapp, _msg);

        Types.CSMessageResult memory result = Types.CSMessageResult(1, Types.CS_RESP_SUCCESS, bytes(""));
        Types.CSMessage memory response = Types.CSMessage(Types.CS_RESULT,result.encodeCSMessageResult());

        Types.CSMessage memory message2 = Types.CSMessage(Types.CS_REQUEST,expectedRequest.encodeCSMessageRequestV2());

        vm.expectEmit();
        emit CallMessageSent(address(responseContract), iconDapp, 1);

        vm.expectCall(address(baseConnection), abi.encodeCall(baseConnection.sendMessage, (iconNid, Types.NAME, -1, response.encodeCSMessage())));
        vm.expectCall(address(baseConnection), abi.encodeCall(baseConnection.sendMessage, (iconNid, Types.NAME, 1, message2.encodeCSMessage())));
        callService.executeCall(1, data1);
    }

    function testSendMessageDefaultProtocolNotSet() public {
        bytes memory data = bytes("test");
        bytes memory rollbackData = bytes("");

        vm.expectRevert("NoDefaultConnection");
        callService.sendCallMessage{value: 0 ether}(iconDapp, data, rollbackData);
    }

    function testHandleResponseDefaultProtocol() public {
        bytes memory data = bytes("test");

        callService.setDefaultConnection(iconNid, address(baseConnection));

        Types.CSMessageRequestV2 memory request = Types.CSMessageRequestV2(iconDapp, ParseAddress.toString(address(dapp)), 1, 1, data, new string[](0));
        Types.CSMessage memory message = Types.CSMessage(Types.CS_REQUEST,request.encodeCSMessageRequestV2());

        vm.expectEmit();
        emit CallMessage(iconDapp, ParseAddress.toString(address(dapp)), 1, 1, data);

        vm.prank(address(baseConnection));
        callService.handleMessage(iconNid, RLPEncodeStruct.encodeCSMessage(message));
    }

    function testHandleBTPMessageWithInvalidService() public {
        bytes memory data = bytes("test");

        callService.setDefaultConnection(iconNid, address(baseConnection));

        Types.CSMessageRequestV2 memory request = Types.CSMessageRequestV2(iconDapp, ParseAddress.toString(address(dapp)), 1, 1, data, new string[](0));
        Types.CSMessage memory message = Types.CSMessage(Types.CS_REQUEST,request.encodeCSMessageRequestV2());

        vm.prank(address(baseConnection));
        vm.expectRevert("InvalidServiceName");
        callService.handleBTPMessage(iconNid, "btp", 1, RLPEncodeStruct.encodeCSMessage(message));
    }

    function testHandleBTPMessage() public {
        bytes memory data = bytes("test");

        callService.setDefaultConnection(iconNid, address(baseConnection));

        Types.CSMessageRequestV2 memory request = Types.CSMessageRequestV2(iconDapp, ParseAddress.toString(address(dapp)), 1, 1, data, new string[](0));
        Types.CSMessage memory message = Types.CSMessage(Types.CS_REQUEST,request.encodeCSMessageRequestV2());

        vm.expectEmit();
        emit CallMessage(iconDapp, ParseAddress.toString(address(dapp)), 1, 1, data);

        vm.prank(address(baseConnection));
        callService.handleBTPMessage(iconNid, "xcallM", 1, RLPEncodeStruct.encodeCSMessage(message));
    }

    function testHandleBTPError() public {
        string memory data = "data"; 

        vm.expectRevert("CallRequestNotFound");
        callService.handleBTPError(iconNid, Types.NAME, 1, 1, data);
    }

    function testInvalidNid() public {
        bytes memory data = bytes("test");
        callService.setDefaultConnection(iconNid, address(baseConnection));

        Types.CSMessageRequestV2 memory request = Types.CSMessageRequestV2(iconDapp, ParseAddress.toString(address(dapp)), 1, 1, data, new string[](0));
        Types.CSMessage memory message = Types.CSMessage(Types.CS_REQUEST,request.encodeCSMessageRequestV2());

        vm.prank(address(baseConnection));
        vm.expectRevert("Invalid Network ID");
        callService.handleMessage(ethNid, RLPEncodeStruct.encodeCSMessage(message));
    }

    function testHandleResponseDefaultProtocolInvalidSender() public {
        bytes memory data = bytes("test");

        callService.setDefaultConnection(iconNid, address(baseConnection));
        Types.CSMessageRequestV2 memory request = Types.CSMessageRequestV2(iconDapp, ParseAddress.toString(address(dapp)), 1, 1, data, new string[](0));
        Types.CSMessage memory message = Types.CSMessage(Types.CS_REQUEST,request.encodeCSMessageRequestV2());

        vm.prank(address(user));
        vm.expectRevert("NotAuthorized");
        callService.handleMessage(iconNid, RLPEncodeStruct.encodeCSMessage(message));
    }

    function testHandleResponseSingleProtocol() public {
        bytes memory data = bytes("test");

        string[] memory sources = new string[](1);
        sources[0] = ParseAddress.toString(address(baseConnection));

        Types.CSMessageRequestV2 memory request = Types.CSMessageRequestV2(iconDapp, ParseAddress.toString(address(dapp)), 1, 1, data, sources);
        Types.CSMessage memory message = Types.CSMessage(Types.CS_REQUEST,request.encodeCSMessageRequestV2());
        vm.prank(address(baseConnection));

        vm.expectEmit();
        emit CallMessage(iconDapp, ParseAddress.toString(address(dapp)), 1, 1, data);

        callService.handleMessage(iconNid, RLPEncodeStruct.encodeCSMessage(message));
    }

    function testHandleResponseSingleProtocolInvalidSender() public {
        bytes memory data = bytes("test");

        string[] memory sources = new string[](1);
        sources[0] = ParseAddress.toString(address(baseConnection));

        Types.CSMessageRequestV2 memory request = Types.CSMessageRequestV2(iconDapp, ParseAddress.toString(address(dapp)), 1, 1, data, sources);
        Types.CSMessage memory message = Types.CSMessage(Types.CS_REQUEST,request.encodeCSMessageRequestV2());

        vm.prank(address(connection1));
        vm.expectRevert("NotAuthorized");

        callService.handleMessage(iconNid, RLPEncodeStruct.encodeCSMessage(message));
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

        Types.CSMessageRequestV2 memory request = Types.CSMessageRequestV2(iconDapp, ParseAddress.toString(address(dapp)), 1, 1, data, connections);
        Types.CSMessage memory message = Types.CSMessage(Types.CS_REQUEST,request.encodeCSMessageRequestV2());

        vm.prank(address(connection1));
        callService.handleMessage(iconNid, RLPEncodeStruct.encodeCSMessage(message));

        vm.expectEmit();
        emit CallMessage(iconDapp, ParseAddress.toString(address(dapp)), 1, 1, data);
        vm.prank(address(connection2));
        callService.handleMessage(iconNid, RLPEncodeStruct.encodeCSMessage(message));
    }

    function testExecuteCallSingleProtocol() public {
        bytes memory data = bytes("test");

        Types.CSMessageRequestV2 memory request = Types.CSMessageRequestV2(iconDapp, ParseAddress.toString(address(receiver)), 1, 1, data, _baseSource);
        Types.CSMessage memory message = Types.CSMessage(Types.CS_REQUEST,request.encodeCSMessageRequestV2());

        vm.prank(address(baseConnection));
        callService.handleMessage(iconNid, RLPEncodeStruct.encodeCSMessage(message));

        vm.expectEmit();
        emit CallExecuted(1, 1, "");

        vm.prank(user);
        vm.mockCall(address(receiver), abi.encodeWithSelector(receiver.handleCallMessage.selector, iconDapp, data, _baseSource), abi.encode(1));
        callService.executeCall(1, data);
    }

    function testExecuteCallUnsupportedMessageType() public {
        bytes memory data = bytes("test");

        Types.CSMessageRequestV2 memory request = Types.CSMessageRequestV2(iconDapp, ParseAddress.toString(address(receiver)), 1, 4, data, _baseSource);
        Types.CSMessage memory message = Types.CSMessage(Types.CS_REQUEST,request.encodeCSMessageRequestV2());

        vm.prank(address(baseConnection));
        callService.handleMessage(iconNid, RLPEncodeStruct.encodeCSMessage(message));

        vm.expectRevert("Message type is not yet supported");
        vm.prank(user);
        callService.executeCall(1, data);
    }

    function testExecuteCallDefaultProtocol() public {
        bytes memory data = bytes("test");

        defaultServiceReceiver = IDefaultCallServiceReceiver(address(0x5678));
        callService.setDefaultConnection(netTo, address(baseConnection));

        Types.CSMessageRequestV2 memory request = Types.CSMessageRequestV2(iconDapp, ParseAddress.toString(address(defaultServiceReceiver)), 1, 1, data, _baseSource);
        Types.CSMessage memory message = Types.CSMessage(Types.CS_REQUEST,request.encodeCSMessageRequestV2());

        vm.prank(address(baseConnection));
        callService.handleMessage(iconNid, RLPEncodeStruct.encodeCSMessage(message));

        vm.expectEmit();
        emit CallExecuted(1, 1, "");

        vm.prank(user);
        vm.mockCall(address(defaultServiceReceiver), abi.encodeWithSelector(defaultServiceReceiver.handleCallMessage.selector, iconDapp, data), abi.encode(1));
        callService.executeCall(1, data);
    }
    
    function testExecuteCallPersistent() public {
        bytes memory data = bytes("test");

        defaultServiceReceiver = IDefaultCallServiceReceiver(address(0x5678));
        callService.setDefaultConnection(netTo, address(baseConnection));

        string[] memory source;
        Types.CSMessageRequestV2 memory request = Types.CSMessageRequestV2(iconDapp, ParseAddress.toString(address(defaultServiceReceiver)), 1, Types.PERSISTENT_MESSAGE_TYPE, data, source);
        Types.CSMessage memory message = Types.CSMessage(Types.CS_REQUEST,request.encodeCSMessageRequestV2());

        vm.prank(address(baseConnection));
        callService.handleMessage(iconNid, RLPEncodeStruct.encodeCSMessage(message));        

        vm.mockCall(address(defaultServiceReceiver), abi.encodeWithSelector(defaultServiceReceiver.handleCallMessage.selector, iconDapp, data), abi.encode(1));
        vm.prank(user);
        callService.executeCall(1, data);

        vm.expectRevert("InvalidRequestId");
        vm.prank(user);
        callService.executeCall(1, data);
    }
    
    function testExecuteCallMultiProtocol() public {
        bytes memory data = bytes("test");

        defaultServiceReceiver = IDefaultCallServiceReceiver(address(0x5678));
        connection1 = IConnection(address(0x0000000000000000000000000000000000000011));
        connection2 = IConnection(address(0x0000000000000000000000000000000000000012));

        string[] memory connections = new string[](2);
        connections[0] = ParseAddress.toString(address(connection1));
        connections[1] = ParseAddress.toString(address(connection2));

        vm.mockCall(address(connection1), abi.encodeWithSelector(connection1.getFee.selector), abi.encode(0));
        vm.mockCall(address(connection2), abi.encodeWithSelector(connection2.getFee.selector), abi.encode(0));

        Types.CSMessageRequestV2 memory request = Types.CSMessageRequestV2(iconDapp, ParseAddress.toString(address(receiver)), 1, 1, data, connections);
        Types.CSMessage memory message = Types.CSMessage(Types.CS_REQUEST,request.encodeCSMessageRequestV2());

        vm.prank(address(connection1));
        callService.handleMessage(iconNid, RLPEncodeStruct.encodeCSMessage(message));

        vm.prank(address(connection2));
        callService.handleMessage(iconNid, RLPEncodeStruct.encodeCSMessage(message));

        vm.expectEmit();
        emit CallExecuted(1, 1, "");

        vm.prank(user);
        vm.mockCall(address(receiver), abi.encodeWithSelector(receiver.handleCallMessage.selector, iconDapp, data, connections), abi.encode(1));
        callService.executeCall(1, data);
    }

    function testRollBackSingleProtocol() public {
        bytes memory data = bytes("test");
        bytes memory rollbackData = bytes("rollback");

        vm.prank(address(dapp));
        vm.expectEmit();
        emit CallMessageSent(address(dapp), iconDapp, 1);

        uint256 sn = callService.sendCallMessage{value: 0 ether}(iconDapp, data, rollbackData, _baseSource, _baseDestination);
        assertEq(sn, 1);

        Types.CSMessageResult memory response = Types.CSMessageResult(1, Types.CS_RESP_FAILURE,bytes(""));
        Types.CSMessage memory message = Types.CSMessage(Types.CS_RESULT, RLPEncodeStruct.encodeCSMessageResult(response));

        vm.expectEmit();
        emit ResponseMessage(1, Types.CS_RESP_FAILURE);
        emit RollbackMessage(1);

        vm.prank(address(baseConnection));
        callService.handleMessage(iconNid, RLPEncodeStruct.encodeCSMessage(message));

        assertEq(callService.verifySuccess(sn),false);
    }

    function testRollBackDefaultProtocol() public {
        bytes memory data = bytes("test");
        bytes memory rollbackData = bytes("rollback");

        callService.setDefaultConnection(netTo, address(baseConnection));

        vm.prank(address(dapp));
        vm.expectEmit();
        emit CallMessageSent(address(dapp), iconDapp, 1);

        uint256 sn = callService.sendCallMessage{value: 0 ether}(iconDapp, data, rollbackData, _baseSource, _baseDestination);
        assertEq(sn, 1);

        Types.CSMessageResult memory response = Types.CSMessageResult(1, Types.CS_RESP_FAILURE,bytes(""));
        Types.CSMessage memory message = Types.CSMessage(Types.CS_RESULT, RLPEncodeStruct.encodeCSMessageResult(response));

        vm.expectEmit();
        emit ResponseMessage(1, Types.CS_RESP_FAILURE);

        vm.prank(address(baseConnection));
        callService.handleMessage(iconNid, RLPEncodeStruct.encodeCSMessage(message));

        assertEq(callService.verifySuccess(sn),false);
    }

    function testRollBackDefaultProtocolInvalidSender() public {
        bytes memory data = bytes("test");
        bytes memory rollbackData = bytes("rollback");

        callService.setDefaultConnection(netTo, address(baseConnection));

        vm.prank(address(dapp));
        vm.expectEmit();
        emit CallMessageSent(address(dapp), iconDapp, 1);

        uint256 sn = callService.sendCallMessage{value: 0 ether}(iconDapp, data, rollbackData, _baseSource, _baseDestination);
        assertEq(sn, 1);

        Types.CSMessageResult memory response = Types.CSMessageResult(1, Types.CS_RESP_FAILURE,bytes(""));
        Types.CSMessage memory message = Types.CSMessage(Types.CS_RESULT, RLPEncodeStruct.encodeCSMessageResult(response));

        vm.prank(address(user));
        vm.expectRevert("NotAuthorized");
        callService.handleMessage(iconNid, RLPEncodeStruct.encodeCSMessage(message));

        assertEq(callService.verifySuccess(sn),false);
    }

    function testRollbackMultiProtocol() public {
        bytes memory data = bytes("test");
        bytes memory rollbackData = bytes("rollback");

        connection1 = IConnection(address(0x0000000000000000000000000000000000000011));
        connection2 = IConnection(address(0x0000000000000000000000000000000000000012));

        vm.mockCall(address(connection1), abi.encodeWithSelector(connection1.getFee.selector), abi.encode(0));
        vm.mockCall(address(connection2), abi.encodeWithSelector(connection2.getFee.selector), abi.encode(0));

        string[] memory connections = new string[](2);
        connections[0] = ParseAddress.toString(address(connection1));
        connections[1] = ParseAddress.toString(address(connection2));

        string[] memory destinations = new string[](2);
        destinations[0] = "0x1icon";
        destinations[1] = "0x2icon";

        vm.prank(address(dapp));
        vm.expectEmit();
        emit CallMessageSent(address(dapp), iconDapp, 1);

        uint256 sn = callService.sendCallMessage{value: 0 ether}(iconDapp, data, rollbackData, connections, destinations);
        assertEq(sn, 1);

        Types.CSMessageResult memory response = Types.CSMessageResult(1, Types.CS_RESP_FAILURE,bytes(""));
        Types.CSMessage memory message = Types.CSMessage(Types.CS_RESULT, RLPEncodeStruct.encodeCSMessageResult(response));

        vm.prank(address(connection1));
        callService.handleMessage(iconNid, RLPEncodeStruct.encodeCSMessage(message));

        vm.expectEmit();
        emit ResponseMessage(1, Types.CS_RESP_FAILURE);

        vm.prank(address(connection2));
        callService.handleMessage(iconNid, RLPEncodeStruct.encodeCSMessage(message));

        assertEq(callService.verifySuccess(sn),false);
    }

    function testRollBackSuccess() public {
        bytes memory data = bytes("test");
        bytes memory rollbackData = bytes("rollback");

        vm.prank(address(dapp));
        vm.expectEmit();
        emit CallMessageSent(address(dapp), iconDapp, 1);

        uint256 sn = callService.sendCallMessage{value: 0 ether}(iconDapp, data, rollbackData, _baseSource, _baseDestination);
        assertEq(sn, 1);

        Types.CSMessageResult memory response = Types.CSMessageResult(1, Types.CS_RESP_SUCCESS,bytes(""));
        Types.CSMessage memory message = Types.CSMessage(Types.CS_RESULT, RLPEncodeStruct.encodeCSMessageResult(response));

        vm.expectEmit();
        emit ResponseMessage(1, Types.CS_RESP_SUCCESS);

        vm.prank(address(baseConnection));
        callService.handleMessage(iconNid, RLPEncodeStruct.encodeCSMessage(message));

        assertEq(callService.verifySuccess(sn),true);
    }

    function testExecuteRollBackDefaultProtocol() public {
       bytes memory data = bytes("test");
       bytes memory rollbackData = bytes("rollback");

       string memory xcallAddr = NetworkAddress.networkAddress(ethNid, ParseAddress.toString(address(callService)));

       callService.setDefaultConnection(iconNid, address(baseConnection));

       vm.startPrank(address(dapp));

       string[] memory connections = new string[](1);
       connections[0] = "";


       uint256 sn = callService.sendCallMessage{value: 0 ether}(iconDapp, data, rollbackData);
       assertEq(sn, 1);
       vm.stopPrank();

       Types.CSMessageResult memory msgRes = Types.CSMessageResult(1, Types.CS_RESP_FAILURE,bytes(""));
       Types.CSMessage memory message = Types.CSMessage(Types.CS_RESULT, msgRes.encodeCSMessageResult());

       vm.prank(address(baseConnection));
       callService.handleMessage(iconNid, message.encodeCSMessage());

       vm.expectEmit();
       emit RollbackExecuted(1);

       vm.mockCall(address(dapp), abi.encodeWithSelector(dapp.handleCallMessage.selector, xcallAddr, rollbackData), abi.encode(1));
       vm.prank(user);
       callService.executeRollback(1);

       assertEq(callService.verifySuccess(sn),false);
   }

   function testExecuteRollBackSingleProtocol() public {
       bytes memory data = bytes("test");
       bytes memory rollbackData = bytes("rollback");

       string memory xcallAddr = NetworkAddress.networkAddress(ethNid, ParseAddress.toString(address(callService)));

       vm.prank(address(dapp));
       vm.expectEmit();
       emit CallMessageSent(address(dapp), iconDapp, 1);

       uint256 sn = callService.sendCallMessage{value: 0 ether}(iconDapp, data, rollbackData, _baseSource, _baseDestination);
       assertEq(sn, 1);

       Types.CSMessageResult memory msgRes = Types.CSMessageResult(1, Types.CS_RESP_FAILURE,bytes(""));
       Types.CSMessage memory message = Types.CSMessage(Types.CS_RESULT, msgRes.encodeCSMessageResult());

       vm.prank(address(baseConnection));
       callService.handleMessage(iconNid, RLPEncodeStruct.encodeCSMessage(message));

       vm.expectEmit();
       emit RollbackExecuted(1);

       vm.mockCall(address(dapp), abi.encodeWithSelector(receiver.handleCallMessage.selector, xcallAddr, rollbackData, _baseSource), abi.encode(1));
       vm.prank(user);
       callService.executeRollback(1);

       assertEq(callService.verifySuccess(sn),false);
   }

    function testExecuteRollbackMultiProtocol() public {
        bytes memory data = bytes("test");
        bytes memory rollbackData = bytes("rollback");

        string memory xcallAddr = NetworkAddress.networkAddress(ethNid, ParseAddress.toString(address(callService)));

        connection1 = IConnection(address(0x0000000000000000000000000000000000000011));
        connection2 = IConnection(address(0x0000000000000000000000000000000000000012));

        vm.mockCall(address(connection1), abi.encodeWithSelector(connection1.getFee.selector), abi.encode(0));
        vm.mockCall(address(connection2), abi.encodeWithSelector(connection2.getFee.selector), abi.encode(0));

        string[] memory connections = new string[](2);
        connections[0] = ParseAddress.toString(address(connection1));
        connections[1] = ParseAddress.toString(address(connection2));

        string[] memory destinations = new string[](2);
        destinations[0] = "0x1icon";
        destinations[1] = "0x2icon";

        vm.prank(address(dapp));
        vm.expectEmit();
        emit CallMessageSent(address(dapp), iconDapp, 1);

        uint256 sn = callService.sendCallMessage{value: 0 ether}(iconDapp, data, rollbackData, connections, destinations);
        assertEq(sn, 1);

        Types.CSMessageResult memory response = Types.CSMessageResult(1, Types.CS_RESP_FAILURE,bytes(""));
        Types.CSMessage memory message = Types.CSMessage(Types.CS_RESULT, RLPEncodeStruct.encodeCSMessageResult(response));

        vm.prank(address(connection1));
        callService.handleMessage(iconNid, RLPEncodeStruct.encodeCSMessage(message));

        vm.expectEmit();
        emit ResponseMessage(1, Types.CS_RESP_FAILURE);
        emit RollbackMessage(1);

        vm.prank(address(connection2));
        callService.handleMessage(iconNid, RLPEncodeStruct.encodeCSMessage(message));

        vm.prank(user);
        vm.mockCall(address(dapp), abi.encodeWithSelector(receiver.handleCallMessage.selector, xcallAddr, rollbackData, connections), abi.encode(1));
        callService.executeRollback(sn);

        assertEq(callService.verifySuccess(sn),false);
    }

    function testExecuteCallMultiProtocolRollback() public {
        bytes memory data = bytes("test");

        defaultServiceReceiver = IDefaultCallServiceReceiver(address(0x5678));
        connection1 = IConnection(address(0x0000000000000000000000000000000000000011));
        connection2 = IConnection(address(0x0000000000000000000000000000000000000012));

        string[] memory connections = new string[](2);
        connections[0] = ParseAddress.toString(address(connection1));
        connections[1] = ParseAddress.toString(address(connection2));

        vm.mockCall(address(connection1), abi.encodeWithSelector(connection1.getFee.selector), abi.encode(0));
        vm.mockCall(address(connection2), abi.encodeWithSelector(connection2.getFee.selector), abi.encode(0));

        Types.CSMessageRequestV2 memory request = Types.CSMessageRequestV2(iconDapp, ParseAddress.toString(address(receiver)), 1, 1, data, connections);
        Types.CSMessage memory message = Types.CSMessage(Types.CS_REQUEST,request.encodeCSMessageRequestV2());

        vm.prank(address(connection1));
        callService.handleMessage(iconNid, RLPEncodeStruct.encodeCSMessage(message));

        vm.prank(address(connection2));
        callService.handleMessage(iconNid, RLPEncodeStruct.encodeCSMessage(message));

        vm.expectEmit();
        emit CallExecuted(1, 1, "");

        vm.prank(user);
        vm.mockCall(address(receiver), abi.encodeWithSelector(receiver.handleCallMessage.selector, iconDapp, data, connections), abi.encode(1));

        Types.CSMessageResult memory msgResponse = Types.CSMessageResult(1, Types.CS_RESP_SUCCESS,bytes(""));
        message = Types.CSMessage(Types.CS_RESULT, RLPEncodeStruct.encodeCSMessageResult(msgResponse));

        vm.expectCall(address(connection1), abi.encodeCall(connection1.sendMessage, (iconNid, Types.NAME, -1, message.encodeCSMessage())));
        vm.expectCall(address(connection2), abi.encodeCall(connection2.sendMessage, (iconNid, Types.NAME, -1, message.encodeCSMessage())));

        callService.executeCall(1, data);
    }

    function testExecuteCallDefaultProtocolRollback() public {
        bytes memory data = bytes("test");

        defaultServiceReceiver = IDefaultCallServiceReceiver(address(0x5678));
        callService.setDefaultConnection(netTo, address(baseConnection));

        Types.CSMessageRequestV2 memory request = Types.CSMessageRequestV2(iconDapp, ParseAddress.toString(address(defaultServiceReceiver)), 1, 1, data, _baseSource);
        Types.CSMessage memory message = Types.CSMessage(Types.CS_REQUEST,request.encodeCSMessageRequestV2());

        vm.prank(address(baseConnection));
        callService.handleMessage(iconNid, RLPEncodeStruct.encodeCSMessage(message));

        vm.expectEmit();
        emit CallExecuted(1, 1, "");

        vm.prank(user);
        vm.mockCall(address(defaultServiceReceiver), abi.encodeWithSelector(defaultServiceReceiver.handleCallMessage.selector, iconDapp, data), abi.encode(0));

        Types.CSMessageResult memory msgResponse = Types.CSMessageResult(1, Types.CS_RESP_SUCCESS,bytes(""));
        message = Types.CSMessage(Types.CS_RESULT, RLPEncodeStruct.encodeCSMessageResult(msgResponse));
        vm.expectCall(address(baseConnection), abi.encodeCall(baseConnection.sendMessage, (iconNid, Types.NAME, -1, message.encodeCSMessage())));
        callService.executeCall(1, data);
    }


    function testExecuteCallFailedExecution() public {
         bytes memory data = bytes("test");

        Types.CSMessageRequestV2 memory request = Types.CSMessageRequestV2(iconDapp, ParseAddress.toString(address(receiver)), 1, 1, data, _baseSource);
        Types.CSMessage memory message = Types.CSMessage(Types.CS_REQUEST,request.encodeCSMessageRequestV2());

        vm.prank(address(baseConnection));
        vm.mockCallRevert(address(baseConnection), abi.encodeWithSelector(receiver.handleCallMessage.selector, iconDapp, data, _baseSource), bytes("UserRevert"));
        callService.handleMessage(iconNid, RLPEncodeStruct.encodeCSMessage(message));

        Types.CSMessageResult memory msgResponse = Types.CSMessageResult(1, Types.CS_RESP_FAILURE, bytes(""));
        message = Types.CSMessage(Types.CS_RESULT, RLPEncodeStruct.encodeCSMessageResult(msgResponse));

        vm.expectEmit();
        emit CallExecuted(1, 0, "unknownError");

        vm.expectCall(address(baseConnection), abi.encodeCall(baseConnection.sendMessage, (iconNid, Types.NAME, -1, message.encodeCSMessage())));

        callService.executeCall(1, data);
    }

}
