package network.icon.intent.structs;

import java.math.BigInteger;

import network.icon.intent.utils.TokenPermissionsData;
import score.ByteArrayObjectWriter;
import score.Context;
import score.ObjectReader;
import score.ObjectWriter;

public class PermitTransferFrom {
    public TokenPermissionsData permitted;
    public BigInteger nonce;
    public BigInteger deadline;

    private PermitTransferFrom() {
    }

    public PermitTransferFrom(TokenPermissionsData _permitted, BigInteger _nonce, BigInteger _deadline) {
        this.permitted = _permitted;
        this.nonce = _nonce;
        this.deadline = _deadline;
    }

    public static void writeObject(ObjectWriter writer, PermitTransferFrom obj) {
        obj.writeObject(writer);
    }

    public static PermitTransferFrom readObject(ObjectReader reader) {
        PermitTransferFrom obj = new PermitTransferFrom();
        reader.beginList();
        obj.permitted = reader.read(TokenPermissionsData.class);
        obj.nonce = reader.readBigInteger();
        obj.deadline = reader.readBigInteger();
        reader.end();
        return obj;
    }

    public void writeObject(ObjectWriter writer) {
        writer.beginList(3);
        writer.write(this.permitted);
        writer.write(this.nonce);
        writer.write(this.deadline);
        writer.end();
    }

    public byte[] toBytes() {
        ByteArrayObjectWriter writer = Context.newByteArrayObjectWriter("RLPn");
        PermitTransferFrom.writeObject(writer, this);
        return writer.toByteArray();
    }

    public static PermitTransferFrom fromBytes(byte[] bytes) {
        ObjectReader reader = Context.newByteArrayObjectReader("RLPn", bytes);
        return readObject(reader);
    }

    public TokenPermissionsData getPermitted() {
        return permitted;
    }

    public void setPermitted(TokenPermissionsData permitted) {
        this.permitted = permitted;
    }

    public BigInteger getNonce() {
        return nonce;
    }

    public void setNonce(BigInteger nonce) {
        this.nonce = nonce;
    }

    public BigInteger getDeadline() {
        return deadline;
    }

    public void setDeadline(BigInteger deadline) {
        this.deadline = deadline;
    }
}
