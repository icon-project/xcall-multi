package network.icon.intent.structs;

import java.math.BigInteger;

import score.ByteArrayObjectWriter;
import score.Context;
import score.ObjectReader;
import score.ObjectWriter;

public class SwapOrder {
    public BigInteger id; // unique ID
    public String emitter;// Address of emitter contract
    public String srcNID; // Source Network ID
    public String dstNID; // Destination Network ID
    public String creator; // The user who created the order
    public String destinationAddress; // Destination address on the destination network
    public String token; // Token to be swapped
    public BigInteger amount; // Amount of the token to be swapped
    public String toToken; // Token to receive on the destination network
    public BigInteger toAmount; // Minimum amount of the toToken to receive
    public String data; // Additional data (if any) for future use (is this the right type?)

    public SwapOrder(BigInteger id, String emitter, String srcNID, String dstNID, String creator,
            String destinationAddress, String token, BigInteger amount, String toToken, BigInteger toAmount,
            String data) {
        this.id = id;
        this.emitter = emitter;
        this.srcNID = srcNID;
        this.dstNID = dstNID;
        this.creator = creator;
        this.destinationAddress = destinationAddress;
        this.token = token;
        this.amount = amount;
        this.toToken = toToken;
        this.toAmount = toAmount;
        this.data = data;
    }

    private SwapOrder() {
    }

    public static void writeObject(ObjectWriter writer, SwapOrder obj) {
        obj.writeObject(writer);
    }

    // add read object method
    public static SwapOrder readObject(ObjectReader reader) {
        SwapOrder obj = new SwapOrder();
        reader.beginList();
        obj.id = reader.readBigInteger();
        obj.emitter = reader.readString();
        obj.srcNID = reader.readString();
        obj.dstNID = reader.readString();
        obj.creator = reader.readString();
        obj.destinationAddress = reader.readString();
        obj.token = reader.readString();
        obj.amount = reader.readBigInteger();
        obj.toToken = reader.readString();
        obj.toAmount = reader.readBigInteger();
        obj.data = reader.readString();
        reader.end();
        return obj;
    }

    public void writeObject(ObjectWriter writer) {
        writer.beginList(11);
        writer.write(this.id);
        writer.write(this.emitter);
        writer.write(this.srcNID);
        writer.write(this.dstNID);
        writer.write(this.creator);
        writer.write(this.destinationAddress);
        writer.write(this.token);
        writer.write(this.amount);
        writer.write(this.toToken);
        writer.write(this.toAmount);
        writer.write(this.data);
        writer.end();
    }

    public byte[] toBytes() {
        ByteArrayObjectWriter writer = Context.newByteArrayObjectWriter("RLPn");
        SwapOrder.writeObject(writer, this);
        return writer.toByteArray();
    }

    public static SwapOrder fromBytes(byte[] bytes) {
        ObjectReader reader = Context.newByteArrayObjectReader("RLPn", bytes);
        return readObject(reader);
    }

    public BigInteger getId() {
        return id;
    }

    public void setId(BigInteger id) {
        this.id = id;
    }

    public String getEmitter() {
        return emitter;
    }

    public void setEmitter(String emitter) {
        this.emitter = emitter;
    }

    public String getSrcNID() {
        return srcNID;
    }

    public void setSrcNID(String srcNID) {
        this.srcNID = srcNID;
    }

    public String getDstNID() {
        return dstNID;
    }

    public void setDstNID(String dstNID) {
        this.dstNID = dstNID;
    }

    public String getCreator() {
        return creator;
    }

    public void setCreator(String creator) {
        this.creator = creator;
    }

    public String getDestinationAddress() {
        return destinationAddress;
    }

    public void setDestinationAddress(String destinationAddress) {
        this.destinationAddress = destinationAddress;
    }

    public String getToken() {
        return token;
    }

    public void setToken(String token) {
        this.token = token;
    }

    public BigInteger getAmount() {
        return amount;
    }

    public void setAmount(BigInteger amount) {
        this.amount = amount;
    }

    public String getToToken() {
        return toToken;
    }

    public void setToToken(String toToken) {
        this.toToken = toToken;
    }

    public BigInteger getToAmount() {
        return toAmount;
    }

    public void setToAmount(BigInteger toAmount) {
        this.toAmount = toAmount;
    }

    public String getData() {
        return data;
    }

    public void setData(String data) {
        this.data = data;
    }
}
