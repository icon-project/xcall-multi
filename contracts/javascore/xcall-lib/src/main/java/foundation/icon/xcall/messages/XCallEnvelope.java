package foundation.icon.xcall.messages;

import java.util.List;

import score.ByteArrayObjectWriter;
import score.Context;
import score.ObjectReader;
import score.ObjectWriter;
import scorex.util.ArrayList;

public class XCallEnvelope {
    public int type;
    public byte[] message;
    public String[] sources = new String[]{};
    public String[] destinations = new String[]{};;

    
    public XCallEnvelope(int type, byte[] message, String[] sources, String[] destinations) {
        this.type = type;
        this.message = message;
        this.sources = sources;
        this.destinations = destinations;
    }

    public XCallEnvelope(Message message, String[] sources, String[] destinations) {
        this.type = message.getType();
        this.message = message.toBytes();
        this.sources = sources;
        this.destinations = destinations;
    }

    public XCallEnvelope(Message message) {
        this.type = message.getType();
        this.message = message.toBytes();
    }

    public int getType() {
        return type;
    }

    public byte[] getMessage() {
        return message;
    }

    public String[] getSources() {
        return sources;
    }

    public String[] getDestinations() {
        return destinations;
    }

    public static void writeObject(ObjectWriter w, XCallEnvelope envelope) {
        w.beginList(3);
        w.write(envelope.type);
        w.write(envelope.message);
        w.beginList(envelope.sources.length);
        for(String protocol : envelope.sources) {
            w.write(protocol);
        }
        w.end();
        w.beginList(envelope.destinations.length);
        for(String protocol : envelope.destinations) {
            w.write(protocol);
        }
        w.end();
        w.end();
    }

    public static XCallEnvelope readObject(ObjectReader r) {
        r.beginList();
        XCallEnvelope call = new XCallEnvelope(
            r.readInt(),
            r.readByteArray(),
            readProtocols(r),
            readProtocols(r)
        );
        return call;
    }

    private static String[] readProtocols(ObjectReader r) {
        r.beginList();
        List<String> protocolsList = new ArrayList<>();
        while(r.hasNext()) {
            protocolsList.add(r.readString());
        }
        int size = protocolsList.size();
        String[] protocols = new String[size];
        for(int i=0; i < size; i++) {
            protocols[i] = protocolsList.get(i);
        }
        r.end();
        return protocols;
    }

    public byte[] toBytes() {
        ByteArrayObjectWriter writer = Context.newByteArrayObjectWriter("RLPn");
        XCallEnvelope.writeObject(writer, this);
        return writer.toByteArray();
    }

    public static XCallEnvelope fromBytes(byte[] bytes) {
        ObjectReader reader = Context.newByteArrayObjectReader("RLPn", bytes);
        return readObject(reader);
    }

}
