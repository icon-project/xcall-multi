package relay.aggregator;

import java.math.BigInteger;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.function.Executable;

import score.Address;
import score.Context;
import score.UserRevertedException;
import score.DictDB;

import com.iconloop.score.test.Account;
import com.iconloop.score.test.Score;
import com.iconloop.score.test.ServiceManager;
import com.iconloop.score.test.TestBase;
import foundation.icon.icx.KeyWallet;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.spy;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;

class RelayAggregatorTest extends TestBase {
    private final ServiceManager sm = getServiceManager();

    private  KeyWallet admin;
    private  Account adminAc;
    
    private  KeyWallet relayerOne;
    private  Account relayerOneAc;

    private  KeyWallet relayerTwo;
    private  Account relayerTwoAc;

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

        Address[] relayers = new Address[]{relayerOneAc.getAddress(), relayerTwoAc.getAddress()};
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
    @SuppressWarnings("unchecked")
    public void testRegisterPacket() {
        String nid = "0x2.icon";
        BigInteger sn = BigInteger.ONE;
        byte[] data = new byte[]{0x01, 0x02};
    
        DictDB<BigInteger, byte[]> mockDictDB = mock(DictDB.class);

        when(aggregatorSpy.getPackets(nid)).thenReturn(mockDictDB);

        aggregator.invoke(adminAc, "registerPacket", nid, sn, data);

        verify(mockDictDB).set(sn, data);
    }

    @Test
    public void testRegisterPacket_duplicate() {
        String nid = "0x2.icon";
        BigInteger sn = BigInteger.ONE;
        byte[] data = new byte[]{0x01, 0x02};

        aggregator.invoke(adminAc, "registerPacket", nid, sn, data);

        Executable action = () -> aggregator.invoke(adminAc, "registerPacket", nid, sn, data);
        UserRevertedException e = assertThrows(UserRevertedException.class, action);
    
        assertEquals("Reverted(0): Packet already exists", e.getMessage());
    }

    @Test
    public void testSubmitSignature() throws Exception {
        String nid = "0x2.icon";
        BigInteger sn = BigInteger.ONE;
        byte[] data = new byte[]{0x01, 0x02};

        aggregator.invoke(adminAc, "registerPacket", nid, sn, data);

        byte[] dataHash = Context.hash("sha-256", data);
        byte[] sign = relayerOne.sign(dataHash);;

        aggregator.invoke(relayerOneAc, "submitSignature", nid, sn, sign);

        verify(aggregatorSpy).setSignature(nid, sn, relayerOneAc.getAddress(), sign);
    }

    @Test
    public void testSubmitSignature_duplicate() throws Exception {
        String nid = "0x2.icon";
        BigInteger sn = BigInteger.ONE;
        byte[] data = new byte[]{0x01, 0x02};

        aggregator.invoke(adminAc, "registerPacket", nid, sn, data);

        byte[] dataHash = Context.hash("sha-256", data);
        byte[] sign = relayerOne.sign(dataHash);;

        aggregator.invoke(relayerOneAc, "submitSignature", nid, sn, sign);

        Executable action = () -> aggregator.invoke(relayerOneAc, "submitSignature", nid, sn, sign);
        UserRevertedException e = assertThrows(UserRevertedException.class, action);
        assertEquals("Reverted(0): Signature already exists", e.getMessage());
    }

    @Test
    public void testSubmitSignature_packetUnregistered() throws Exception {
        String nid = "0x2.icon";
        BigInteger sn = BigInteger.ONE;
        byte[] data = new byte[]{0x01, 0x02};

        byte[] dataHash = Context.hash("sha-256", data);
        byte[] sign = relayerOne.sign(dataHash);;

        Executable action = () -> aggregator.invoke(relayerOneAc, "submitSignature", nid, sn, sign);
        UserRevertedException e = assertThrows(UserRevertedException.class, action);
        assertEquals("Reverted(0): Packet not registered", e.getMessage());
    }

    @Test
    public void testSubmitSignature_invalidSignatureWithWrongData() throws Exception {
        String nid = "0x2.icon";
        BigInteger sn = BigInteger.ONE;
        byte[] data = new byte[]{0x01, 0x02};

        aggregator.invoke(adminAc, "registerPacket", nid, sn, data);

        byte[] wrongData = new byte[]{0x01, 0x02, 0x03};
        byte[] dataHash = Context.hash("sha-256", wrongData);
        byte[] sign = relayerOne.sign(dataHash);;

        Executable action = () -> aggregator.invoke(relayerOneAc, "submitSignature", nid, sn, sign);
        UserRevertedException e = assertThrows(UserRevertedException.class, action);
    
        assertEquals("Reverted(0): Invalid signature", e.getMessage());
    }
}
