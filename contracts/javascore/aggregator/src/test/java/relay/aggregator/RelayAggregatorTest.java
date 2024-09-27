package relay.aggregator;

import java.math.BigInteger;

import org.bouncycastle.util.Arrays;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.function.Executable;

import score.Address;
import score.Context;
import score.UserRevertedException;

import com.iconloop.score.test.Account;
import com.iconloop.score.test.Score;
import com.iconloop.score.test.ServiceManager;
import com.iconloop.score.test.TestBase;

import foundation.icon.icx.KeyWallet;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.Mockito.spy;
import static org.mockito.Mockito.verify;

class RelayAggregatorTest extends TestBase {
    private final ServiceManager sm = getServiceManager();

    private KeyWallet admin;
    private Account adminAc;

    private KeyWallet relayerOne;
    private Account relayerOneAc;

    private KeyWallet relayerTwo;
    private Account relayerTwoAc;

    private KeyWallet relayerThree;
    private Account relayerThreeAc;

    private KeyWallet relayerFour;
    private Account relayerFourAc;

    private Score aggregator;
    private RelayAggregator aggregatorSpy;

    @BeforeEach
    void setup() throws Exception {
        admin = KeyWallet.create();
        adminAc = sm.getAccount(Address.fromString(admin.getAddress().toString()));

        relayerOne = KeyWallet.create();
        relayerOneAc = sm.getAccount(Address.fromString(relayerOne.getAddress().toString()));

        relayerTwo = KeyWallet.create();
        relayerTwoAc = sm.getAccount(Address.fromString(relayerTwo.getAddress().toString()));

        relayerThree = KeyWallet.create();
        relayerThreeAc = sm.getAccount(Address.fromString(relayerThree.getAddress().toString()));

        relayerFour = KeyWallet.create();
        relayerFourAc = sm.getAccount(Address.fromString(relayerFour.getAddress().toString()));

        Address[] relayers = new Address[] { relayerOneAc.getAddress(), relayerTwoAc.getAddress(),
                relayerThreeAc.getAddress() };
        aggregator = sm.deploy(adminAc, RelayAggregator.class, adminAc.getAddress(), relayers);

        aggregatorSpy = (RelayAggregator) spy(aggregator.getInstance());
        aggregator.setInstance(aggregatorSpy);
    }

    @Test
    public void testSetAdmin() {
        Account newAdminAc = sm.createAccount();
        aggregator.invoke(adminAc, "setAdmin", newAdminAc.getAddress());

        Address result = (Address) aggregator.call("getAdmin");
        assertEquals(newAdminAc.getAddress(), result);
    }

    @Test
    public void testSetAdmin_unauthorized() {
        Account normalAc = sm.createAccount();
        Account newAdminAc = sm.createAccount();

        Executable action = () -> aggregator.invoke(normalAc, "setAdmin", newAdminAc.getAddress());
        UserRevertedException e = assertThrows(UserRevertedException.class, action);

        assertEquals("Reverted(0): Unauthorized: caller is not the leader relayer", e.getMessage());
    }

    @Test
    public void testSetSignatureThreshold() {
        int threshold = 3;
        aggregator.invoke(adminAc, "setSignatureThreshold", threshold);

        Integer result = (Integer) aggregator.call("getSignatureThreshold");
        assertEquals(threshold, result);
    }

    @Test
    public void testSetSignatureThreshold_unauthorised() {
        int threshold = 3;

        Executable action = () -> aggregator.invoke(relayerOneAc, "setSignatureThreshold", threshold);
        UserRevertedException e = assertThrows(UserRevertedException.class, action);

        assertEquals("Reverted(0): Unauthorized: caller is not the leader relayer", e.getMessage());
    }

    @Test
    public void testAddRelayers() {
        Account relayerFiveAc = sm.createAccount();
        Address[] newRelayers = new Address[] { relayerFourAc.getAddress(), relayerFiveAc.getAddress() };

        aggregator.invoke(adminAc, "addRelayers", (Object) newRelayers);

        Address[] updatedRelayers = (Address[]) aggregator.call("getRelayers");

        assertTrue(updatedRelayers[updatedRelayers.length - 1].equals(relayerFiveAc.getAddress()));
        assertTrue(updatedRelayers[updatedRelayers.length - 2].equals(relayerFourAc.getAddress()));
    }

