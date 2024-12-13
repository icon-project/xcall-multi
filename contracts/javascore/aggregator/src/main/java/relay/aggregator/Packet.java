package relay.aggregator;

import java.math.BigInteger;

import score.ByteArrayObjectWriter;
import score.Context;
import score.ObjectReader;
import score.ObjectWriter;

public class Packet {

    /**
     * The ID of the source network (chain) from where the packet originated.
     */
    private final String srcNetwork;

    /**
     * The contract address on the source network (chain).
     */
    private final String srcContractAddress;

    /**
     * The sequence number of the packet in the source network (chain).
     */
    private final BigInteger srcSn;

    /**
     * The source height of the packet in the source network (chain).
     */
    private final BigInteger srcHeight;

    /**
     * The ID of the destination network (chain) where the packet is being sent.
     */
    private final String dstNetwork;

    /**
     * The contract address on the destination network (chain).
     */
    private final String dstContractAddress;

    /**
     * The payload data associated with this packet.
     */
    private final byte[] data;

    /**
     * Constructs a new {@code Packet} object with the specified {@code PacketID},
     * destination network, and data.
     * All parameters must be non-null.
     *
     * @param id         the unique identifier for the packet.
     * @param dstNetwork the ID of the destination network (chain).
     * @param data       the payload data for this packet.
     * @throws IllegalArgumentException if {@code srcNetwork},
     *                                  {@code srcContractAddress}, {@code srcSn},
     *                                  {@code srcHeight},
     *                                  {@code dstNetwork},
     *                                  {@code dstContractAddress}, or {@code data}
     *                                  is
     *                                  {@code null}.
     */
    public Packet(String srcNetwork, String srcContractAddress, BigInteger srcSn, BigInteger srcHeight,
            String dstNetwork, String dstContractAddress,
            byte[] data) {
        Boolean isIllegalArg = srcNetwork == null || srcContractAddress == null || srcSn == null
                || srcHeight == null || dstNetwork == null || dstContractAddress == null || data == null;
        Context.require(!isIllegalArg,
                "srcNetwork, contractAddress, srcSn, srcHeight, dstNetwork, and data cannot be null");
        this.srcNetwork = srcNetwork;
        this.srcContractAddress = srcContractAddress;
        this.srcSn = srcSn;
        this.srcHeight = srcHeight;
        this.dstNetwork = dstNetwork;
        this.dstContractAddress = dstContractAddress;
        this.data = data;
    }

    public byte[] getId() {
        return Context.hash("sha-256", this.toBytes());
    }

    /**
     * Returns the source network (chain) from where the packet originated.
     *
     * @return the source network ID.
     */
    public String getSrcNetwork() {
        return srcNetwork;
    }

    /**
     * Returns the contract address on the source network (chain).
     *
     * @return the source contract address.
     */
    public String getSrcContractAddress() {
        return srcContractAddress;
    }

    /**
     * Returns the sequence number of the packet in the source network (chain).
     *
     * @return the sequence number.
     */
    public BigInteger getSrcSn() {
        return srcSn;
    }

    /**
     * Returns the height of the packet in the source network (chain).
     *
     * @return the source height.
     */
    public BigInteger getSrcHeight() {
        return srcHeight;
    }

    /**
     * Returns the destination network (chain) where the packet is being sent.
     *
     * @return the destination network ID.
     */
    public String getDstNetwork() {
        return dstNetwork;
    }

    /**
     * Returns the contract address on the destination network (chain).
     *
     * @return the destination contract address.
     */
    public String getDstContractAddress() {
        return dstContractAddress;
    }

    /**
     * Returns a copy of the data associated with this packet.
     *
     * @return a byte array containing the packet data.
     */
    public byte[] getData() {
        return data;
    }

    public static void writeObject(ObjectWriter w, Packet p) {
        w.beginList(7);
        w.write(p.srcNetwork);
        w.write(p.srcContractAddress);
        w.write(p.srcSn);
        w.write(p.srcHeight);
        w.write(p.dstNetwork);
        w.write(p.dstContractAddress);
        w.writeNullable(p.data);
        w.end();
    }

    public static Packet readObject(ObjectReader r) {
        r.beginList();
        Packet p = new Packet(
                r.readString(),
                r.readString(),
                r.readBigInteger(),
                r.readBigInteger(),
                r.readString(),
                r.readString(),
                r.readNullable(byte[].class));
        r.end();
        return p;
    }

    public byte[] toBytes() {
        ByteArrayObjectWriter writer = Context.newByteArrayObjectWriter("RLPn");
        Packet.writeObject(writer, this);
        return writer.toByteArray();
    }
}
