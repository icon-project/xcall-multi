package network.icon.intent;

import java.math.BigInteger;
import java.util.Arrays;

import network.icon.intent.constants.Constant;
import network.icon.intent.structs.Cancel;
import network.icon.intent.structs.OrderFill;
import network.icon.intent.structs.OrderMessage;
import network.icon.intent.structs.SwapOrder;
import network.icon.intent.utils.PermitTransferFromData;
import network.icon.intent.utils.SignatureTransferDetailsData;
import network.icon.intent.utils.SwapOrderData;
import score.*;
import score.annotation.EventLog;
import score.annotation.External;
import score.annotation.Payable;

public class Intent extends GeneralizedConnection {

    public final VarDB<BigInteger> depositId = Context.newVarDB(Constant.DEPOSIT_ID, BigInteger.class);
    public final VarDB<String> networkId = Context.newVarDB(Constant.NETWORK_ID, String.class);
    public final VarDB<BigInteger> protocolFee = Context.newVarDB(Constant.PROTOCOL_FEE, BigInteger.class);
    public static final VarDB<Address> feeHandler = Context.newVarDB(Constant.FEE_HANDLER, Address.class);
    public static final VarDB<Address> owner = Context.newVarDB(Constant.OWNER, Address.class);
    public final VarDB<Address> nativeAddress = Context.newVarDB(Constant.NATIVE_ADDRESS, Address.class);
    final static BranchDB<Address, DictDB<Address, BigInteger>> deposit = Context.newBranchDB(Constant.DEPOSIT,
            BigInteger.class);
    final static DictDB<BigInteger, SwapOrder> orders = Context.newDictDB(Constant.ORDERS, SwapOrder.class);
    final static DictDB<byte[], Boolean> finishedOrders = Context.newDictDB(Constant.FINISHED_ORDERS, Boolean.class);
    public final VarDB<PermitTransferFromData> permit = Context.newVarDB(Constant.PERMIT, PermitTransferFromData.class);

    @EventLog(indexed = 3)
    public void SwapIntent(
            BigInteger id,
            String emitter,
            String srcNID,
            String dstNID,
            String creator,
            String destinationAddress,
            String token,
            BigInteger amount,
            String toToken,
            BigInteger toAmount,
            byte[] data) {
    }

    @EventLog(indexed = 2)
    public void OrderFilled(BigInteger id, String srcNID) {
    }

    @EventLog(indexed = 2)
    public void OrderCancelled(BigInteger id, String srcNID) {
    }

    @EventLog(indexed = 1)
    public void OrderClosed(BigInteger id) {
    }

    public Intent(String _nid, BigInteger _protocolFee, Address _feeHandler, Address _relayer) {
        networkId.set(_nid);
        protocolFee.set(_protocolFee);
        feeHandler.set(_feeHandler);
        relayAddress.set(_relayer);
        nativeAddress.set(Address.fromString("cx0000000000000000000000000000000000000000"));
        owner.set(Context.getCaller());
    }

    @External
    @Payable
    public void swap(SwapOrderData swapOrderData) {
        Context.require(swapOrderData.token != null, "Token can't be null");
        Context.require(Context.getCaller().toString().equals(swapOrderData.creator), "Creator must be sender");

        Address token = Address.fromString(swapOrderData.token);
        Address nativAddress = nativeAddress.get();
        if (token.equals(nativAddress)) {
            Context.require(Context.getValue().equals(swapOrderData.amount),
                    "Deposit amount not equal to order amount");
        } else {
            Context.require(Context.getValue().equals(BigInteger.valueOf(0)),
                    "Nativ Token Must Be Zero");
            Context.call(token, "transfer", Context.getAddress(), swapOrderData.amount);
        }

        SwapOrder swapOrder = new SwapOrder(swapOrderData.id, swapOrderData.emitter, swapOrderData.srcNID,
                swapOrderData.dstNID, swapOrderData.creator, swapOrderData.destinationAddress, swapOrderData.token,
                swapOrderData.amount, swapOrderData.toToken, swapOrderData.toAmount, swapOrderData.data);

        _swap(swapOrder);
    }

