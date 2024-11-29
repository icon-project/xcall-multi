package relay.aggregator;

import java.math.BigInteger;
import java.util.Arrays;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.function.Executable;

import score.Address;
import score.Context;
import score.UserRevertedException;
import scorex.util.HashSet;

import com.iconloop.score.test.Account;
import com.iconloop.score.test.Score;
import com.iconloop.score.test.ServiceManager;
import com.iconloop.score.test.TestBase;

import foundation.icon.icx.KeyWallet;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
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

        aggregator = sm.deploy(adminAc, RelayAggregator.class, adminAc.getAddress());

        Address[] relayers = new Address[] { adminAc.getAddress(), relayerOneAc.getAddress(), relayerTwoAc.getAddress(),
                relayerThreeAc.getAddress() };

        aggregator.invoke(adminAc, "setRelayers", (Object) relayers, 2);

        aggregatorSpy = (RelayAggregator) spy(aggregator.getInstance());
        aggregator.setInstance(aggregatorSpy);
    }

    @Test
    public void testSetAdmin() {
        Address oldAdmin = (Address) aggregator.call("getAdmin");

        Account newAdminAc = sm.createAccount();
        aggregator.invoke(adminAc, "setAdmin", newAdminAc.getAddress());

        Address newAdmin = (Address) aggregator.call("getAdmin");
        assertEquals(newAdminAc.getAddress(), newAdmin);

        Address[] relayers = (Address[]) aggregator.call("getRelayers");

        boolean containsNewAdmin = Arrays.asList(relayers).contains(newAdmin);
        boolean containsOldAdmin = Arrays.asList(relayers).contains(oldAdmin);

        assertTrue(containsNewAdmin);
        assertFalse(containsOldAdmin);
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

        Executable action = () -> aggregator.invoke(relayerOneAc,
                "setSignatureThreshold", threshold);
        UserRevertedException e = assertThrows(UserRevertedException.class, action);

        assertEquals("Reverted(0): Unauthorized: caller is not the leader relayer",
                e.getMessage());
    }

    @Test
    public void testSetRelayers() {
        Account relayerFiveAc = sm.createAccount();
        Address[] newRelayers = new Address[] { relayerThreeAc.getAddress(), relayerFourAc.getAddress(),
                relayerFiveAc.getAddress() };

        Integer threshold = 3;
        aggregator.invoke(adminAc, "setRelayers", (Object) newRelayers, threshold);

        Address[] updatedRelayers = (Address[]) aggregator.call("getRelayers");

        Address[] expectedRelayers = new Address[] { adminAc.getAddress(), relayerThreeAc.getAddress(),
                relayerFourAc.getAddress(),
                relayerFiveAc.getAddress() };

        HashSet<Address> updatedRelayersSet = new HashSet<Address>();
        for (Address rlr : updatedRelayers) {
            updatedRelayersSet.add(rlr);
        }

        HashSet<Address> expectedRelayersSet = new HashSet<Address>();
        for (Address rlr : expectedRelayers) {
            expectedRelayersSet.add(rlr);
        }

        assertEquals(expectedRelayersSet, updatedRelayersSet);

        Integer result = (Integer) aggregator.call("getSignatureThreshold");
        assertEquals(threshold, result);
    }

    @Test
    public void testSetRelayers_unauthorized() {
        Account relayerFiveAc = sm.createAccount();
        Address[] newRelayers = new Address[] { relayerThreeAc.getAddress(), relayerFourAc.getAddress(),
                relayerFiveAc.getAddress() };

        Integer threshold = 3;
        Executable action = () -> aggregator.invoke(relayerOneAc, "setRelayers",
                (Object) newRelayers, threshold);
        UserRevertedException e = assertThrows(UserRevertedException.class, action);

        assertEquals("Reverted(0): Unauthorized: caller is not the leader relayer",
                e.getMessage());

    }

    @Test
    public void testSetRelayers_invalidThreshold() {
        Account relayerFiveAc = sm.createAccount();
        Address[] newRelayers = new Address[] { relayerThreeAc.getAddress(), relayerFourAc.getAddress(),
                relayerFiveAc.getAddress() };

        Integer threshold = 5;
        Executable action = () -> aggregator.invoke(adminAc, "setRelayers",
                (Object) newRelayers, threshold);
        UserRevertedException e = assertThrows(UserRevertedException.class, action);

        assertEquals("Reverted(0): threshold value should be at least 1 and not greater than relayers size",
                e.getMessage());

    }

    @Test
    public void testSetRelayers_invalidThresholdZero() {
        Account relayerFiveAc = sm.createAccount();
        Address[] newRelayers = new Address[] { relayerThreeAc.getAddress(), relayerFourAc.getAddress(),
                relayerFiveAc.getAddress() };

        Integer threshold = 0;
        Executable action = () -> aggregator.invoke(adminAc, "setRelayers",
                (Object) newRelayers, threshold);
        UserRevertedException e = assertThrows(UserRevertedException.class, action);

        assertEquals("Reverted(0): threshold value should be at least 1 and not greater than relayers size",
                e.getMessage());

    }

    @Test
    public void testPacketSubmitted_true() throws Exception {
        String srcNetwork = "0x2.icon";
        String dstNetwork = "sui";
        BigInteger srcSn = BigInteger.ONE;
        BigInteger srcHeight = BigInteger.ONE;
        String srcContractAddress = "hxjuiod";
        String dstContractAddress = "hxjuiod";
        byte[] data = new byte[] { 0x01, 0x02 };

        aggregator.invoke(adminAc, "setSignatureThreshold", 2);

        byte[] dataHash = Context.hash("sha-256", data);
        byte[] sign = relayerOne.sign(dataHash);

        aggregator.invoke(relayerOneAc, "submitPacket", srcNetwork,
                srcContractAddress, srcSn, srcHeight, dstNetwork,
                dstContractAddress, data,
                sign);

        boolean submitted = (boolean) aggregator.call("packetSubmitted",
                relayerOneAc.getAddress(), srcNetwork,
                srcContractAddress, srcSn);
        assertEquals(submitted, true);
    }

    @Test
    public void testPacketSubmitted_false() throws Exception {
        String srcNetwork = "0x2.icon";
        BigInteger srcSn = BigInteger.ONE;
        String srcContractAddress = "hxjuiod";

        boolean submitted = (boolean) aggregator.call("packetSubmitted",
                relayerOneAc.getAddress(), srcNetwork,
                srcContractAddress, srcSn);
        assertEquals(submitted, false);
    }

    @Test
    public void testSubmitPacket() throws Exception {
        String srcNetwork = "0x2.icon";
        String dstNetwork = "sui";
        BigInteger srcSn = BigInteger.ONE;
        BigInteger srcHeight = BigInteger.ONE;
        String srcContractAddress = "hxjuiod";
        String dstContractAddress = "hxjuiod";
        byte[] data = new byte[] { 0x01, 0x02 };

        aggregator.invoke(adminAc, "setSignatureThreshold", 2);

        byte[] dataHash = Context.hash("sha-256", data);
        byte[] sign = relayerOne.sign(dataHash);

        aggregator.invoke(relayerOneAc, "submitPacket", srcNetwork,
                srcContractAddress, srcSn, srcHeight, dstNetwork,
                dstContractAddress, data,
                sign);

        String pktID = Packet.createId(srcNetwork, srcContractAddress, srcSn);
        verify(aggregatorSpy).PacketRegistered(srcNetwork, srcContractAddress, srcSn,
                srcHeight, dstNetwork,
                dstContractAddress, data);
        verify(aggregatorSpy).setSignature(pktID, relayerOneAc.getAddress(), sign);
    }

    @Test
    public void testSubmitPacket_thresholdReached() throws Exception {
        String srcNetwork = "0x2.icon";
        String dstNetwork = "sui";
        BigInteger srcSn = BigInteger.ONE;
        BigInteger srcHeight = BigInteger.ONE;
        String srcContractAddress = "hxjuiod";
        String dstContractAddress = "hxjuiod";
        byte[] data = new byte[] { 0x01, 0x02 };

        aggregator.invoke(adminAc, "setSignatureThreshold", 2);

        byte[] dataHash = Context.hash("sha-256", data);

        byte[] signAdmin = admin.sign(dataHash);
        aggregator.invoke(adminAc, "submitPacket", srcNetwork, srcContractAddress,
                srcSn, srcHeight, dstNetwork,
                dstContractAddress, data,
                signAdmin);

        byte[] signOne = relayerOne.sign(dataHash);
        aggregator.invoke(relayerOneAc, "submitPacket", srcNetwork,
                srcContractAddress, srcSn, srcHeight, dstNetwork,
                dstContractAddress,
                data,
                signOne);

        byte[][] sigs = new byte[2][];
        sigs[0] = signAdmin;
        sigs[1] = signOne;

        byte[] encodedSigs = RelayAggregator.serializeSignatures(sigs);
        byte[][] decodedSigs = RelayAggregator.deserializeSignatures(encodedSigs);

        assertArrayEquals(signAdmin, decodedSigs[0]);
        assertArrayEquals(signOne, decodedSigs[1]);

        verify(aggregatorSpy).PacketAcknowledged(srcNetwork, srcContractAddress,
                srcSn, srcHeight, dstNetwork,
                dstContractAddress, data,
                encodedSigs);
    }

    @Test
    public void testSubmitPacket_unauthorized() throws Exception {
        String srcNetwork = "0x2.icon";
        String dstNetwork = "sui";
        BigInteger srcSn = BigInteger.ONE;
        BigInteger srcHeight = BigInteger.ONE;
        String srcContractAddress = "hxjuiod";
        String dstContractAddress = "hxjuiod";
        byte[] data = new byte[] { 0x01, 0x02 };

        byte[] dataHash = Context.hash("sha-256", data);
        byte[] sign = relayerFour.sign(dataHash);

        Executable action = () -> aggregator.invoke(relayerFourAc, "submitPacket",
                srcNetwork, srcContractAddress,
                srcSn,
                srcHeight, dstNetwork, dstContractAddress, data, sign);

        UserRevertedException e = assertThrows(UserRevertedException.class, action);
        assertEquals("Reverted(0): Unauthorized: caller is not a registered relayer",
                e.getMessage());
    }

    @Test
    public void testSubmitPacket_duplicate() throws Exception {
        String srcNetwork = "0x2.icon";
        String dstNetwork = "sui";
        BigInteger srcSn = BigInteger.ONE;
        BigInteger srcHeight = BigInteger.ONE;
        String srcContractAddress = "hxjuiod";
        String dstContractAddress = "hxjuiod";
        byte[] data = new byte[] { 0x01, 0x02 };

        aggregator.invoke(adminAc, "setSignatureThreshold", 2);

        byte[] dataHash = Context.hash("sha-256", data);
        byte[] sign = relayerOne.sign(dataHash);

        aggregator.invoke(relayerOneAc, "submitPacket", srcNetwork,
                srcContractAddress, srcSn, srcHeight, dstNetwork,
                dstContractAddress, data, sign);

        Executable action = () -> aggregator.invoke(relayerOneAc, "submitPacket",
                srcNetwork, srcContractAddress, srcSn,
                srcHeight, dstNetwork, dstContractAddress,
                data, sign);
        ;
        UserRevertedException e = assertThrows(UserRevertedException.class, action);
        assertEquals("Reverted(0): Signature already exists", e.getMessage());
    }
}
