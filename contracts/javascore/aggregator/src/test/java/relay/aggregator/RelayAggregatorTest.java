package relay.aggregator;

import java.math.BigInteger;
import java.security.MessageDigest;
import java.security.PrivateKey;
import java.security.Security;
import java.security.AlgorithmParameters;
import java.security.KeyFactory;
import java.security.Signature;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.function.Executable;

import org.bouncycastle.jce.provider.BouncyCastleProvider;
import java.security.spec.ECPrivateKeySpec;
import java.security.spec.ECGenParameterSpec;
import java.security.spec.ECParameterSpec;

import score.Address;
import score.UserRevertedException;
import score.DictDB;

import com.iconloop.score.test.Account;
import com.iconloop.score.test.Score;
import com.iconloop.score.test.ServiceManager;
import com.iconloop.score.test.TestBase;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.spy;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;

class RelayAggregatorTest extends TestBase {
    private final ServiceManager sm = getServiceManager();

    private final Account adminAc = sm.createAccount();

    private final Account relayerAcOne = sm.getAccount(new Address(HexString.toBytes("hxe794bcf6a92efb5e0b58cfc4728236c650d86dce")));
    private final String relayerAcOnePkey = "40b90166d6ace4acbdc59596fd483e487bf58ec4ae5ff31e4f97f039af5b23f7";
    
    private final Account relayerAcTwo = sm.getAccount(new Address(HexString.toBytes("hx3445c3d4341a9b1fc2ae6fd578a4453ab0072c07")));
    private final String relayerAcTwoPkey = "0xb533387c502c39eea62f4c2a5be31e388fa87313438236656b4c71be92fed066";

    private Score aggregator;
    private RelayAggregator aggregatorSpy;

    @BeforeEach
    void setup() throws Exception {
        Address[] relayers = new Address[]{relayerAcOne.getAddress(), relayerAcTwo.getAddress()};
        aggregator = sm.deploy(adminAc, RelayAggregator.class, adminAc.getAddress(), relayers);

        aggregatorSpy = (RelayAggregator) spy(aggregator.getInstance());
        aggregator.setInstance(aggregatorSpy);

        Security.addProvider(new BouncyCastleProvider());
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
    @SuppressWarnings("unchecked")
    public void testSubmitSignature() throws Exception {
        String nid = "0x2.icon";
        BigInteger sn = BigInteger.ONE;
        byte[] data = new byte[]{0x01, 0x02};

        aggregator.invoke(adminAc, "registerPacket", nid, sn, data);
    
        DictDB<Address, String> mockSignatures = mock(DictDB.class);
        

        when(aggregatorSpy.getSignatures(nid, sn)).thenReturn(mockSignatures);

        byte[] dataHash = hashData(data);
        String sign = signData(dataHash,relayerAcOnePkey);

        aggregator.invoke(relayerAcOne, "submitSignature", nid, sn, sign);

        verify(mockSignatures).set(relayerAcOne.getAddress(), sign);
    }

    private static byte[] hashData(byte[] data) throws Exception {
        MessageDigest digest = MessageDigest.getInstance("SHA-256");
        return digest.digest(data);
    }

    private static String signData(byte[] dataHash, String pKeyStr) throws Exception {
        byte[] pKeyBytes = HexString.toBytes(pKeyStr);
        
        PrivateKey pKey = parsePrivateKey(pKeyBytes);

        Signature signature = Signature.getInstance("SHA256withECDSA", "BC");
        signature.initSign(pKey);
        signature.update(dataHash);
        
        byte[] sign = signature.sign();
        return HexString.fromBytes(sign);
    }

    public static PrivateKey parsePrivateKey(byte[] pKeyBytes) throws Exception {
        // Ensure the private key is 32 bytes
        if (pKeyBytes.length != 32) {
            throw new IllegalArgumentException("Invalid private key length: must be 32 bytes.");
        }

        // Create a BigInteger from the private key bytes (unsigned)
        BigInteger privateKeyInt = new BigInteger(1, pKeyBytes);

        // Create EC PrivateKeySpec using the secp256k1 curve parameters
        ECParameterSpec ecSpec = getSecp256k1Curve();
        ECPrivateKeySpec privateKeySpec = new ECPrivateKeySpec(privateKeyInt, ecSpec);

        // Create the KeyFactory for generating EC keys
        KeyFactory keyFactory = KeyFactory.getInstance("ECDSA", "BC");

        // Generate the private key from the spec
        return keyFactory.generatePrivate(privateKeySpec);
    }

    private static ECParameterSpec getSecp256k1Curve() {
        try {
            AlgorithmParameters parameters = AlgorithmParameters.getInstance("EC", "BC");
            parameters.init(new ECGenParameterSpec("secp256k1"));
            return parameters.getParameterSpec(ECParameterSpec.class);
        } catch (Exception e) {
            throw new RuntimeException("Failed to get secp256k1 curve parameters", e);
        }
    }
}