    @External
    public void swapPermit2(
            SwapOrderData order,
            byte[] signature,
            PermitTransferFromData permit) {
        Permit2 permit2 = new Permit2();
        order.id = BigInteger.ZERO;

        SignatureTransferDetailsData transferDetails = new SignatureTransferDetailsData();
        transferDetails.to = Context.getAddress();
        transferDetails.requestedAmount = order.amount;

        permit2.permitWitnessTransferFrom(
                permit,
                transferDetails,
                Address.fromString(order.creator),
                Context.hash("keccak-256", order.data),
                "",
                signature);

        SwapOrder swapOrder = new SwapOrder(
                order.id,
                order.emitter,
                order.srcNID,
                order.dstNID,
                order.creator,
                order.destinationAddress,
                order.token,
                order.amount,
                order.toToken,
                order.toAmount,
                order.data);

        _swap(swapOrder);
    }

    void _swap(SwapOrder swapOrder) {
        BigInteger id = this.depositId.getOrDefault(BigInteger.ZERO).add(BigInteger.valueOf(1));
        swapOrder.id = id;
        Context.require(swapOrder.srcNID.equals(this.networkId.get()), "NID is misconfigured");
        Context.require(swapOrder.emitter.equals(Context.getAddress().toString()),
                "Emitter specified is not this");
        orders.set(id, swapOrder);
        SwapIntent(id, swapOrder.emitter, swapOrder.srcNID, swapOrder.dstNID,
                swapOrder.creator,
                swapOrder.destinationAddress, swapOrder.token, swapOrder.amount,
                swapOrder.toToken, swapOrder.toAmount,
                swapOrder.data);
    }

    @External
    @Payable
    public void fill(SwapOrderData swapOrderData, String solverAddress) {
        SwapOrder swapOrder = new SwapOrder(swapOrderData.id, swapOrderData.emitter, swapOrderData.srcNID,
                swapOrderData.dstNID, swapOrderData.creator, swapOrderData.destinationAddress, swapOrderData.token,
                swapOrderData.amount, swapOrderData.toToken, swapOrderData.toAmount, swapOrderData.data);

        byte[] orderBytes = swapOrder.toBytes();
        byte[] orderHash = Context.hash("keccak-256", orderBytes);

        Boolean isFilled = finishedOrders.getOrDefault(orderHash, false);

        Context.require(!isFilled, "Order has already been filled");
        finishedOrders.set(orderHash, true);

        BigInteger fee = swapOrder.toAmount.multiply(protocolFee.get()).divide(BigInteger.valueOf(1000)); // add divide
                                                                                                          // by 10000
        BigInteger toAmount = swapOrder.toAmount.subtract(fee);
        _transferResult(swapOrder.destinationAddress, swapOrder.toToken, toAmount,
                fee);

        OrderFill orderFill = new OrderFill(swapOrder.id, orderBytes, solverAddress);
        if (swapOrder.srcNID.equals(swapOrder.dstNID)) {
            _resolveFill(networkId.get(), orderFill);
            return;
        }
        OrderMessage orderMessage = new OrderMessage(Constant.FILL, orderHash);
        _sendMessage(swapOrder.srcNID, Context.hash("keccak-256", orderMessage.toBytes()));
        OrderFilled(swapOrder.id, swapOrder.srcNID);
    }

    @External
    public void cancel(BigInteger id) {
        SwapOrder order = orders.get(id);
        if (order == null) {
            Context.revert("Order already has been cancelled");
        }
        Context.require(Address.fromString(order.creator).equals(Context.getCaller()),
                "Only creator can cancel this order");

        if (order.srcNID.equals(order.dstNID)) {
            _resolveCancel(nativeAddress.get().toString(), order.toBytes());
            return;
        }

        Cancel cancel = new Cancel();
        cancel.orderBytes = order.toBytes();

        OrderMessage _msg = new OrderMessage(Constant.CANCEL, cancel.orderBytes);
        _sendMessage(order.dstNID, _msg.toBytes());
    }

