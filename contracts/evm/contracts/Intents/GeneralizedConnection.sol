// SPDX-License-Identifier: Apache-2.0
pragma solidity >=0.8.0;
pragma abicoder v2;

contract GeneralizedConnection {
    mapping(string => mapping(uint256 => bool)) public receipts;
    address public relayAdress;
    uint256 public connSn;

    event Message(string targetNetwork, uint256 sn, bytes _msg);

    modifier onlyRelay() {
        require(msg.sender == this.admin(), "OnlyRelayer");
        _;
    }

    /**
     @notice Sends the message to a specific network.
     @param to  String ( Network Id of destination network )
     @param _msg Bytes ( serialized bytes of Service Message )
     */
    function _sendMessage(
        string memory  to,
        bytes memory _msg
    ) internal  {
        connSn++;
        emit Message(to, connSn, _msg);
    }

    /**
     @notice Sends the message to a xCall.
     @param srcNetwork  String ( Network Id )
     @param _connSn Integer ( connection message sn )
     */
    function _recvMessage(
        string memory srcNetwork,
        uint256 _connSn
    ) internal onlyRelay {
        require(!receipts[srcNetwork][_connSn], "Duplicate Message");
        receipts[srcNetwork][_connSn] = true;
    }

    /**
     @notice Gets a message receipt
     @param srcNetwork String ( Network Id )
     @param _connSn Integer ( connection message sn )
     @return boolean if is has been recived or not
     */
    function getReceipt(
        string memory srcNetwork,
        uint256 _connSn
    ) public view returns (bool) {
        return receipts[srcNetwork][_connSn];
    }

    /**
        @notice Set the address of the admin.
        @param _address The address of the admin.
     */
    function setAdmin(address _address) external onlyRelay {
        relayAdress = _address;
    }

    /**
       @notice Gets the address of admin
       @return (Address) the address of admin
    */
    function admin() external view returns (address) {
        return relayAdress;
    }
}
