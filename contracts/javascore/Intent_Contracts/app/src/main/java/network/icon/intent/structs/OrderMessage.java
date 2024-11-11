package network.icon.intent.structs;

import java.math.BigInteger;

import score.ByteArrayObjectWriter;
import score.Context;
import score.ObjectReader;
import score.ObjectWriter;

public class OrderMessage {
    public BigInteger messageType;
    public byte[] message;

    public OrderMessage(BigInteger messageType, byte[] message) {
        this.messageType = messageType;
        this.message = message;
    }

    private OrderMessage() {
    }

    public static void writeObject(ObjectWriter writer, OrderMessage obj) {
        obj.writeObject(writer);
    }

    public void writeObject(ObjectWriter writer) {
        writer.beginList(2);
        writer.write(this.messageType);
        writer.write(this.message);
        writer.end();
    }

    public static OrderMessage readObject(ObjectReader reader) {
        OrderMessage obj = new OrderMessage();
        reader.beginList();
        obj.messageType = reader.readBigInteger();
        obj.message = reader.readByteArray();
        reader.end();
        return obj;
    }

    public byte[] toBytes() {
        ByteArrayObjectWriter writer = Context.newByteArrayObjectWriter("RLPn");
        OrderMessage.writeObject(writer, this);
        return writer.toByteArray();
    }

    public static OrderMessage fromBytes(byte[] bytes) {
        ObjectReader reader = Context.newByteArrayObjectReader("RLPn", bytes);
        return readObject(reader);
    }

    public BigInteger getMessageType() {
        return messageType;
    }

    public void setMessageType(BigInteger messageType) {
        this.messageType = messageType;
    }

    public byte[] getMessage() {
        return message;
    }

    public void setMessage(byte[] message) {
        this.message = message;
    }
}
