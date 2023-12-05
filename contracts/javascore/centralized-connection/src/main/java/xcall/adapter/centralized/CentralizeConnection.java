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

 package xcall.adapter.centralized;

 import java.math.BigInteger;
 import score.ByteArrayObjectWriter;
 import score.Context;
 import score.ObjectReader;
 import score.ObjectWriter;
 
 
 import score.Address;
 import score.BranchDB;
 import score.Context;
 import score.DictDB;
 import score.RevertedException;
 import score.UserRevertedException;
 import score.VarDB;
 
 
 import score.annotation.EventLog;
 import score.annotation.External;
 import score.annotation.Optional;
 import score.annotation.Payable;
 
 public class CentralizeConnection {
     protected final VarDB<Address> admin = Context.newVarDB("admin", Address.class);
     protected final VarDB<Address> xCall = Context.newVarDB("callService", Address.class);
     protected final VarDB<Address> relayer = Context.newVarDB("relayer", Address.class);
 
     protected final DictDB<String, BigInteger> messageFees = Context.newDictDB("messageFees", BigInteger.class);
     protected final DictDB<String, BigInteger> responseFees = Context.newDictDB("responseFees", BigInteger.class);
     protected final DictDB<byte[], Boolean> seenDeliveryVaaHashes = Context.newDictDB("seenDeliveryVaaHashes", Boolean.class);
 
     public XCallCentralizeConnection(Address _xCall, Address _relayer) {
         if ( xCall.get() == null ) {
             xCall.set(_xCall);
             admin.set(Context.getCaller());
             relayer.set(_relayer);
         }
     }
 
     @EventLog(indexed=1)
     public void Message(String targetNetwork, BigInteger sn, byte[] msg) {}
 
    @External
    public void setRelayer(Address _relayer) {
        Context.require(Context.getCaller().equals(admin.get()), "Only admin can set relayer");
        relayer.set(_relayer);
    }

    @External(readonly = true)
    public Address getRelayer() {
        return relayer.get();
    }
 
     @External
     public void setFee(String networkId, BigInteger messageFee, BigInteger responseFee) {
         Context.require(Context.getCaller().equals(admin.get()), "Only admin can set fees");
         messageFees.set(networkId, messageFee);
         responseFees.set(networkId, responseFee);
     }
 
     @External(readonly = true)
     public BigInteger getFee(String to, boolean response) {
         BigInteger messageFee = messageFees.get(to);
         if (response) {
             BigInteger responseFee = responseFees.get(to);
             return messageFee.add(responseFee);
         }
         return messageFee;
     }
 
     @Payable
     @External
     public void sendMessage(String to, String svc, BigInteger sn, byte[] msg) {
         Context.require(Context.getCaller().equals(xCall.get()), "Only xCall can send messages");
         BigInteger fee = this.getFee(to, false);
         Context.require(Context.getValue().compareTo(fee)>0,"Fee is not Sufficient");
         Message(to, sn, msg);
     }
 
     @Payable
     @External
     public void recvMessage(String srcNID, String sn, byte[] msg) {
         byte[] hash = Context.hash("keccak-256",encodePacked(msg,sn));
         Context.require(seenDeliveryVaaHashes.getOrDefault(hash,null)==null, "Message already processed");
         seenDeliveryVaaHashes.set(hash, true);
         Context.call(xCall.get(), "handleMessage", srcNID, msg);
     }
 
     @External
     public void setAdmin(Address address) {
        Context.require(Context.getCaller().equals(admin.get()), "Only admin can set admin");
        admin.set(address);
     }
 
     @External(readonly = true)
     public Address admin() {
         return admin.get();
     }
 
     public static byte[] encodePacked(Object... params) {
         StringBuilder result = new StringBuilder();
         for (Object param : params) {
             result.append(param.toString());
         }
         return result.toString().getBytes();
     }
 }