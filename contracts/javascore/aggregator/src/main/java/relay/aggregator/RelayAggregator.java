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
import score.VarDB;
import score.DictDB;
import score.BranchDB;

import score.annotation.External;

public class RelayAggregator {
    private final VarDB<Address> admin = Context.newVarDB("admin", Address.class);

    private final DictDB<Address, Boolean> relayers = Context.newDictDB("relayers", Boolean.class);

    private final BranchDB<String, DictDB<BigInteger, byte[]>> packets = Context.newBranchDB("packets", byte[].class);

    private final BranchDB<String, BranchDB<BigInteger, DictDB<Address, String>>> signatures = Context.newBranchDB("signatures", DictDB.class);

    public RelayAggregator(Address _admin, Address[] _relayers) {
        if (admin.get() == null) {
            admin.set(_admin);
            for (Address relayer : _relayers) {
                relayers.set(relayer, true);
            }
        }
    }

    /**
     * Registers a new packet.
     *
     * @param nid network ID
     * @param sn sequence number
     * @param data packet data
     */
    @External
    public void registerPacket(String nid, BigInteger sn, byte[] data) {
        adminOnly();
        DictDB<BigInteger, byte[]> packetDict = getPackets(nid);
        byte[] pkt = packetDict.get(sn);
        Context.require(pkt == null, "Packet already exists");

        packetDict.set(sn, data);
    }

    /**
     * Submits a signature for the registered packet.
     *
     * @param nid network ID
     * @param sn sequence number
     * @param signature packet signature
     */
    @External
    public void submitSignature(String nid, BigInteger sn, String signature) {
        relayersOnly();

        DictDB<BigInteger, byte[]> packetDict = getPackets(nid);
        byte[] packetData = packetDict.get(sn);
        Context.require(packetData != null, "Packet does not exist");


        DictDB<Address, String> addressSigns = getSignatures(nid, sn);

        Address caller = Context.getCaller();
        String sign = addressSigns.get(caller);
        Context.require(sign == null, "Signature already exists.");

        addressSigns.set(caller, signature);
    }

    /**
     * Retrieves the signatures dictionary for the packet.
     *
     * @param nid network ID of the source chain
     * @param sn sequence number of the source chain message
     * @return map of relayer addresses to signatures
     */
    protected DictDB<Address, String> getSignatures(String nid, BigInteger sn) {
        BranchDB<BigInteger, DictDB<Address, String>> snSigns = signatures.at(nid);
        if (snSigns == null) {
            snSigns = Context.newBranchDB(nid+"-signs", DictDB.class);
        }
        return snSigns.at(sn);
    }
    

    /**
     * Sets the admin address.
     *
     * @param _admin the new admin address
     */
    @External
    public void setAdmin(Address _admin) {
        adminOnly();
        admin.set(_admin);
    }

    /**
     * Retrieves the admin address.
     *
     * @return admin address.
     */
    @External(readonly = true)
    public Address getAdmin() {
        return admin.get();
    }

    /**
     * Retrieves the packets dictionary.
     * *
     * @param nid network id of the source chain
     * @return list of mapping from sn to packet data.
     */
    protected DictDB<BigInteger, byte[]> getPackets(String nid) {
        return packets.at(nid);
    }

    /**
     * Checks if the caller of the function is the admin.
     *
     * @return true if the caller is the admin, false otherwise
     */
    private void adminOnly() {
        Context.require(Context.getCaller().equals(admin.get()), "Unauthorized: caller is not the leader relayer");
    }

    /**
     * Checks if the caller of the function is among the relayers.
     *
     * @return true if the caller is among the relayers, otherwise false
     */
    private void relayersOnly() {
        Address caller = Context.getCaller();
        boolean isRelayer = relayers.get(caller);
        Context.require(isRelayer, "Unauthorized: caller is not a registered relayer");
    }
}