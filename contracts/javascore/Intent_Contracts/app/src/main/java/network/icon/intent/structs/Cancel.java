package network.icon.intent.structs;

import score.ByteArrayObjectWriter;
import score.Context;
import score.ObjectReader;
import score.ObjectWriter;

public class Cancel {
    public byte[] orderBytes;

    public Cancel() {
    }

    public static void writeObject(ObjectWriter writer, Cancel obj) {
        obj.writeObject(writer);
    }

    public void writeObject(ObjectWriter writer) {
        writer.beginList(1);
        writer.write(this.orderBytes);
        writer.end();
    }

    public static Cancel readObject(ObjectReader reader) {
        Cancel obj = new Cancel();
        reader.beginList();
        obj.orderBytes = reader.readByteArray();
        reader.end();
        return obj;
    }

    public static Cancel fromBytes(byte[] bytes) {
        ObjectReader reader = Context.newByteArrayObjectReader("RLPn", bytes);
        return readObject(reader);
    }

    public byte[] toBytes() {
        ByteArrayObjectWriter writer = Context.newByteArrayObjectWriter("RLPn");
        Cancel.writeObject(writer, this);
        return writer.toByteArray();
    }

}
