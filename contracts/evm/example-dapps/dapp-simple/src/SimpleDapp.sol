// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.0;
import "@iconfoundation/btp2-solidity-library/interfaces/IDefaultCallServiceReceiver.sol";
import "@iconfoundation/btp2-solidity-library/interfaces/ICallService.sol";
import "@iconfoundation/btp2-solidity-library/utils/ParseAddress.sol";

import "@openzeppelin/contracts-upgradeable/proxy/utils/Initializable.sol";

import "../lib/RollBack.sol";

contract SimpleDapp is IDefaultCallServiceReceiver, Initializable {
    using ParseAddress for string;

    address public callSvc;
    uint256 public id;
    mapping(uint256 => Rollback.RollbackData) public rollbacks;

    modifier onlyCallService() {
        require(msg.sender == callSvc, "onlyCallService");
        _;
    }

    function initialize(address _callService) public {
        callSvc = _callService;
    }

    function getNextId() internal returns (uint256) {
        id++;
        return id;
    }

    function sendMessage(
        string memory _to,
        bytes memory _data,
        bytes memory rollback
    ) external payable {
        if (rollback.length > 0) {
            uint256 newId = getNextId();
            Rollback.RollbackData memory rbData = Rollback.RollbackData(
                newId,
                rollback,
                0
            );
            uint256 ssn = _sendCallMessage(msg.value, _to, _data, rollback);
            rbData.ssn = ssn;
            rollbacks[newId] = rbData;
        } else {
            _sendCallMessage(msg.value, _to, _data, "");
        }
    }

    function _sendCallMessage(
        uint256 value,
        string memory to,
        bytes memory data,
        bytes memory rollback
    ) internal returns (uint256) {
        try
            ICallService(callSvc).sendCallMessage{value: value}(
                to,
                data,
                rollback
            )
        returns (uint256 result) {
            return result;
        } catch (bytes memory e) {
            revert(string(e));
        }
    }

    function handleCallMessage(
        string memory _from,
        bytes memory _data
    ) external onlyCallService {
        if (address(this) == _from._toAddress()) {
            Rollback.RollbackData memory received = Rollback.decodeRollbackData(
                _data
            );
            Rollback.RollbackData storage stored = rollbacks[received.id];
            require(stored.id == received.id, "invalid received id");
            require(
                keccak256(stored.rollback) == keccak256(received.rollback),
                "rollbackData mismatch"
            );
            delete rollbacks[received.id];
            emit RollbackDataReceived(_from, stored.ssn, received.rollback);
        } else {
            emit MessageReceived(_from, _data);
        }
    }

    event MessageReceived(string from, bytes data);
    event RollbackDataReceived(string from, uint256 ssn, bytes rollback);
}
