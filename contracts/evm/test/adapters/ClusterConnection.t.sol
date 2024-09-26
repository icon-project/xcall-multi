// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import {Test, console2 } from "forge-std/Test.sol";
import {LZEndpointMock} from "@lz-contracts/mocks/LZEndpointMock.sol";
import "@xcall/contracts/adapters/ClusterConnection.sol";
import "@xcall/contracts/xcall/CallService.sol";
import "@xcall/contracts/mocks/multi-protocol-dapp/MultiProtocolSampleDapp.sol";
import "@xcall/utils/Types.sol";

contract ClusterConnectionTest is Test {
    using RLPEncodeStruct for Types.CSMessage;
    using RLPEncodeStruct for Types.CSMessageRequestV2;

    event CallExecuted(uint256 indexed _reqId, int _code, string _msg);

    event RollbackExecuted(uint256 indexed _sn);

    event Message(string targetNetwork, int256 sn, bytes msg);

    event ResponseOnHold(uint256 indexed _sn);

    MultiProtocolSampleDapp dappSource;
    MultiProtocolSampleDapp dappTarget;

    CallService xCallSource;
    CallService xCallTarget;

    ClusterConnection adapterSource;
    ClusterConnection adapterTarget;

    address public sourceRelayer;
    address public destinationRelayer;

    string public nidSource = "icon.local";
    string public nidTarget = "evm.local";

    address public owner = address(uint160(uint256(keccak256("owner"))));
    address public admin = address(uint160(uint256(keccak256("admin"))));
    address public user = address(uint160(uint256(keccak256("user"))));    
        
    event CallMessage(
        string indexed _from,
        string indexed _to,
        uint256 indexed _sn,
        uint256 _reqId,
        bytes _data
    );

    address public source_relayer =
        address(uint160(uint256(keccak256("source_relayer"))));
    address public destination_relayer =
        address(uint160(uint256(keccak256("destination_relayer"))));

    function _setupSource() internal {
        console2.log("------>setting up source<-------");
        xCallSource = new CallService();
        xCallSource.initialize(nidSource);

        dappSource = new MultiProtocolSampleDapp();
        dappSource.initialize(address(xCallSource));

        adapterSource = new ClusterConnection();
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

        adapterTarget = new ClusterConnection();
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
        string memory to = NetworkAddress.networkAddress(
            nidTarget,
            ParseAddress.toString(address(dappTarget))
        );

        uint256 cost = adapterSource.getFee(nidTarget, false);

        bytes memory data = bytes("test");
        bytes memory rollback = bytes("");

        dappSource.sendMessage{value: cost}(to, data, rollback);
        vm.stopPrank();
    }

    function testRecvMessage() public {
        bytes memory data = bytes("test");
        string memory iconDapp = NetworkAddress.networkAddress(
            nidSource,
            "0xa"
        );
        Types.CSMessageRequestV2 memory request = Types.CSMessageRequestV2(
            iconDapp,
            ParseAddress.toString(address(dappSource)),
            1,
            Types.CALL_MESSAGE_TYPE,
            data,
            new string[](0)
        );
        Types.CSMessage memory message = Types.CSMessage(
            Types.CS_REQUEST,
            request.encodeCSMessageRequestV2()
        );

        vm.startPrank(destination_relayer);
        adapterTarget.recvMessage(
            nidSource,
            1,
            RLPEncodeStruct.encodeCSMessage(message)
        );
        vm.stopPrank();
    }

    function testRecvMessageUnAuthorized() public {
        bytes memory data = bytes("test");
        string memory iconDapp = NetworkAddress.networkAddress(
            nidSource,
            "0xa"
        );
        Types.CSMessageRequestV2 memory request = Types.CSMessageRequestV2(
            iconDapp,
            ParseAddress.toString(address(dappSource)),
            1,
            Types.CALL_MESSAGE_TYPE,
            data,
            new string[](0)
        );
        Types.CSMessage memory message = Types.CSMessage(
            Types.CS_REQUEST,
            request.encodeCSMessageRequestV2()
        );

        vm.startPrank(user);
        vm.expectRevert("OnlyRelayer");
        adapterTarget.recvMessage(
            nidSource,
            1,
            RLPEncodeStruct.encodeCSMessage(message)
        );
        vm.stopPrank();
    }

    function testRecvMessageDuplicateMsg() public {
        bytes memory data = bytes("test");
        string memory iconDapp = NetworkAddress.networkAddress(
            nidSource,
            "0xa"
        );
        Types.CSMessageRequestV2 memory request = Types.CSMessageRequestV2(
            iconDapp,
            ParseAddress.toString(address(dappSource)),
            1,
            Types.CALL_MESSAGE_TYPE,
            data,
            new string[](0)
        );
        Types.CSMessage memory message = Types.CSMessage(
            Types.CS_REQUEST,
            request.encodeCSMessageRequestV2()
        );

        vm.startPrank(destination_relayer);
        adapterTarget.recvMessage(
            nidSource,
            1,
            RLPEncodeStruct.encodeCSMessage(message)
        );

        vm.expectRevert("Duplicate Message");
        adapterTarget.recvMessage(
            nidSource,
            1,
            RLPEncodeStruct.encodeCSMessage(message)
        );
        vm.stopPrank();
    }

    function testRevertMessage() public {
        vm.startPrank(destination_relayer);
        vm.expectRevert("CallRequestNotFound");
        adapterTarget.revertMessage(1);
        vm.stopPrank();
    }

    function testRevertMessageUnauthorized() public {
        vm.startPrank(user);
        vm.expectRevert("OnlyRelayer");
        adapterTarget.revertMessage(1);
        vm.stopPrank();
    }

    function testSetFees() public {
        vm.prank(source_relayer);
        adapterSource.setFee(nidTarget, 5 ether, 5 ether);

        assertEq(adapterSource.getFee(nidTarget, true), 10 ether);
        assertEq(adapterSource.getFee(nidTarget, false), 5 ether);
    }

    function testSetFeesUnauthorized() public {
        vm.prank(user);

        vm.expectRevert("OnlyRelayer");
        adapterSource.setFee(nidTarget, 5 ether, 5 ether);
    }

    function testClaimFeesUnauthorized() public {
        vm.prank(user);

        vm.expectRevert("OnlyRelayer");
        adapterSource.claimFees();
    }

    function testClaimFees() public {
        testSetFees();
        vm.startPrank(user);
        string memory to = NetworkAddress.networkAddress(
            nidTarget,
            ParseAddress.toString(address(dappTarget))
        );

        uint256 cost = adapterSource.getFee(nidTarget, true);

        bytes memory data = bytes("test");
        bytes memory rollback = bytes("rollback");

        dappSource.sendMessage{value: cost}(to, data, rollback);
        vm.stopPrank();

        assert(address(adapterSource).balance == 10 ether);

        vm.startPrank(source_relayer);
        adapterSource.claimFees();
        vm.stopPrank();

        assert(source_relayer.balance == 10 ether);
    }

    function testGetReceipt() public {
        bytes memory data = bytes("test");
        string memory iconDapp = NetworkAddress.networkAddress(
            nidSource,
            "0xa"
        );
        Types.CSMessageRequestV2 memory request = Types.CSMessageRequestV2(
            iconDapp,
            ParseAddress.toString(address(dappSource)),
            1,
            Types.CALL_MESSAGE_TYPE,
            data,
            new string[](0)
        );
        Types.CSMessage memory message = Types.CSMessage(
            Types.CS_REQUEST,
            request.encodeCSMessageRequestV2()
        );

        assert(adapterTarget.getReceipt(nidSource, 1) == false);

        vm.startPrank(destination_relayer);
        adapterTarget.recvMessage(
            nidSource,
            1,
            RLPEncodeStruct.encodeCSMessage(message)
        );
        vm.stopPrank();

        assert(adapterTarget.getReceipt(nidSource, 1) == true);
    }    

    function testRecvMessageWithMultiSignature() public {
        bytes memory data = bytes("test");
        string memory iconDapp = NetworkAddress.networkAddress(
            nidSource,
            "0xa"
        );
        Types.CSMessageRequestV2 memory request = Types.CSMessageRequestV2(
            iconDapp,
            ParseAddress.toString(address(dappSource)),
            1,
            Types.CALL_MESSAGE_TYPE,
            data,
            new string[](0)
        );
        Types.CSMessage memory message = Types.CSMessage(
            Types.CS_REQUEST,
            request.encodeCSMessageRequestV2()
        );
        uint256 pk = hexStringToUint256("ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80");
        uint256 pk2 = hexStringToUint256("47e179ec197488593b187f80a00eb0da91f1b9d0b13f8733639f19c30a34926a");
        uint256 pk3 = hexStringToUint256("59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d");
        uint256 pk4 = hexStringToUint256("2a871d0798f97d79848a013d4936a73bf4cc922c825d33c1cf7073dff6d409c6");
        bytes32 hash = keccak256(RLPEncodeStruct.encodeCSMessage(message));
        vm.startPrank(destination_relayer);
        adapterTarget.addSigner(address(0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266));
        adapterTarget.addSigner(address(0x15d34AAf54267DB7D7c367839AAf71A00a2C6A65));
        adapterTarget.addSigner(address(0x70997970C51812dc3A010C7d01b50e0d17dc79C8));
        adapterTarget.addSigner(address(0xa0Ee7A142d267C1f36714E4a8F75612F20a79720));       
        adapterTarget.setRequiredCount(4);
        vm.expectEmit();
        emit CallMessage(iconDapp, ParseAddress.toString(address(dappSource)), 1, 1, data);
        vm.expectCall(address(xCallTarget), abi.encodeCall(xCallTarget.handleMessage, (nidSource,RLPEncodeStruct.encodeCSMessage(message))));
        bytes[] memory signatures = new bytes[](4) ;
        signatures[0] = signMessage(pk,hash);
        signatures[1] = signMessage(pk2,hash);
        signatures[2] = signMessage(pk3,hash);
        signatures[3] = signMessage(pk4,hash);
        adapterTarget.recvMessageWithSignatures(
            nidSource,
            1,
            RLPEncodeStruct.encodeCSMessage(message),
            signatures
        );
        vm.stopPrank();
    }

    function signMessage(uint256 pk,bytes32 hash) private pure returns (bytes memory){
        (uint8 v, bytes32 r, bytes32 s) = vm.sign(pk, hash);
        bytes memory signature = combineSignature(r,s,v);
        return signature;
    }
    
    function combineSignature(bytes32 r, bytes32 s, uint8 v) private pure returns (bytes memory) {
        return abi.encodePacked(r, s, v);
    }

    function hexStringToUint256(string memory hexString) public pure returns (uint256) {
        bytes memory hexBytes = bytes(hexString);
        uint256 number = 0;

        for (uint i = 0; i < hexBytes.length; i++) {
            uint8 hexDigit = uint8(hexBytes[i]);

            // Convert ASCII characters 0-9 and A-F or a-f to their numeric values
            if (hexDigit >= 48 && hexDigit <= 57) {
                number = number * 16 + (hexDigit - 48); // 0-9
            } else if (hexDigit >= 65 && hexDigit <= 70) {
                number = number * 16 + (hexDigit - 55); // A-F
            } else if (hexDigit >= 97 && hexDigit <= 102) {
                number = number * 16 + (hexDigit - 87); // a-f
            } else {
                revert("Invalid hex character");
            }
        }
        return number;
    }

    function testAddSigner() public {
        vm.startPrank(destination_relayer);
        adapterTarget.addSigner(address(0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266));
        assertEq(2, adapterTarget.listSigners().length);
        vm.stopPrank();
    }

    function testRemoveSigner() public {
        vm.startPrank(destination_relayer);
        adapterTarget.addSigner(address(0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266));
        adapterTarget.addSigner(address(0x976EA74026E726554dB657fA54763abd0C3a0aa9));
        assertEq(3, adapterTarget.listSigners().length);
        adapterTarget.removeSigner(address(0xa0Ee7A142d267C1f36714E4a8F75612F20a79720));
        assertEq(3, adapterTarget.listSigners().length);
        adapterTarget.removeSigner(address(0x976EA74026E726554dB657fA54763abd0C3a0aa9));
        assertEq(2, adapterTarget.listSigners().length);
        vm.stopPrank();
    }

    function testRequiredCount() public {
        vm.startPrank(destination_relayer);
        adapterTarget.setRequiredCount(3);
        assertEq(3, adapterTarget.getRequiredCount());
        vm.stopPrank();
    }
}
