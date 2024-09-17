/*
 * Copyright 2022 ICON Foundation
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

package foundation.icon.xcall;

import score.Address;
import score.BranchDB;
import score.Context;
import score.DictDB;
import score.VarDB;
import score.annotation.EventLog;
import score.annotation.External;
import score.annotation.Optional;
import score.annotation.Payable;

import java.math.BigInteger;
import java.util.Arrays;

import foundation.icon.xcall.messages.CallMessage;
import foundation.icon.xcall.messages.CallMessageWithRollback;
import foundation.icon.xcall.messages.Message;
import foundation.icon.xcall.messages.PersistentMessage;
import foundation.icon.xcall.messages.XCallEnvelope;

public class CallServiceImpl implements CallService, FeeManage {
    public static final int MAX_DATA_SIZE = 2048;
    public static final int MAX_ROLLBACK_SIZE = 1024;
    public static String NID;

    public static final String SN = "sn";
    public static final String REQUEST_ID = "reqId";
    public static final String ROLLBACKS = "requests";
    public static final String PROXY_REQUESTS = "proxyReqs";
    public static final String PENDING_REQUESTS = "pendingReqs";
    public static final String PENDING_RESPONSES = "pendingResponses";
    public static final String SUCCESSFUL_RESPONSES = "successfulResponses";
    public static final String DEFAULT_CONNECTION = "defaultConnection";
    public static final String ADMIN = "admin";
    public static final String PROTOCOL_FEE = "protocolFee";
    public static final String FEE_HANDLER = "feeHandler";

    private final VarDB<BigInteger> sn = Context.newVarDB(SN, BigInteger.class);
    private final VarDB<BigInteger> reqId = Context.newVarDB(REQUEST_ID, BigInteger.class);

    private final DictDB<BigInteger, RollbackData> rollbacks = Context.newDictDB(ROLLBACKS, RollbackData.class);
    private final DictDB<BigInteger, CSMessageRequest> proxyReqs = Context.newDictDB(PROXY_REQUESTS,
            CSMessageRequest.class);
    private final BranchDB<byte[], DictDB<String, Boolean>> pendingReqs = Context.newBranchDB(PENDING_REQUESTS,
            Boolean.class);
    private final BranchDB<byte[], DictDB<String, Boolean>> pendingResponses = Context.newBranchDB(PENDING_RESPONSES,
            Boolean.class);
    private final DictDB<BigInteger, Boolean> successfulResponses = Context.newDictDB(SUCCESSFUL_RESPONSES,
            Boolean.class);

    private final DictDB<String, Address> defaultConnection = Context.newDictDB(DEFAULT_CONNECTION, Address.class);

    // for fee-related operations
    private final VarDB<Address> admin = Context.newVarDB(ADMIN, Address.class);
    private final VarDB<BigInteger> protocolFee = Context.newVarDB(PROTOCOL_FEE, BigInteger.class);
    private final VarDB<Address> feeHandler = Context.newVarDB(FEE_HANDLER, Address.class);

    private static CSMessageRequest replyState = null;
    private static byte[] callReply = null;

    public CallServiceImpl(String networkId) {
        NID = networkId;
        if (admin.get() == null) {
            admin.set(Context.getCaller());
            feeHandler.set(Context.getCaller());
        }
    }

    /* Implementation-specific external */
    @External(readonly = true)
    public String getNetworkAddress() {
        return new NetworkAddress(NID, Context.getAddress()).toString();
    }

    @External(readonly = true)
    public String getNetworkId() {
        return NID;
    }

    private void checkCallerOrThrow(Address caller, String errMsg) {
        Context.require(Context.getCaller().equals(caller), errMsg);
    }

    private BigInteger getNextSn() {
        BigInteger _sn = this.sn.getOrDefault(BigInteger.ZERO);
        _sn = _sn.add(BigInteger.ONE);
        this.sn.set(_sn);
        return _sn;
    }

    private BigInteger getNextReqId() {
        BigInteger _reqId = this.reqId.getOrDefault(BigInteger.ZERO);
        _reqId = _reqId.add(BigInteger.ONE);
        this.reqId.set(_reqId);
        return _reqId;
    }

    private void cleanupCallRequest(BigInteger sn) {
        rollbacks.set(sn, null);
    }

    @Payable
    @External
    public BigInteger sendCall(String _to, byte[] _data) {
        Address caller = Context.getCaller();
        XCallEnvelope envelope = XCallEnvelope.fromBytes(_data);
        BigInteger sn = getNextSn();
        NetworkAddress dst = NetworkAddress.valueOf(_to);

        ProcessResult result = preProcessMessage(sn, dst, envelope);

        String from = new NetworkAddress(NID, caller.toString()).toString();
        CSMessageRequest msgReq = new CSMessageRequest(from, dst.account(), sn, envelope.getType(), result.data,
                envelope.getDestinations());

        byte[] msgBytes = msgReq.toBytes();
        Context.require(msgBytes.length <= MAX_DATA_SIZE, "MaxDataSizeExceeded");
        if (isReply(dst.net, envelope.sources) && !result.needResponse) {
            replyState = null;
            callReply = msgBytes;
        } else {
            BigInteger sendSn = result.needResponse ? sn : BigInteger.ZERO;
            sendMessage(envelope.getSources(), dst.net(), CSMessage.REQUEST, sendSn, msgBytes);
            claimProtocolFee();
        }

        CallMessageSent(caller, dst.toString(), sn);

        return sn;
    }

    @Payable
    @External
    public BigInteger sendCallMessage(String _to,
            byte[] _data,
            @Optional byte[] _rollback,
            @Optional String[] _sources,
            @Optional String[] _destinations) {

        if (_sources == null || _destinations == null) {
            _sources = new String[0];
            _destinations = new String[0];
        }

        Message msg;
        if (_rollback == null || _rollback.length == 0) {
            msg = new CallMessage(_data);
        } else {
            msg = new CallMessageWithRollback(_data, _rollback);
        }

        XCallEnvelope envelope = new XCallEnvelope(msg, _sources, _destinations);
        return sendCall(_to, envelope.toBytes());
    }

    @Override
    @External
    public void executeCall(BigInteger _reqId, byte[] _data) {
        CSMessageRequest req = proxyReqs.get(_reqId);
        Context.require(req != null, "InvalidRequestId");
        // cleanup
        proxyReqs.set(_reqId, null);
        // compare the given data hash with the saved one
        Context.require(Arrays.equals(getDataHash(_data), req.getData()), "DataHashMismatch");
        executeMessage(_reqId, req, _data);
    }

    @Override
    @External
    public void executeRollback(BigInteger _sn) {
        RollbackData req = rollbacks.get(_sn);
        Context.require(req != null, "InvalidSerialNum");
        Context.require(req.enabled(), "RollbackNotEnabled");
        cleanupCallRequest(_sn);
        String[] protocols = req.getProtocols();
        if (protocols.length == 0) {
            Context.call(req.getFrom(), "handleCallMessage", getNetworkAddress(), req.getRollback());
        } else {
            Context.call(req.getFrom(), "handleCallMessage", getNetworkAddress(), req.getRollback(), protocols);
        }

        RollbackExecuted(_sn);
    }

    @External(readonly = true)
    public boolean verifySuccess(BigInteger _sn) {
        return successfulResponses.getOrDefault(_sn, false);
    }

    /* ========== Interfaces with BMC ========== */
    @External
    public void handleBTPMessage(String _from, String _svc, BigInteger _sn, byte[] _msg) {
        handleMessage(_from, _msg);
    }

    @External
    public void handleBTPError(String _src, String _svc, BigInteger _sn, long _code, String _msg) {
        handleError(_sn);
    }
    /* ========================================= */

    @Override
    @External
    public void handleMessage(String _fromNid, byte[] _msg) {
        CSMessage msg = CSMessage.fromBytes(_msg);
        Context.require(!_fromNid.equals(NID), "Invalid network ID");
        switch (msg.getType()) {
            case CSMessage.REQUEST:
                handleRequest(_fromNid, msg.getData());
                break;
            case CSMessage.RESULT:
                handleResult(msg.getData());
                break;
            default:
                Context.revert("UnknownMsgType(" + msg.getType() + ")");
        }
    }

    @Override
    @External
    public void handleError(BigInteger _sn) {
        CSMessageResult res = new CSMessageResult(_sn, CSMessageResult.FAILURE, null);
        handleResult(res.toBytes());
    }

    @External(readonly = true)
    public Address admin() {
        return admin.get();
    }

    @External
    public void setAdmin(Address _address) {
        checkCallerOrThrow(admin(), "OnlyAdmin");
        admin.set(_address);
    }

    @External
    public void setProtocolFee(BigInteger _protocolFee) {
        checkCallerOrThrow(admin(), "OnlyAdmin");
        Context.require(_protocolFee.signum() >= 0, "ValueShouldBePositive");
        protocolFee.set(_protocolFee);
    }

    @External
    public void setProtocolFeeHandler(Address _address) {
        checkCallerOrThrow(admin(), "OnlyAdmin");
        feeHandler.set(_address);
    }

    @External
    public void setDefaultConnection(String _nid, Address _connection) {
        checkCallerOrThrow(admin(), "OnlyAdmin");
        defaultConnection.set(_nid, _connection);
    }

    @External(readonly = true)
    public Address getDefaultConnection(String _nid) {
        return defaultConnection.get(_nid);
    }

    @External(readonly = true)
    public BigInteger getProtocolFee() {
        return protocolFee.getOrDefault(BigInteger.ZERO);
    }

    @External(readonly = true)
    public BigInteger getFee(String _net, boolean _rollback, @Optional String[] _sources) {
        BigInteger fee = getProtocolFee();
        if (_sources == null) {
            _sources = new String[] {};
        }

        if (isReply(_net, _sources) && !_rollback) {
            return BigInteger.ZERO;
        }
        if (_sources == null || _sources.length == 0) {
            Address src = defaultConnection.get(_net);
            Context.require(src != null, "NoDefaultConnection");
            return fee.add(Context.call(BigInteger.class, src, "getFee", _net, _rollback));
        }

        for (String protocol : _sources) {
            Address address = Address.fromString(protocol);
            fee = fee.add(Context.call(BigInteger.class, address, "getFee", _net, _rollback));
        }

        return fee;
    }

    @Override
    @EventLog(indexed = 3)
    public void CallMessage(String _from, String _to, BigInteger _sn, BigInteger _reqId, byte[] _data) {
    }

    @Override
    @EventLog(indexed = 1)
    public void CallExecuted(BigInteger _reqId, int _code, String _msg) {
    }

    @Override
    @EventLog(indexed = 1)
    public void ResponseMessage(BigInteger _sn, int _code) {
    }

    @Override
    @EventLog(indexed = 1)
    public void RollbackMessage(BigInteger _sn) {
    }

    @Override
    @EventLog(indexed = 1)
    public void RollbackExecuted(BigInteger _sn) {
    }

    @Override
    @EventLog(indexed = 3)
    public void CallMessageSent(Address _from, String _to, BigInteger _sn) {
    }

    private void sendMessage(String[] _sources, String netTo, int msgType, BigInteger sn, byte[] data) {
        Address[] sources = prepareProtocols(_sources, netTo);
        CSMessage msg = new CSMessage(msgType, data);
        BigInteger value;
        for (Address src : sources) {
            value = _getFee(src, netTo, sn);
            Context.call(value, src, "sendMessage", netTo, NAME, sn, msg.toBytes());
        }
    }

    private BigInteger _getFee(Address conn, String net, BigInteger sn) {
        if (sn.signum() == -1) {
            return BigInteger.ZERO;
        }

        return Context.call(BigInteger.class, conn, "getFee", net, sn.signum() == 1);
    }

    private int tryExecuteCall(BigInteger id, Address dapp, String from, byte[] data, String[] protocols) {
        try {
            _executeCall(id, dapp, from, data, protocols);
        } catch (Exception e) {
            CallExecuted(id, CSMessageResult.FAILURE, e.toString());
            return CSMessageResult.FAILURE;
        }
        ;

        return CSMessageResult.SUCCESS;
    }

    private void _executeCall(BigInteger id, Address dapp, String from, byte[] data, String[] protocols) {
        if (protocols.length == 0) {
            Context.call(dapp, "handleCallMessage", from, data);
        } else {
            Context.call(dapp, "handleCallMessage", from, data, protocols);
        }
        CallExecuted(id, CSMessageResult.SUCCESS, "");
    }

    private Address[] prepareProtocols(String[] protocols, String toNid) {
        if (protocols.length == 0) {
            Address src = defaultConnection.get(toNid);
            Context.require(src != null, "NoDefaultConnection");
            return new Address[] { src };
        }

        Address[] _protocols = new Address[protocols.length];
        for (int i = 0; i < protocols.length; i++) {
            _protocols[i] = Address.fromString(protocols[i]);
        }

        return _protocols;
    }

    private class ProcessResult {
        public boolean needResponse;
        public byte[] data;

        public ProcessResult(boolean needResponse, byte[] data) {
            this.needResponse = needResponse;
            this.data = data;
        }
    }

    private ProcessResult preProcessMessage(BigInteger sn, NetworkAddress to, XCallEnvelope envelope) {
        switch (envelope.getType()) {
            case CallMessage.TYPE:
            case PersistentMessage.TYPE:
                return new ProcessResult(false, envelope.getMessage());
            case CallMessageWithRollback.TYPE:
                Address caller = Context.getCaller();
                CallMessageWithRollback msg = CallMessageWithRollback.fromBytes(envelope.getMessage());
                Context.require(caller.isContract(), "RollbackNotPossible");
                RollbackData req = new RollbackData(caller, to.net(), envelope.getSources(), msg.getRollback());
                rollbacks.set(sn, req);
                return new ProcessResult(true, msg.getData());
        }

        Context.revert("Message type is not supported");
        return null;
    }

    private void executeMessage(BigInteger reqId, CSMessageRequest req, byte[] data) {
        Address to = Address.fromString(req.getTo());
        String[] protocols = req.getProtocols();
        switch (req.getType()) {
            case CallMessage.TYPE:
                tryExecuteCall(reqId, to, req.getFrom(), data, protocols);
                break;
            case PersistentMessage.TYPE:
                _executeCall(reqId, to, req.getFrom(), data, protocols);
                break;
            case CallMessageWithRollback.TYPE: {
                replyState = req;
                int code = tryExecuteCall(reqId, to, req.getFrom(), data, protocols);
                replyState = null;
                BigInteger sn = req.getSn().negate();
                NetworkAddress from = NetworkAddress.valueOf(req.getFrom());
                byte[] message = null;
                if (callReply != null && code == CSMessageResult.SUCCESS) {
                    message = callReply;
                    callReply = null;
                }

                CSMessageResult response = new CSMessageResult(req.getSn(), code, message);
                sendMessage(protocols, from.net(), CSMessage.RESULT, sn, response.toBytes());
                break;
            }
            default:
                Context.revert("Message type is not yet supported");
        }
    }

    private void claimProtocolFee() {
        BigInteger protocolFee = getProtocolFee();
        BigInteger balance = Context.getBalance(Context.getAddress());
        Context.require(balance.compareTo(protocolFee) >= 0, "InsufficientBalance");
        Context.transfer(feeHandler.get(), balance);
    }

    private void handleRequest(String netFrom, byte[] data) {
        CSMessageRequest msgReq = CSMessageRequest.fromBytes(data);
        String from = msgReq.getFrom();
        Context.require(NetworkAddress.valueOf(from).net().equals(netFrom));

        byte[] hash = Context.hash("sha-256", data);
        DictDB<String, Boolean> pending = pendingReqs.at(hash);
        if (!verifyProtocols(netFrom, msgReq.getProtocols(), pending)) {
            return;
        }

        BigInteger reqId = getNextReqId();

        // emit event to notify the user
        CallMessage(from, msgReq.getTo(), msgReq.getSn(), reqId, msgReq.getData());

        msgReq.hashData();
        proxyReqs.set(reqId, msgReq);
    }

    private void handleReply(RollbackData rollback, CSMessageRequest reply) {
        Context.require(rollback.getTo().equals(NetworkAddress.valueOf(reply.getFrom()).net), "Invalid Reply");
        reply.setProtocols(rollback.getProtocols());

        BigInteger reqId = getNextReqId();
        CallMessage(reply.getFrom(), reply.getTo(), reply.getSn(), reqId, reply.getData());
        reply.hashData();
        proxyReqs.set(reqId, reply);
    }

    private void handleResult(byte[] data) {
        CSMessageResult msgRes = CSMessageResult.fromBytes(data);
        BigInteger resSn = msgRes.getSn();
        RollbackData rollback = rollbacks.get(resSn);

        Context.require(rollback != null, "CallRequest Not Found For " + resSn.toString());

        byte[] hash = Context.hash("sha-256", data);
        DictDB<String, Boolean> pending = pendingResponses.at(hash);
        if (!verifyProtocols(rollback.getTo(), rollback.getProtocols(), pending)) {
            return;
        }

        ResponseMessage(resSn, msgRes.getCode());
        switch (msgRes.getCode()) {
            case CSMessageResult.SUCCESS:
                cleanupCallRequest(resSn);
                if (msgRes.getMsg() != null && msgRes.getMsg().length > 0) {
                    handleReply(rollback, CSMessageRequest.fromBytes(msgRes.getMsg()));
                }
                successfulResponses.set(resSn, true);
                break;
            case CSMessageResult.FAILURE:
            default:
                // emit rollback event
                Context.require(rollback.getRollback() != null, "NoRollbackData");
                rollback.setEnabled();
                rollbacks.set(resSn, rollback);
                RollbackMessage(resSn);
        }
    }

    private boolean verifyProtocols(String fromNid, String[] protocols, DictDB<String, Boolean> pendingDb) {
        Address caller = Context.getCaller();
        if (protocols.length == 0) {
            Context.require(caller.equals(defaultConnection.get(fromNid)), "ProtocolSourceMismatch");
            return true;
        } else if (protocols.length == 1) {
            Context.require(caller.toString().equals(protocols[0]), "ProtocolSourceMismatch");
            return true;
        }


        pendingDb.set(caller.toString(), true);
        for (String protocol : protocols) {
            if (!pendingDb.getOrDefault(protocol, false)) {
                return false;
            }
        }

        for (String protocol : protocols) {
            pendingDb.set(protocol, null);
        }


        return true;
    }
    private boolean isReply(String netId, String[] sources) {
        if (replyState != null) {
            return NetworkAddress.valueOf(replyState.getFrom()).net.equals(netId)
                    && protocolEquals(replyState.getProtocols(), sources);
        }

        return false;
    }

    private boolean protocolEquals(String[] a, String[] b) {
        for (int i = 0; i < b.length; i++) {
            if (!a[i].equals(b[i])) {
                return false;
            }
        }

        return true;
    }

    private byte[] getDataHash(byte[] data) {
        return Context.hash("keccak-256", data);
    }
}