    @External
    public void recvMessage(String srcNetwork, BigInteger _connSn, byte[] _msg) {
        _recvMessage(srcNetwork, _connSn);

        OrderMessage orderMessage = OrderMessage.fromBytes(_msg);

        if (orderMessage.messageType.equals(Constant.FILL)) {
            OrderFill _fill = OrderFill.fromBytes(orderMessage.message);
            _resolveFill(srcNetwork, _fill);
        } else if (orderMessage.messageType.equals(Constant.CANCEL)) {
            Cancel _cancel = Cancel.fromBytes(orderMessage.message);
            _resolveCancel(srcNetwork, _cancel.orderBytes);
        }
    }

    void _resolveCancel(String srcNetwork, byte[] orderBytes) {
        byte[] orderHash = Context.hash("keccak-256", orderBytes);
        if (finishedOrders.getOrDefault(orderHash, false)) {
            return;
        }

        SwapOrder order = SwapOrder.fromBytes(orderBytes);

        Context.require(order.srcNID.equals(srcNetwork), "Invalid Network");

        finishedOrders.set(orderHash, true);

        OrderFill _fill = new OrderFill(order.id, orderBytes, order.creator);

        OrderMessage _msg = new OrderMessage(Constant.FILL, _fill.toBytes());

        _sendMessage(order.srcNID, _msg.toBytes());

        OrderCancelled(order.id, order.srcNID);
    }

    void _resolveFill(String srcNetwork, OrderFill _fill) {
        SwapOrder order = orders.get(_fill.id);
        if (order == null) {
            Context.revert("There is no order to resolve");
        }

        Context.require(Arrays.equals(Context.hash("keccak-256", order.toBytes()), Context.hash("keccak-256",
                _fill.orderBytes)), "Mismatched order");
        Context.require(order.dstNID.equals(srcNetwork), "Invalid Network");

        orders.set(_fill.id, null);
        OrderClosed(_fill.id);

        Address tokenAddress = Address.fromString(order.token);
        if (tokenAddress.equals(nativeAddress.get())) {
            Context.transfer(Address.fromString(_fill.solver), order.amount);

        } else {
            Context.call(tokenAddress, "transfer", Address.fromString(_fill.solver), order.amount);
        }
    }

    void _transferResult(String _toAddress, String _toToken, BigInteger amount,
            BigInteger fee) {
        Address toAddress = Address.fromString(_toAddress);
        Address toTokenAddress = Address.fromString(_toToken);
        if (toTokenAddress.equals(nativeAddress.get())) {
            Context.require(Context.getValue().equals(amount.add(fee)), "\"Deposit amount not equal to order amount\"");
            _nativeTransfer(toAddress, amount);
            _nativeTransfer(feeHandler.get(), fee);
        } else {
            Context.call(toTokenAddress, "transferFrom", Context.getAddress(), toAddress,
                    amount);
            Context.call(toTokenAddress, "transferFrom", Context.getAddress(),
                    feeHandler.get(), fee);
        }
    }

    void _nativeTransfer(Address to, BigInteger amount) {
        Context.transfer(to, amount);
    }

    @External
    public void setPermit(PermitTransferFromData _permit) {
        permit.set(_permit);
    }

    @Payable
    public void fallback() {
    }

    @External
    public void setProtocolFee(BigInteger _protocolFee) {
        OnlyOwner();
        protocolFee.set(_protocolFee);
    }

    @External
    public BigInteger getProtocolFee() {
        return protocolFee.get();
    }

    public static void setFeeHandler(Address _feeHandler) {
        OnlyOwner();
        feeHandler.set(_feeHandler);
    }

    @External
    public Address getFeeHandler() {
        return feeHandler.get();
    }

    public static SwapOrder getOrders(BigInteger id) {
        return orders.get(id);
    }

    public static Boolean getFinishedorders(byte[] messageHash) {
        return finishedOrders.getOrDefault(messageHash, false);
    }

    static void OnlyOwner() {
        Context.require(owner.get().equals(Context.getCaller()), "Not Owner");
    }
}
