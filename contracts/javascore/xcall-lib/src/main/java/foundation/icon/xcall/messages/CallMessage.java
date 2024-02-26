
package foundation.icon.xcall.messages;

public class CallMessage extends Message {
    public static final int TYPE = 0;
    private byte[] data;

    public CallMessage(byte[] data) {
        this.data = data;
    }

    public int getType() {
        return TYPE;
    }

    public byte[] getData() {
        return data;
    }

    public byte[] toBytes() {
        return data;
    }

    public static CallMessage fromBytes(byte[] bytes) {
        return new CallMessage(bytes);
    }
}