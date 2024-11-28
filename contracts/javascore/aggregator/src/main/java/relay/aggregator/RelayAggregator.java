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
    private final Integer DEFAULT_SIGNATURE_THRESHOLD = 1;

    private final VarDB<Integer> signatureThreshold = Context.newVarDB("signatureThreshold", Integer.class);

    private final VarDB<Address> admin = Context.newVarDB("admin", Address.class);

    private final ArrayDB<Address> relayers = Context.newArrayDB("relayers", Address.class);
    private final DictDB<Address, Boolean> relayersLookup = Context.newDictDB("relayersLookup", Boolean.class);

    private final DictDB<String, Packet> packets = Context.newDictDB("packets", Packet.class);
    private final DictDB<String, Boolean> acknowledgedPackets = Context.newDictDB("acknowledgedPackets", Boolean.class);

    private final BranchDB<String, DictDB<Address, byte[]>> signatures = Context.newBranchDB("signatures",
            byte[].class);

    public RelayAggregator(Address _admin) {
        if (admin.get() == null) {
            admin.set(_admin);
            signatureThreshold.set(DEFAULT_SIGNATURE_THRESHOLD);
            addRelayer(_admin);
        }
    }

    @External
    public void setAdmin(Address _admin) {
        adminOnly();

        Context.require(admin.get() != _admin, "admin already set");

        // add new admin as relayer
        addRelayer(_admin);

        // remove old admin from relayer list
        removeRelayer(admin.get());

        admin.set(_admin);
    }

    @External(readonly = true)
    public Address getAdmin() {
        return admin.get();
    }

    @External
    public void setSignatureThreshold(int threshold) {
        adminOnly();
        Context.require(threshold > 0 && threshold <= relayers.size(),
                "threshold value should be at least 1 and not greater than relayers size");
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
    public void setRelayers(Address[] newRelayers, int threshold) {
        adminOnly();

        if (newRelayers.length > 0) {
            HashMap<Address, Boolean> newRelayersMap = new HashMap<Address, Boolean>();
            for (Address newRelayer : newRelayers) {
                newRelayersMap.put(newRelayer, true);
                addRelayer(newRelayer);
            }

            Address adminAdrr = admin.get();
            for (int i = 0; i < relayers.size(); i++) {
                Address oldRelayer = relayers.get(i);
                if (!oldRelayer.equals(adminAdrr) && !newRelayersMap.containsKey(oldRelayer)) {
                    removeRelayer(oldRelayer);
                }
            }
        }

        Context.require(threshold > 0 && threshold <= relayers.size(),
                "threshold value should be at least 1 and not greater than relayers size");

        signatureThreshold.set(threshold);
    }

    @External(readonly = true)
    public boolean packetSubmitted(
            Address relayer,
            String srcNetwork,
            String srcContractAddress,
            BigInteger srcSn) {
        String pktID = Packet.createId(srcNetwork, srcContractAddress, srcSn);
        byte[] existingSign = signatures.at(pktID).get(relayer);
        return existingSign != null;
    }

    @External(readonly = true)
    public boolean packetAcknowledged(
            String srcNetwork,
            String srcContractAddress,
            BigInteger srcSn) {
        String pktID = Packet.createId(srcNetwork, srcContractAddress, srcSn);
        return acknowledgedPackets.get(pktID) != null && acknowledgedPackets.get(pktID) == true;
    }

    @External
    public void submitPacket(
            String srcNetwork,
            String srcContractAddress,
            BigInteger srcSn,
            BigInteger srcHeight,
            String dstNetwork,
            String dstContractAddress,
            byte[] data,
            byte[] signature) {

        relayersOnly();

        Packet pkt = new Packet(srcNetwork, srcContractAddress, srcSn, srcHeight, dstNetwork, dstContractAddress, data);
        String pktID = pkt.getId();

        if (acknowledgedPackets.get(pktID) != null && acknowledgedPackets.get(pktID) == true) {
            return;
        }

        if (packets.get(pktID) == null) {
            packets.set(pktID, pkt);
            if (signatureThreshold.get() > 1) {
                PacketRegistered(
                        pkt.getSrcNetwork(),
                        pkt.getSrcContractAddress(),
                        pkt.getSrcSn(),
                        pkt.getSrcHeight(),
                        pkt.getDstNetwork(),
                        pkt.getDstContractAddress(),
                        pkt.getData());
            }

        }

        byte[] existingSign = signatures.at(pktID).get(Context.getCaller());
        Context.require(existingSign == null, "Signature already exists");

        setSignature(pktID, Context.getCaller(), signature);

        if (signatureThresholdReached(pktID)) {
            byte[][] sigs = getSignatures(srcNetwork, srcContractAddress, srcSn);
            byte[] encodedSigs = serializeSignatures(sigs);
            PacketAcknowledged(
                    pkt.getSrcNetwork(),
                    pkt.getSrcContractAddress(),
                    pkt.getSrcSn(),
                    pkt.getSrcHeight(),
                    pkt.getDstNetwork(),
                    pkt.getDstContractAddress(),
                    pkt.getData(),
                    encodedSigs);
            acknowledgedPackets.set(pktID, true);
            removePacket(pktID);
        }
    }

    private byte[][] getSignatures(String srcNetwork, String srcContractAddress, BigInteger srcSn) {
        String pktID = Packet.createId(srcNetwork, srcContractAddress, srcSn);
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
        Boolean isRelayer = relayersLookup.get(caller);
        Context.require(isRelayer != null && isRelayer, "Unauthorized: caller is not a registered relayer");
    }

    private void addRelayer(Address newRelayer) {
        if (relayersLookup.get(newRelayer) == null) {
            relayers.add(newRelayer);
            relayersLookup.set(newRelayer, true);
        }
    }

    private void removeRelayer(Address oldRelayer) {
        if (relayersLookup.get(oldRelayer)) {
            relayersLookup.set(oldRelayer, null);
            Address top = relayers.pop();
            for (int i = 0; i < relayers.size(); i++) {
                if (oldRelayer.equals(relayers.get(i))) {
                    relayers.set(i, top);
                    break;
                }
            }
        }
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
            String srcContractAddress,
            BigInteger srcSn,
            BigInteger srcHeight,
            String dstNetwork,
            String dstContractAddress,
            byte[] data) {
    }

    @EventLog(indexed = 2)
    public void PacketAcknowledged(
            String srcNetwork,
            String srcContractAddress,
            BigInteger srcSn,
            BigInteger srcHeight,
            String dstNetwork,
            String dstContractAddress,
            byte[] data,
            byte[] signatures) {
    }
}