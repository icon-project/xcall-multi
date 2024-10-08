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

package xcall.adapter.cluster;

import java.math.BigInteger;
import score.Context;

import score.Address;
import score.BranchDB;
import score.DictDB;
import score.VarDB;
import score.ArrayDB;
import scorex.util.ArrayList;

import score.annotation.EventLog;
import score.annotation.External;
import score.annotation.Payable;
import java.util.List;



public class ClusterConnection {
    protected final VarDB<Address> xCall = Context.newVarDB("callService", Address.class);
    protected final VarDB<Address> adminAddress = Context.newVarDB("relayer", Address.class);
    protected final VarDB<BigInteger> reqValidatorCnt = Context.newVarDB("reqValidatorCnt", BigInteger.class);
    private final VarDB<BigInteger> connSn = Context.newVarDB("connSn", BigInteger.class);
    private final ArrayDB<Address> validators =  Context.newArrayDB("signers", Address.class);

    protected final DictDB<String, BigInteger> messageFees = Context.newDictDB("messageFees", BigInteger.class);
    protected final DictDB<String, BigInteger> responseFees = Context.newDictDB("responseFees", BigInteger.class);
    protected final BranchDB<String, DictDB<BigInteger, Boolean>> receipts = Context.newBranchDB("receipts",
            Boolean.class);
    private final DictDB<Address, Boolean> validatorsLookup = Context.newDictDB("validatorsLookup", Boolean.class);

    public ClusterConnection(Address _relayer, Address _xCall) {
        if (xCall.get() == null) {
            xCall.set(_xCall);
            adminAddress.set(_relayer);
            connSn.set(BigInteger.ZERO);
            validators.add(_relayer);
            validatorsLookup.set(_relayer,true);
            ValidatorAdded(_relayer);
        }
    }

      /**
     * Retrieves the validators.
     *
     * @return The validators .
     */
    @External(readonly = true)
    public Address[] listValidators() {
        Address[] sgs = new Address[validators.size()];
        for(int i = 0; i < validators.size(); i++) {
            sgs[i] = validators.get(i);
        }
        return sgs;
    }

    @External
    public void addValidator(Address _validator) {
        OnlyAdmin();
        Context.require(validatorsLookup.get(_validator)==null,"Validator already exists");
        validators.add(_validator);
        validatorsLookup.set(_validator,true);
        ValidatorAdded(_validator);
    }

    @External
    public void removeValidator(Address _validator) {
        OnlyAdmin();
        Context.require(_validator != adminAddress.get(),"cannot remove admin");
        Context.require(validatorsLookup.get(_validator)!=null,"Validator doesn't exists");
        Context.require((this.validators.size() - 1) >= reqValidatorCnt.get().intValue(),"Validator size less than required count after removal");
        Address top = this.validators.pop();
        if (!top.equals(_validator)) {
            for (int i = 0; i < this.validators.size(); i++) {
                if (_validator.equals(this.validators.get(i))) {
                    this.validators.set(i, top);
                    break;
                }
            }
            validatorsLookup.set(_validator,null);
            ValidatorRemoved(_validator);
        }

    }

    @EventLog(indexed = 2)
    public void Message(String targetNetwork, BigInteger connSn, byte[] msg) {
    }

    @EventLog(indexed = 1)
    public void ValidatorAdded(Address _validator) {
    }

    @EventLog(indexed = 1)
    public void ValidatorRemoved(Address _validator) {
    }

    /**
     * Sets the admin address.
     *
     * @param _relayer the new admin address
     */
    @External
    public void setAdmin(Address _relayer) {
        OnlyAdmin();
        adminAddress.set(_relayer);
    }     

    /**
     * Retrieves the admin address.
     *
     * @return The admin address.
     */
    @External(readonly = true)
    public Address admin() {
        return adminAddress.get();
    }

    /**
     * Sets the required validator count
     *
     * @param _validatorCnt the new required validator count
     */
    @External
    public void setRequiredValidatorCount(BigInteger _validatorCnt) {
        OnlyAdmin();
        reqValidatorCnt.set(_validatorCnt);
    }

     /**
     * Retrieves the required validator count.
     *
     * @return The required validator count.
     */
    @External(readonly = true)
    public BigInteger requiredValidatorCount() {
        return reqValidatorCnt.get();
    }

    /**
     * Sets the fee to the target network
     *
     * @param networkId   String Network Id of target chain
     * @param messageFee  The fee needed to send a Message
     * @param responseFee The fee of the response
     */
    @External
    public void setFee(String networkId, BigInteger messageFee, BigInteger responseFee) {
        OnlyAdmin();
        messageFees.set(networkId, messageFee);
        responseFees.set(networkId, responseFee);
    }

