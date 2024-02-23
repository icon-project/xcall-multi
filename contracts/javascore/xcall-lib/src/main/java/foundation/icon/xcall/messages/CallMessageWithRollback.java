package foundation.icon.xcall.messages;

import score.ByteArrayObjectWriter;
import score.Context;
import score.ObjectReader;
import score.ObjectWriter;

public class CallMessageWithRollback extends Message {
    public static final int TYPE = 1;
    private byte[] data;
    private byte[] rollback;
    public CallMessageWithRollback(byte[] data, byte[] rollback) {
        this.data = data;
        this.rollback = rollback;
    }

    public int getType() {
        return TYPE;
    }

    public byte[] getData() {
        return data;
    }

    public byte[] getRollback() {
        return rollback;
    }

    public static void writeObject(ObjectWriter w, CallMessageWithRollback call) {
        w.beginList(2);
        w.write(call.data);
        w.write(call.rollback);
        w.end();
    }

    public static CallMessageWithRollback readObject(ObjectReader r) {
        r.beginList();
        CallMessageWithRollback call = new CallMessageWithRollback(
            r.readByteArray(),
            r.readByteArray()
        );
        return call;
    }

    public byte[] toBytes() {
        ByteArrayObjectWriter writer = Context.newByteArrayObjectWriter("RLPn");
        CallMessageWithRollback.writeObject(writer, this);
        return writer.toByteArray();
    }

    public static CallMessageWithRollback fromBytes(byte[] bytes) {
        ObjectReader reader = Context.newByteArrayObjectReader("RLPn", bytes);
        return readObject(reader);
    }
}
