// SPDX-License-Identifier: Apache-2.0
pragma solidity >=0.8.0;
pragma abicoder v2;

import "openzeppelin-contracts-upgradeable/contracts/proxy/utils/Initializable.sol";
import "@xcall/utils/Types.sol";
import "@xcall/contracts/xcall/interfaces/IConnection.sol";
import "@iconfoundation/btp2-solidity-library/interfaces/ICallService.sol";


contract CentralizedConnection is Initializable, IConnection {

    mapping(string => uint256) private messageFees;
    mapping(string => uint256) private responseFees;

    mapping(bytes32 => bool) public seenDeliveryVaaHashes;
    address private xCall;
    address private adminAddress;
    address private relayer;


    event Message(string targetNetwork,int256 sn,bytes msg);


    modifier onlyAdmin() {
        require(msg.sender == this.admin(), "OnlyAdmin");
        _;
    }

    function initialize(address _xCall, address _relayer ) public initializer {
        adminAddress = msg.sender;
        xCall = _xCall;
        relayer = _relayer;
    }

    function setRelayer(address _relayer) external onlyAdmin {
        relayer = _relayer;
    }

    function getRelayer() public view returns (address _relayer) {
        return relayer;
    }

    function setFee(
        string calldata networkId,
        uint256 messageFee,
        uint256 responseFee
    ) external onlyAdmin {
        messageFees[networkId] = messageFee;
        responseFees[networkId] = responseFee;
    }



    function getFee(string memory _to, bool _response) external view override returns (uint256 _fee) {
        uint256 messageFee = messageFees[_to];
        if (_response == true) {
            uint256 responseFee = responseFees[_to];
            return messageFee + responseFee;
        }
        return messageFee;
    }


    function sendMessage(
        string calldata _to,
        string calldata _svc,
        int256 _sn,
        bytes calldata _msg
    ) external override payable {
        require(msg.sender == xCall, "Only xCall can send messages");
        uint256 fee = this.getFee(_to,false);
        require(msg.value >= fee,"Fee is not Sufficient"); 
        emit Message(_to, _sn, _msg);

    }

    function recvMessage(
        string memory srcNID,
        string memory sn,
        bytes calldata _msg
    ) public {
        bytes32 hash = keccak256(abi.encodePacked(_msg, sn));
        require(!seenDeliveryVaaHashes[hash], "Message already processed");
        seenDeliveryVaaHashes[hash] = true;
        ICallService(xCall).handleMessage(srcNID, _msg);
    }


    /**
   * @notice Set the address of the admin.
     * @param _address The address of the admin.
     */
    function setAdmin(address _address) external onlyAdmin {
        adminAddress = _address;
    }

    /**
       @notice Gets the address of admin
       @return (Address) the address of admin
    */
    function admin(
    ) external view returns (
        address
    ) {
        return adminAddress;
    }
}