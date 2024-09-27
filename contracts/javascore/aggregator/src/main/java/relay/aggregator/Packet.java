package relay.aggregator;

import java.math.BigInteger;

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
    private final String contractAddress;

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
     *                                  {@code contractAddress}, {@code srcSn},
     *                                  {@code srcHeight},
     *                                  {@code dstNetwork}, or {@code data} is
     *                                  {@code null}.
     */
    public Packet(String srcNetwork, String contractAddress, BigInteger srcSn, BigInteger srcHeight, String dstNetwork,
            byte[] data) {
        Boolean isIllegalArg = srcNetwork == null || contractAddress == null || contractAddress == null || srcSn == null
                || srcHeight == null || dstNetwork == null || data == null;
        Context.require(!isIllegalArg,
                "srcNetwork, contractAddress, srcSn, srcHeight, dstNetwork, and data cannot be null");
        if (isIllegalArg) {
        }
        this.srcNetwork = srcNetwork;
        this.contractAddress = contractAddress;
        this.srcSn = srcSn;
        this.srcHeight = srcHeight;
        this.dstNetwork = dstNetwork;
        this.data = data;
    }

    public String getId() {
        return createId(this.srcNetwork, this.contractAddress, this.srcSn);
    }

    public static String createId(String srcNetwork, String contractAddress, BigInteger srcSn) {
        return srcNetwork + "/" + contractAddress + "/" + srcSn.toString();
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
     * @return the contract address.
     */
    public String getContractAddress() {
        return contractAddress;
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
     * Returns a copy of the data associated with this packet.
     *
     * @return a byte array containing the packet data.
     */
    public byte[] getData() {
        return data;
    }

    public static void writeObject(ObjectWriter w, Packet p) {
        w.beginList(6);
        w.write(p.srcNetwork);
        w.write(p.contractAddress);
        w.write(p.srcSn);
        w.write(p.srcHeight);
        w.write(p.dstNetwork);
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
                r.readNullable(byte[].class));
        r.end();
        return p;
    }
}