    @Test
    public void testAddRelayers_unauthorised() {
        Account relayerFiveAc = sm.createAccount();
        Address[] newRelayers = new Address[] { relayerFourAc.getAddress(), relayerFiveAc.getAddress() };

        Executable action = () -> aggregator.invoke(relayerOneAc, "addRelayers", (Object) newRelayers);
        UserRevertedException e = assertThrows(UserRevertedException.class, action);

        assertEquals("Reverted(0): Unauthorized: caller is not the leader relayer", e.getMessage());
    }

    @Test
    public void testRemoveRelayers() {
        Address[] relayerToBeRemoved = new Address[] { relayerOneAc.getAddress(),
                relayerTwoAc.getAddress() };

        aggregator.invoke(adminAc, "removeRelayers", (Object) relayerToBeRemoved);

        Address[] updatedRelayers = (Address[]) aggregator.call("getRelayers");

        Boolean removed = true;
        for (Address rlr : updatedRelayers) {
            if (rlr.equals(relayerOneAc.getAddress()) || rlr.equals(relayerTwoAc.getAddress())) {
                removed = false;
                break;
            }
        }

        assertTrue(removed);
        assertEquals(updatedRelayers[0], relayerThreeAc.getAddress());
    }

    @Test
    public void testRemoveRelayers_unauthorised() {
        Address[] relayerToBeRemoved = new Address[] { relayerOneAc.getAddress(),
                relayerTwoAc.getAddress() };

        aggregator.invoke(adminAc, "removeRelayers", (Object) relayerToBeRemoved);

        Executable action = () -> aggregator.invoke(relayerFourAc, "removeRelayers", (Object) relayerToBeRemoved);
        UserRevertedException e = assertThrows(UserRevertedException.class, action);

        assertEquals("Reverted(0): Unauthorized: caller is not the leader relayer", e.getMessage());
    }

    @Test
    public void testRegisterPacket() {
        String srcNetwork = "0x2.icon";
        String dstNetwork = "sui";
        BigInteger srcSn = BigInteger.ONE;
        BigInteger srcHeight = BigInteger.ONE;
        String contractAddress = "hxjuiod";
        byte[] data = new byte[] { 0x01, 0x02 };

        aggregator.invoke(adminAc, "registerPacket", srcNetwork, contractAddress, srcSn, srcHeight, dstNetwork, data);

        verify(aggregatorSpy).PacketRegistered(srcNetwork, contractAddress, srcSn, srcHeight, dstNetwork, data);
    }

    @Test
    public void testRegisterPacket_nullArg() {
        String srcNetwork = null;
        String dstNetwork = "sui";
        BigInteger srcSn = BigInteger.ONE;
        BigInteger srcHeight = BigInteger.ONE;
        String contractAddress = "hxjuiod";
        byte[] data = new byte[] { 0x01, 0x02 };

        Executable action = () -> aggregator.invoke(adminAc, "registerPacket", srcNetwork, contractAddress, srcSn,
                srcHeight, dstNetwork, data);

        UserRevertedException e = assertThrows(UserRevertedException.class, action);

        assertEquals("Reverted(0): srcNetwork, contractAddress, srcSn, srcHeight, dstNetwork, and data cannot be null",
                e.getMessage());
    }

    @Test
    public void testRegisterPacket_duplicate() {
        String srcNetwork = "0x2.icon";
        String dstNetwork = "sui";
        BigInteger srcSn = BigInteger.ONE;
        BigInteger srcHeight = BigInteger.ONE;
        String contractAddress = "hxjuiod";
        byte[] data = new byte[] { 0x01, 0x02 };

        aggregator.invoke(adminAc, "registerPacket", srcNetwork, contractAddress, srcSn, srcHeight, dstNetwork, data);

        Executable action = () -> aggregator.invoke(adminAc, "registerPacket", srcNetwork, contractAddress, srcSn,
                srcHeight, dstNetwork, data);
        UserRevertedException e = assertThrows(UserRevertedException.class, action);

        assertEquals("Reverted(0): Packet already exists", e.getMessage());
    }

    @Test
    public void testAcknowledgePacket() throws Exception {
        String srcNetwork = "0x2.icon";
        String dstNetwork = "sui";
        BigInteger srcSn = BigInteger.ONE;
        BigInteger srcHeight = BigInteger.ONE;
        String contractAddress = "hxjuiod";
        byte[] data = new byte[] { 0x01, 0x02 };

        aggregator.invoke(adminAc, "registerPacket", srcNetwork, contractAddress, srcSn, srcHeight, dstNetwork, data);

        byte[] dataHash = Context.hash("sha-256", data);
        byte[] sign = relayerOne.sign(dataHash);

        aggregator.invoke(relayerOneAc, "acknowledgePacket", srcNetwork, contractAddress, srcSn, sign);

        String pktID = Packet.createId(srcNetwork, contractAddress, srcSn);
        verify(aggregatorSpy).setSignature(pktID, relayerOneAc.getAddress(), sign);
    }

