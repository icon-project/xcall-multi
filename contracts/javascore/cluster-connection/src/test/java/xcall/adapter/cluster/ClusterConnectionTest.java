package xcall.adapter.cluster;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.mockito.ArgumentMatchers.eq;
import static org.mockito.Mockito.*;

import java.nio.charset.StandardCharsets;
import java.security.*;

import java.math.BigInteger;
import java.util.Arrays;

import score.Context;

import org.bouncycastle.jce.provider.BouncyCastleProvider;

import foundation.icon.icx.KeyWallet;



import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.mockito.MockedStatic;
import org.mockito.Mockito;

import com.iconloop.score.test.Account;
import com.iconloop.score.test.Score;
import com.iconloop.score.test.ServiceManager;
import com.iconloop.score.test.TestBase;

import score.UserRevertedException;
import score.Address;
import score.ByteArrayObjectWriter;
import foundation.icon.xcall.CallService;
import foundation.icon.xcall.CallServiceScoreInterface;


import xcall.icon.test.MockContract;

public class ClusterConnectionTest extends TestBase {
    protected final ServiceManager sm = getServiceManager();

    protected final Account owner = sm.createAccount();
    protected final Account user = sm.createAccount();
    protected final Account admin = sm.createAccount();
    protected final Account xcallMock = sm.createAccount();

    protected final Account source_relayer = sm.createAccount();
    protected final Account destination_relayer = sm.createAccount();

    protected Score xcall, connection;
    protected CallService xcallSpy;
    protected ClusterConnection connectionSpy;

    protected static String nidSource = "nid.source";
    protected static String nidTarget = "nid.target";

    // static MockedStatic<Context> contextMock;

    protected MockContract<CallService> callservice;

    @BeforeEach
    public void setup() throws Exception {
        Security.addProvider(new BouncyCastleProvider());
        callservice = new MockContract<>(CallServiceScoreInterface.class, CallService.class, sm, owner);


        connection = sm.deploy(owner, ClusterConnection.class, source_relayer.getAddress(),
                callservice.getAddress());
        connectionSpy = (ClusterConnection) spy(connection.getInstance());
        connection.setInstance(connectionSpy);
    }

    @Test
    public void testSetAdmin() {

        connection.invoke(owner, "setAdmin", admin.getAddress());
        assertEquals(connection.call("admin"), admin.getAddress());
    }

    @Test
    public void testSetAdmin_unauthorized() {
        UserRevertedException e = assertThrows(UserRevertedException.class,
                () -> connection.invoke(user, "setAdmin", admin.getAddress()));
        assertEquals("Reverted(0): " + "Only admin can call this function", e.getMessage());
    }

    @Test
    public void setFee() {
        connection.invoke(source_relayer, "setFee", nidTarget, BigInteger.TEN, BigInteger.TEN);
        assertEquals(connection.call("getFee", nidTarget, true), BigInteger.TEN.add(BigInteger.TEN));
    }

    @Test
    public void sendMessage() {
        connection.invoke(callservice.account, "sendMessage", nidTarget, "xcall", BigInteger.ONE, "test".getBytes());
        verify(connectionSpy).Message(nidTarget, BigInteger.ONE, "test".getBytes());
    }

    @Test
    public void testSendMessage_unauthorized() {
        UserRevertedException e = assertThrows(UserRevertedException.class,
                () -> connection.invoke(user, "sendMessage", nidTarget, "xcall", BigInteger.ONE, "test".getBytes()));
        assertEquals("Reverted(0): " + "Only xCall can send messages", e.getMessage());
    }

    @Test
    public void testRevertMessage() {

        connection.invoke(source_relayer, "revertMessage", BigInteger.ONE);
    }

    @Test
    public void testRevertMessage_unauthorized(){
        UserRevertedException e = assertThrows(UserRevertedException.class, ()->connection.invoke(user, "revertMessage", BigInteger.ONE));
        assertEquals("Reverted(0): "+"Only relayer can call this function", e.getMessage());
        
    }

