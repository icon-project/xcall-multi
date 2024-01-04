package foundation.icon.xcall.messages;

public abstract class Message {
    public abstract int getType();

    public abstract byte[] toBytes();
}
