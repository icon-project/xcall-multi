package network.icon.intent.structs;

import java.math.BigInteger;

import score.ByteArrayObjectWriter;
import score.Context;
import score.ObjectReader;
import score.ObjectWriter;

public class OrderFill {
    public BigInteger id; // ID of the order being filled
    public byte[] orderBytes; // rlp of the order
    public String solver;// Address of the solver that fills the order

    public OrderFill(BigInteger id, byte[] orderBytes, String solver) {
        this.id = id;
        this.orderBytes = orderBytes;
        this.solver = solver;
    }

    private OrderFill() {
    }

    public static void writeObject(ObjectWriter writer, OrderFill obj) {
        obj.writeObject(writer);
    }

    public void writeObject(ObjectWriter writer) {
        writer.beginList(3);
        writer.write(this.id);
        writer.write(this.orderBytes);
        writer.write(this.solver);
        writer.end();
    }

    public static OrderFill readObject(ObjectReader reader) {
        OrderFill obj = new OrderFill();
        reader.beginList();
        obj.id = reader.readBigInteger();
        obj.orderBytes = reader.readByteArray();
        obj.solver = reader.readString();
        reader.end();
        return obj;
    }

    public byte[] toBytes() {
        ByteArrayObjectWriter writer = Context.newByteArrayObjectWriter("RLPn");
        OrderFill.writeObject(writer, this);
        return writer.toByteArray();
    }

    public static OrderFill fromBytes(byte[] bytes) {
        ObjectReader reader = Context.newByteArrayObjectReader("RLPn", bytes);
        return readObject(reader);
    }

    public BigInteger getId() {
        return id;
    }

    public void setId(BigInteger id) {
        this.id = id;
    }

    public byte[] getOrderBytes() {
        return orderBytes;
    }

    public void setOrderBytes(byte[] orderBytes) {
        this.orderBytes = orderBytes;
    }

    public String getSolver() {
        return solver;
    }

    public void setSolver(String solver) {
        this.solver = solver;
    }
}