    @Test
    public void testSetFeesUnauthorized(){
        UserRevertedException e = assertThrows(UserRevertedException.class,() -> connection.invoke(user, "setFee", "0xevm",
        BigInteger.TEN, BigInteger.TEN));
        assertEquals("Reverted(0): "+"Only relayer can call this function", e.getMessage());
    }

    @Test
    public void testClaimFees(){
        setFee();
        connection.invoke(source_relayer, "claimFees");
        assertEquals(source_relayer.getBalance(), BigInteger.ZERO);

        UserRevertedException e = assertThrows(UserRevertedException.class,() -> connection.invoke(callservice.account, "sendMessage", nidTarget,
        "xcall", BigInteger.ONE, "null".getBytes()));
        assertEquals(e.getMessage(), "Reverted(0): Insufficient balance");

        try (MockedStatic<Context> contextMock = Mockito.mockStatic(Context.class, Mockito.CALLS_REAL_METHODS)) {
            contextMock.when(() -> Context.getValue()).thenReturn(BigInteger.valueOf(20));
            connection.invoke(callservice.account, "sendMessage", nidTarget,"xcall", BigInteger.ONE, "null".getBytes());
        }

        
        try (MockedStatic<Context> contextMock = Mockito.mockStatic(Context.class, Mockito.CALLS_REAL_METHODS)) {
            contextMock.when(() -> Context.getBalance(connection.getAddress())).thenReturn(BigInteger.valueOf(20));
            contextMock.when(() -> Context.transfer(source_relayer.getAddress(),BigInteger.valueOf(20))).then(invocationOnMock -> null);
            connection.invoke(source_relayer, "claimFees");
        }
    }

    @Test
    public void testClaimFees_unauthorized(){
        setFee();
        UserRevertedException e = assertThrows(UserRevertedException.class,() -> connection.invoke(user, "claimFees"));
        assertEquals(e.getMessage(), "Reverted(0): "+"Only relayer can call this function");
    }

    public MockedStatic.Verification value() {
        return () -> Context.getValue();
    }

    @Test
    public void testRecvMessageWithSignatures() throws Exception{
        byte[] data = "test".getBytes();
        byte[] messageHash = getMessageHash(nidSource, BigInteger.ONE, data);
        byte[][] byteArray = new byte[1][];
        KeyWallet wallet = KeyWallet.create();
        byteArray[0] = wallet.sign(messageHash);
        byte[][] validators = new byte[][] {
                compressPublicKey(wallet.getPublicKey().toByteArray()),
        };
        connection.invoke(owner, "updateValidators", validators, BigInteger.ONE);        
        connection.invoke(source_relayer, "recvMessageWithSignatures", nidSource, BigInteger.ONE, data, byteArray);
        verify(callservice.mock).handleMessage(eq(nidSource), eq("test".getBytes()));
    }

    private byte[] compressPublicKey(byte[] uncompressedPublicKey) {
        byte[] xBytes = Arrays.copyOfRange(uncompressedPublicKey, 1, 33); // 32 bytes for x
        byte[] yBytes = Arrays.copyOfRange(uncompressedPublicKey, 33, 65); // 32 bytes for y
        BigInteger y = new BigInteger(1, yBytes);
        byte prefix = (y.testBit(0)) ? (byte) 0x03 : (byte) 0x02;
        byte[] compressedKey = new byte[33];
        compressedKey[0] = prefix;
        System.arraycopy(xBytes, 0, compressedKey, 1, 32);
        return compressedKey;

    }