    @Test
    public void testAcknowledgePacket_thresholdReached() throws Exception {
        String srcNetwork = "0x2.icon";
        String dstNetwork = "sui";
        BigInteger srcSn = BigInteger.ONE;
        BigInteger srcHeight = BigInteger.ONE;
        String contractAddress = "hxjuiod";
        byte[] data = new byte[] { 0x01, 0x02 };

        aggregator.invoke(adminAc, "registerPacket", srcNetwork, contractAddress, srcSn, srcHeight, dstNetwork, data);

        byte[] dataHash = Context.hash("sha-256", data);

        byte[] signOne = relayerOne.sign(dataHash);
        aggregator.invoke(relayerOneAc, "acknowledgePacket", srcNetwork, contractAddress, srcSn, signOne);

        byte[] signTwo = relayerTwo.sign(dataHash);
        aggregator.invoke(relayerTwoAc, "acknowledgePacket", srcNetwork,
                contractAddress, srcSn, signTwo);

        byte[][] sigs = new byte[2][];
        sigs[0] = signOne;
        sigs[1] = signTwo;

        byte[] encodedSigs = RelayAggregator.serializeSignatures(sigs);
        byte[][] decodedSigs = RelayAggregator.deserializeSignatures(encodedSigs);

        assertTrue(Arrays.areEqual(signOne, decodedSigs[0]));
        assertTrue(Arrays.areEqual(signTwo, decodedSigs[1]));

        verify(aggregatorSpy).PacketAcknowledged(srcNetwork, contractAddress, srcSn, srcHeight, dstNetwork, data,
                encodedSigs);
    }

    @Test
    public void testAcknowledgePacket_unauthorized() throws Exception {
        String srcNetwork = "0x2.icon";
        String dstNetwork = "sui";
        BigInteger srcSn = BigInteger.ONE;
        BigInteger srcHeight = BigInteger.ONE;
        String contractAddress = "hxjuiod";
        byte[] data = new byte[] { 0x01, 0x02 };

        aggregator.invoke(adminAc, "registerPacket", srcNetwork, contractAddress, srcSn, srcHeight, dstNetwork, data);

        byte[] dataHash = Context.hash("sha-256", data);
        byte[] sign = relayerFour.sign(dataHash);

        Executable action = () -> aggregator.invoke(relayerFourAc, "acknowledgePacket", srcNetwork, contractAddress,
                srcSn,
                sign);

        UserRevertedException e = assertThrows(UserRevertedException.class, action);
        assertEquals("Reverted(0): Unauthorized: caller is not a registered relayer",
                e.getMessage());
    }

    @Test
    public void testAcknowledgePacket_duplicate() throws Exception {
        String srcNetwork = "0x2.icon";
        String dstNetwork = "sui";
        BigInteger srcSn = BigInteger.ONE;
        BigInteger srcHeight = BigInteger.ONE;
        String contractAddress = "hxjuiod";
        byte[] data = new byte[] { 0x01, 0x02 };

        aggregator.invoke(adminAc, "registerPacket", srcNetwork, contractAddress, srcSn, srcHeight, dstNetwork, data);

        byte[] dataHash = Context.hash("sha-256", data);
        byte[] sign = relayerOne.sign(dataHash);

        aggregator.invoke(relayerOneAc, "acknowledgePacket", srcNetwork, contractAddress, srcSn, sign);

        Executable action = () -> aggregator.invoke(relayerOneAc, "acknowledgePacket", srcNetwork, contractAddress,
                srcSn,
                sign);
        UserRevertedException e = assertThrows(UserRevertedException.class, action);
        assertEquals("Reverted(0): Signature already exists", e.getMessage());
    }

    @Test
    public void testAcknowledgePacket_packetUnregistered() throws Exception {
        String srcNetwork = "0x2.icon";
        BigInteger srcSn = BigInteger.ONE;
        String contractAddress = "hxjuiod";
        byte[] data = new byte[] { 0x01, 0x02 };

        byte[] dataHash = Context.hash("sha-256", data);
        byte[] sign = relayerOne.sign(dataHash);

        Executable action = () -> aggregator.invoke(relayerOneAc, "acknowledgePacket", srcNetwork, contractAddress,
                srcSn,
                sign);
        UserRevertedException e = assertThrows(UserRevertedException.class, action);
        assertEquals("Reverted(0): Packet not registered", e.getMessage());
    }
}
