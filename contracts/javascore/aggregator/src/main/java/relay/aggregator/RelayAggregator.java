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

package relay.aggregator;

import java.math.BigInteger;

import score.Context;
import score.Address;
import score.ArrayDB;
import score.VarDB;
import score.DictDB;
import score.BranchDB;
import score.ByteArrayObjectWriter;
import score.annotation.EventLog;
import score.annotation.External;
import score.ObjectReader;
import scorex.util.ArrayList;
import scorex.util.HashMap;

public class RelayAggregator {
    private final Integer DEFAULT_SIGNATURE_THRESHOLD = 2;

    private final VarDB<Integer> signatureThreshold = Context.newVarDB("signatureThreshold", Integer.class);

    private final VarDB<Address> admin = Context.newVarDB("admin", Address.class);

    private final ArrayDB<Address> relayers = Context.newArrayDB("relayers", Address.class);

    private final DictDB<String, Packet> packets = Context.newDictDB("packets", Packet.class);

    private final BranchDB<String, DictDB<Address, byte[]>> signatures = Context.newBranchDB("signatures",
            byte[].class);

    public RelayAggregator(Address _admin) {
        if (admin.get() == null) {
            admin.set(_admin);
            signatureThreshold.set(DEFAULT_SIGNATURE_THRESHOLD);
        }
    }

    @External
    public void setAdmin(Address _admin) {
        adminOnly();
        admin.set(_admin);
    }

    @External(readonly = true)
    public Address getAdmin() {
        return admin.get();
    }

    @External
    public void setSignatureThreshold(int threshold) {
        adminOnly();
        signatureThreshold.set(threshold);
    }

    @External(readonly = true)
    public int getSignatureThreshold() {
        return signatureThreshold.get();
    }

    @External(readonly = true)
    public Address[] getRelayers() {
        Address[] rlrs = new Address[relayers.size()];
        for (int i = 0; i < relayers.size(); i++) {
            rlrs[i] = relayers.get(i);
        }
        return rlrs;
    }

    @External
    public void addRelayers(Address[] newRelayers) {
        adminOnly();

        Context.require(newRelayers != null && newRelayers.length != 0, "new relayers cannot be empty");

        HashMap<Address, Boolean> existingRelayers = new HashMap<Address, Boolean>();
        for (int i = 0; i < relayers.size(); i++) {
            Address relayer = relayers.get(i);
            existingRelayers.put(relayer, true);
        }

        for (Address newRelayer : newRelayers) {
            if (!existingRelayers.containsKey(newRelayer)) {
                relayers.add(newRelayer);
                existingRelayers.put(newRelayer, true);
            }
        }
    }

    @External
    public void removeRelayers(Address[] relayersToBeRemoved) {
        adminOnly();

        Context.require(relayersToBeRemoved != null && relayersToBeRemoved.length != 0,
                "relayers to be removed cannot be empty");

        HashMap<Address, Integer> existingRelayers = new HashMap<Address, Integer>();
        for (int i = 0; i < relayers.size(); i++) {
            Address relayer = relayers.get(i);
            existingRelayers.put(relayer, i);
        }

        for (Address relayerToBeRemoved : relayersToBeRemoved) {
            if (existingRelayers.containsKey(relayerToBeRemoved)) {
                Address top = relayers.pop();
                if (!top.equals(relayerToBeRemoved)) {
                    Integer pos = existingRelayers.get(relayerToBeRemoved);
                    relayers.set(pos, top);
                }
                existingRelayers.remove(relayerToBeRemoved);
            }
        }
    }

    @External
    public void registerPacket(
            String srcNetwork,
            String contractAddress,
            BigInteger srcSn,
            BigInteger srcHeight,
            String dstNetwork,
            byte[] data) {

        adminOnly();

        Packet pkt = new Packet(srcNetwork, contractAddress, srcSn, srcHeight, dstNetwork, data);
        String id = pkt.getId();

        Context.require(packets.get(id) == null, "Packet already exists");

        packets.set(id, pkt);

        PacketRegistered(
                pkt.getSrcNetwork(),
                pkt.getContractAddress(),
                pkt.getSrcSn(),
                pkt.getSrcHeight(),
                pkt.getDstNetwork(),
                pkt.getData());
    }