    @Test
    public void testRecvMessageWithMultiSignatures() throws Exception{
        byte[] data = "test".getBytes();
        byte[] messageHash = getMessageHash(nidSource, BigInteger.ONE, data);
        byte[][] byteArray = new byte[2][];
        KeyWallet wallet = KeyWallet.create();
        KeyWallet wallet2 = KeyWallet.create();
        byteArray[0] = wallet.sign(messageHash);
        byteArray[1] = wallet2.sign(messageHash);
        byte[] compressPublicKey = compressPublicKey(wallet.getPublicKey().toByteArray());
        byte[][] validators = new byte[][] {
                compressPublicKey(wallet.getPublicKey().toByteArray()),
                compressPublicKey(wallet2.getPublicKey().toByteArray())
        };
        connection.invoke(owner, "updateValidators", validators, BigInteger.TWO);   
        connection.invoke(source_relayer, "recvMessageWithSignatures", nidSource, BigInteger.ONE, data, byteArray);
        verify(callservice.mock).handleMessage(eq(nidSource), eq("test".getBytes()));
    }

    @Test
    public void testRecvMessageWithSignaturesNotEnoughSignatures() throws Exception{
        byte[] data = "test".getBytes();
        byte[] messageHash = getMessageHash(nidSource, BigInteger.ONE, data);
        KeyWallet wallet = KeyWallet.create();
        KeyWallet wallet2 = KeyWallet.create();
        byte[][] byteArray = new byte[1][];
        byteArray[0] = wallet.sign(messageHash);
        byte[][] validators = new byte[][] {
                compressPublicKey(wallet.getPublicKey().toByteArray()),
                compressPublicKey(wallet2.getPublicKey().toByteArray()),
        };
        connection.invoke(owner, "updateValidators", validators, BigInteger.TWO);
        UserRevertedException e = assertThrows(UserRevertedException.class,
                ()->connection.invoke(source_relayer, "recvMessageWithSignatures", nidSource, BigInteger.ONE, data, byteArray));
        assertEquals("Reverted(0): Not enough signatures", e.getMessage());
        verifyNoInteractions(callservice.mock);
    }

    @Test
    public void testRecvMessageWithSignaturesNotEnoughValidSignatures() throws Exception{
        byte[] data = "test".getBytes();
        byte[] messageHash = getMessageHash(nidSource, BigInteger.ONE, data);
        KeyWallet wallet = KeyWallet.create();
        KeyWallet wallet2 = KeyWallet.create();
        byte[][] byteArray = new byte[2][];
        byteArray[0] = wallet.sign(messageHash);
        byteArray[1] = wallet.sign(messageHash);
        byte[][] validators = new byte[][] {
                compressPublicKey(wallet.getPublicKey().toByteArray()),
                compressPublicKey(wallet2.getPublicKey().toByteArray()),
        };
        connection.invoke(owner, "updateValidators", validators, BigInteger.TWO);
        UserRevertedException e = assertThrows(UserRevertedException.class,
                ()->connection.invoke(source_relayer, "recvMessageWithSignatures", nidSource, BigInteger.ONE, data, byteArray));
        assertEquals("Reverted(0): Not enough valid signatures", e.getMessage());
        verifyNoInteractions(callservice.mock);
    }


    public static byte[] getMessageHash(String srcNetwork, BigInteger _connSn, byte[] msg) {
        ByteArrayObjectWriter writer = Context.newByteArrayObjectWriter("RLPn");
        writer.beginList(3);
        writer.write(srcNetwork);
        writer.write(_connSn);
        writer.write(msg);
        writer.end();
        return Context.hash("keccak-256", writer.toByteArray());
    }
    @Test
    public void testAddSigners() throws Exception{
        KeyWallet wallet = KeyWallet.create();
        KeyWallet wallet2 = KeyWallet.create();
        byte[][] validators = new byte[][] {
                compressPublicKey(wallet.getPublicKey().toByteArray()),
                compressPublicKey(wallet2.getPublicKey().toByteArray()),
        };
        connection.invoke(owner, "updateValidators", validators, BigInteger.TWO);
        String[] signers = connection.call(String[].class,"listValidators");
        assertEquals(signers.length, 2);
    }

}