    /**
     * Returns the fee associated with the given destination address.
     *
     * @param to       String Network Id of target chain
     * @param response whether the responding fee is included
     * @return The fee of sending a message to a given destination network
     */
    @External(readonly = true)
    public BigInteger getFee(String to, boolean response) {
        BigInteger messageFee = messageFees.getOrDefault(to, BigInteger.ZERO);
        if (response) {
            BigInteger responseFee = responseFees.getOrDefault(to, BigInteger.ZERO);
            return messageFee.add(responseFee);
        }
        return messageFee;
    }

    /**
     * Sends a message to the specified network.
     *
     * @param to  Network Id of destination network
     * @param svc name of the service
     * @param sn  positive for two-way message, zero for one-way message, negative
     *            for response(for xcall message)
     * @param msg serialized bytes of Service Message
     */
    @Payable
    @External
    public void sendMessage(String to, String svc, BigInteger sn, byte[] msg) {
        Context.require(Context.getCaller().equals(xCall.get()), "Only xCall can send messages");
        BigInteger fee = BigInteger.ZERO;
        if (sn.compareTo(BigInteger.ZERO) > 0) {
            fee = getFee(to, true);
        } else if (sn.equals(BigInteger.ZERO)) {
            fee = getFee(to, false);
        }

        BigInteger nextConnSn = connSn.get().add(BigInteger.ONE);
        connSn.set(nextConnSn);

        Context.require(Context.getValue().compareTo(fee) >= 0, "Insufficient balance");
        Message(to, nextConnSn, msg);
    }

    /**
     * Receives a message from a source network.
     *
     * @param srcNetwork the source network id from which the message is received
     * @param _connSn    the serial number of the connection message
     * @param msg        serialized bytes of Service Message
     * @param signatures array of signatures
     */
     @External
     public void recvMessageWithSignatures(String srcNetwork, BigInteger _connSn, byte[] msg,
                                           byte[][] signatures) {
         OnlyAdmin();
         Context.require(signatures.length >= reqValidatorCnt.get().intValue(), "Not enough signatures");
         List<Address> uniqueValidators = new ArrayList<>();
         for (byte[] signature : signatures) {
             Address validator = getValidator(msg, signature);
             Context.require(validatorsLookup.get(validator)!=null, "Invalid signature provided");
             if (!uniqueValidators.contains(validator)) {
                 uniqueValidators.add(validator);
             }
         }
         Context.require(uniqueValidators.size() >= reqValidatorCnt.get().intValue(), "Not enough valid signatures");
         recvMessage(srcNetwork, _connSn, msg);
     }

    /**
     * Receives a message from a source network.
     *
     * @param srcNetwork the source network id from which the message is received
     * @param _connSn    the serial number of the connection message
     * @param msg        serialized bytes of Service Message
     */
    @External
    public void recvMessage(String srcNetwork, BigInteger _connSn, byte[] msg) {
        OnlyAdmin();
        Context.require(!receipts.at(srcNetwork).getOrDefault(_connSn, false), "Duplicate Message");
        receipts.at(srcNetwork).set(_connSn, true);
        Context.call(xCall.get(), "handleMessage", srcNetwork, msg);
    }

    private Address getValidator(byte[] msg, byte[] sig){
        byte[] hashMessage = getHash(msg);
        byte[] key = Context.recoverKey("ecdsa-secp256k1", hashMessage, sig, true);
        return Context.getAddressFromKey(key);
    }

    private byte[] getHash(byte[] msg){
        return Context.hash("keccak-256", msg);
    }

    /**
     * Reverts a message.
     *
     * @param sn the serial number of xcall message representing the message to
     *           revert
     */
    @External
    public void revertMessage(BigInteger sn) {
        OnlyAdmin();
        Context.call(xCall.get(), "handleError", sn);
    }

    /**
     * Claim the fees.
     *
     */
    @External
    public void claimFees() {
        OnlyAdmin();
        Context.transfer(admin(), Context.getBalance(Context.getAddress()));
    }

    /**
     * Get the receipts for a given source network and serial number.
     *
     * @param srcNetwork the source network id
     * @param _connSn    the serial number of connection message
     * @return the receipt if is has been recived or not
     */
    @External(readonly = true)
    public boolean getReceipts(String srcNetwork, BigInteger _connSn) {
        return receipts.at(srcNetwork).getOrDefault(_connSn, false);
    }

    /**
     * Checks if the caller of the function is the admin.
     *
     * @return true if the caller is the admin, false otherwise
     */
    private void OnlyAdmin() {
        Context.require(Context.getCaller().equals(adminAddress.get()), "Only admin can call this function");
    }

}