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

import score.Context;

import java.math.BigInteger;

import score.Address;
import score.VarDB;
import score.DictDB;
import score.BranchDB;

import score.annotation.External;

public class RelayAggregator {
    protected final VarDB<Address> admin = Context.newVarDB("admin", Address.class);

    protected final DictDB<Address, Boolean> relayers = Context.newDictDB("relayers", Boolean.class);

    protected final BranchDB<String, DictDB<BigInteger, byte[]>> packets = Context.newBranchDB("packets", byte[].class);

    // protected final BranchDB<String, DictDB<BigInteger, String[]>> signatures = Context.newBranchDB("signatures", String[].class);

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
     * @param nid  the network ID
     * @param sn   the sequence number
     * @param data the packet data
     */
    @External
    public void registerPacket(String nid, BigInteger sn, byte[] data) {
        adminOnly();
        DictDB<BigInteger, byte[]> packetDict = packets.at(nid);
        byte[] pkt = packetDict.get(sn);
        Context.require(pkt == null, "Packet already exists.");

        packetDict.set(sn, data);
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
     * Checks if the caller of the function is the admin.
     *
     * @return true if the caller is the admin, false otherwise
     */
    private void adminOnly() {
        Context.require(Context.getCaller().equals(admin.get()), "Unauthorized to call this method");
    }
}