    @External
    public void acknowledgePacket(
            String srcNetwork,
            String contractAddress,
            BigInteger srcSn,
            byte[] signature) {

        relayersOnly();

        String pktID = Packet.createId(srcNetwork, contractAddress, srcSn);
        Packet pkt = packets.get(pktID);
        Context.require(pkt != null, "Packet not registered");

        byte[] existingSign = signatures.at(pktID).get(Context.getCaller());
        Context.require(existingSign == null, "Signature already exists");

        setSignature(pktID, Context.getCaller(), signature);

        if (signatureThresholdReached(pktID)) {
            byte[][] sigs = getSignatures(srcNetwork, contractAddress, srcSn);
            byte[] encodedSigs = serializeSignatures(sigs);
            PacketAcknowledged(
                    pkt.getSrcNetwork(),
                    pkt.getContractAddress(),
                    pkt.getSrcSn(),
                    pkt.getSrcHeight(),
                    pkt.getDstNetwork(),
                    pkt.getData(),
                    encodedSigs);
            removePacket(pktID);
        }
    }

    private byte[][] getSignatures(String srcNetwork, String contractAddress, BigInteger srcSn) {
        String pktID = Packet.createId(srcNetwork, contractAddress, srcSn);
        DictDB<Address, byte[]> signDict = signatures.at(pktID);
        ArrayList<byte[]> signatureList = new ArrayList<byte[]>();

        for (int i = 0; i < relayers.size(); i++) {
            Address relayer = relayers.get(i);
            byte[] sign = signDict.get(relayer);
            if (sign != null) {
                signatureList.add(sign);
            }
        }

        byte[][] sigs = new byte[signatureList.size()][];
        for (int i = 0; i < signatureList.size(); i++) {
            sigs[i] = signatureList.get(i);
        }
        return sigs;
    }

    protected void setSignature(String pktID, Address addr, byte[] sign) {
        signatures.at(pktID).set(addr, sign);
    }

    protected static byte[] serializeSignatures(byte[][] sigs) {
        ByteArrayObjectWriter w = Context.newByteArrayObjectWriter("RLPn");
        w.beginList(sigs.length);

        for (byte[] sig : sigs) {
            w.write(sig);
        }

        w.end();
        return w.toByteArray();
    }

    protected static byte[][] deserializeSignatures(byte[] encodedSigs) {
        ObjectReader r = Context.newByteArrayObjectReader("RLPn", encodedSigs);

        ArrayList<byte[]> sigList = new ArrayList<>();

        r.beginList();
        while (r.hasNext()) {
            sigList.add(r.readByteArray());
        }
        r.end();

        byte[][] sigs = new byte[sigList.size()][];
        for (int i = 0; i < sigList.size(); i++) {
            sigs[i] = sigList.get(i);
        }

        return sigs;
    }

    private void adminOnly() {
        Context.require(Context.getCaller().equals(admin.get()), "Unauthorized: caller is not the leader relayer");
    }

    private void relayersOnly() {
        Address caller = Context.getCaller();
        Boolean isRelayer = false;
        for (int i = 0; i < relayers.size(); i++) {
            Address relayer = relayers.get(i);
            if (relayer.equals(caller)) {
                isRelayer = true;
                break;
            }
        }
        Context.require(isRelayer, "Unauthorized: caller is not a registered relayer");
    }

    private Boolean signatureThresholdReached(String pktID) {
        int noOfSignatures = 0;
        for (int i = 0; i < relayers.size(); i++) {
            Address relayer = relayers.get(i);
            byte[] relayerSign = signatures.at(pktID).get(relayer);
            if (relayerSign != null) {
                noOfSignatures++;
            }
        }
        return noOfSignatures >= signatureThreshold.get();
    }

    private void removePacket(String pktID) {
        packets.set(pktID, null);
        DictDB<Address, byte[]> signDict = signatures.at(pktID);

        for (int i = 0; i < relayers.size(); i++) {
            Address relayer = relayers.get(i);
            signDict.set(relayer, null);
        }
    }

    @EventLog(indexed = 2)
    public void PacketRegistered(
            String srcNetwork,
            String contractAddress,
            BigInteger srcSn,
            BigInteger srcHeight,
            String dstNetwork,
            byte[] data) {
    }

    @EventLog(indexed = 2)
    public void PacketAcknowledged(
            String srcNetwork,
            String contractAddress,
            BigInteger srcSn,
            BigInteger srcHeight,
            String dstNetwork,
            byte[] data,
            byte[] signatures) {
    }
}