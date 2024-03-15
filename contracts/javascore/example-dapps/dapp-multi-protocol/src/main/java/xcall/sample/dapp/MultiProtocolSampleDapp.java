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

package xcall.sample.dapp;

import java.math.BigInteger;

import score.Address;
import score.ArrayDB;
import score.BranchDB;
import score.Context;
import score.annotation.EventLog;
import score.annotation.External;
import score.annotation.Optional;
import score.annotation.Payable;
import foundation.icon.xcall.CallServiceReceiver;
import foundation.icon.xcall.NetworkAddress;
import foundation.icon.xcall.messages.Message;
import foundation.icon.xcall.messages.CallMessage;
import foundation.icon.xcall.messages.CallMessageWithRollback;
import foundation.icon.xcall.messages.XCallEnvelope;
import foundation.icon.xcall.messages.PersistentMessage;

public class MultiProtocolSampleDapp implements CallServiceReceiver {
    private final Address callSvc;
    private final BranchDB<String, ArrayDB<String>> sources = Context.newBranchDB("source", String.class);
    private final BranchDB<String, ArrayDB<String>> destinations = Context.newBranchDB("destination", String.class);

    public MultiProtocolSampleDapp(Address _callService) {
        this.callSvc = _callService;
    }

    private void onlyCallService() {
        Context.require(Context.getCaller().equals(this.callSvc), "onlyCallService");
    }

    @External
    public void addConnection(String nid, String source, String destination) {
        this.sources.at(nid).add(source);
        this.destinations.at(nid).add(destination);
    }

    @External(readonly = true)
    public String[] getSources(String nid) {
        return toArray(this.sources.at(nid));
    }

    @External(readonly = true)
    public String[] getDestinations(String nid) {
        return toArray(this.destinations.at(nid));
    }

    public String[] toArray(ArrayDB<String> db) {
        int size = db.size();
        String[] arr = new String[size];
        for (int i = 0; i < size; i++) {
            arr[i] = db.get(i);
        }

        return arr;
    }

    @Payable
    @External
    public void sendNewMessage(String _to, byte[] _data, int messageType, @Optional byte[] _rollback) {
        String net = NetworkAddress.valueOf(_to).net();

        Message msg;
        XCallEnvelope envelope;
        if (messageType == PersistentMessage.TYPE) {
            msg = new PersistentMessage(_data);
            envelope = new XCallEnvelope(msg, getSources(net), getDestinations(net));
            _sendCall(Context.getValue(), _to, envelope.toBytes());
        } else if (messageType == CallMessage.TYPE) {
            msg = new CallMessage(_data);
            envelope = new XCallEnvelope(msg, getSources(net), getDestinations(net));
            _sendCall(Context.getValue(), _to, envelope.toBytes());
        } else if (messageType == CallMessageWithRollback.TYPE) {
            msg = new CallMessageWithRollback(_data, _rollback);
            envelope = new XCallEnvelope(msg, getSources(net), getDestinations(net));
            _sendCall(Context.getValue(), _to, envelope.toBytes());
        } else {
            Context.revert("invalid message type");
        }
    }

    @Payable
    @External
    public void sendMessageAny(String _to, byte[] _data) {
        _sendCall(Context.getValue(), _to, _data);
    }

    private BigInteger _sendCall(BigInteger value, String to, byte[] envelope) {
        return Context.call(BigInteger.class, value, this.callSvc, "sendCall", to, envelope);
    }

    @Payable
    @External
    public void sendMessage(String _to, byte[] _data, @Optional byte[] _rollback) {
        _sendCallMessage(Context.getValue(), _to, _data, _rollback);
    }

    private BigInteger _sendCallMessage(BigInteger value, String to, byte[] data, byte[] rollback) {
        String net = NetworkAddress.valueOf(to).net();
        return Context.call(BigInteger.class, value, this.callSvc, "sendCallMessage", to, data, rollback,
                getSources(net), getDestinations(net));
    }

    @External
    public void handleCallMessage(String _from, byte[] _data, String[] protocols) {
        onlyCallService();
        NetworkAddress from = NetworkAddress.parse(_from);
        String rollbackAddress = Context.call(String.class, this.callSvc, "getNetworkAddress");
        Context.println("handleCallMessage: from=" + _from + ", data=" + new String(_data));
        if (rollbackAddress.equals(_from)) {
            return;
        } else {
            Context.require(equals(protocols, getSources(from.net())), "invalid protocols");

            Context.require(!new String(_data).equals("rollback"), "failed");

            if (new String(_data).equals("reply-response")) {
                // response message
                _sendCallMessage(BigInteger.ZERO, _from, new byte[] { 1, 2, 3 }, null);
            }
            MessageReceived(_from, _data);
        }
    }

    @EventLog
    public void MessageReceived(String _from, byte[] _data) {
    }

    public static <T> boolean equals(T[] a, T[] b) {
        if (a == b)
            return true;
        if (a == null || b == null)
            return false;

        int length = a.length;
        if (b.length != length)
            return false;

        for (int i = 0; i < length; i++) {
            if (!a[i].equals(b[i]))
                return false;
        }

        return true;
    }
}
