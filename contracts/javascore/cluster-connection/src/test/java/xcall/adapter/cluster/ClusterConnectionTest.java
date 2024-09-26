package xcall.adapter.cluster;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.AdditionalMatchers.aryEq;
import static org.mockito.ArgumentMatchers.anyBoolean;
import static org.mockito.ArgumentMatchers.anyString;
import static org.mockito.ArgumentMatchers.eq;
import static org.mockito.ArgumentMatchers.any;
import static org.mockito.Mockito.*;

import java.security.*;

import java.beans.Transient;
import java.math.BigInteger;
import score.Context;

import org.bouncycastle.jce.provider.BouncyCastleProvider;
import org.bouncycastle.jcajce.provider.digest.Keccak;

import foundation.icon.icx.KeyWallet;


import org.junit.jupiter.api.Assertions;
import org.junit.jupiter.api.BeforeAll;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.function.Executable;
import org.mockito.MockedStatic;
import org.mockito.Mockito;

import com.iconloop.score.test.Account;
import com.iconloop.score.test.Score;
import com.iconloop.score.test.ServiceManager;
import com.iconloop.score.test.TestBase;

import xcall.adapter.cluster.ClusterConnection;
import score.UserRevertedException;
import score.Address;
import foundation.icon.ee.types.Bytes;
import foundation.icon.icx.Call;
import foundation.icon.score.client.RevertedException;
import foundation.icon.xcall.CSMessage;
import foundation.icon.xcall.CSMessageRequest;
import foundation.icon.xcall.CallService;
import foundation.icon.xcall.CallServiceReceiver;
import foundation.icon.xcall.CallServiceScoreInterface;
import foundation.icon.xcall.ConnectionScoreInterface;
import foundation.icon.xcall.Connection;
import foundation.icon.xcall.NetworkAddress;
import s.java.math.BigDecimal;

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

    // @BeforeAll
    // protected static void init() {
    //     contextMock = Mockito.mockStatic(Context.class, Mockito.CALLS_REAL_METHODS);
    // }

    @BeforeEach
    public void setup() throws Exception {
        Security.addProvider(new BouncyCastleProvider());
        callservice = new MockContract<>(CallServiceScoreInterface.class, CallService.class, sm, owner);

        // xcall = sm.deploy(owner, CallService.class, nidSource);
        // xcallSpy = (CallService) spy(xcall.getInstance());
        // xcall.setInstance(xcallSpy);
        // contextMock.reset();

        connection = sm.deploy(owner, ClusterConnection.class, source_relayer.getAddress(),
                callservice.getAddress());
        connectionSpy = (ClusterConnection) spy(connection.getInstance());
        connection.setInstance(connectionSpy);
    }

    @Test
    public void testSetAdmin() {
        // connection.invoke(source_relayer, "setFee", "0xevm", BigInteger.TEN,
        // BigInteger.TEN);

        connection.invoke(source_relayer, "setAdmin", admin.getAddress());
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
    public void testRecvMessage() {
        connection.invoke(source_relayer, "recvMessage", nidSource, BigInteger.ONE, "test".getBytes());
        verify(callservice.mock).handleMessage(eq(nidSource), eq("test".getBytes()));
    }

    @Test
    public void testRecvMessage_unauthorized(){

        UserRevertedException e = assertThrows(UserRevertedException.class, ()->  connection.invoke(xcallMock, "recvMessage",  nidSource, BigInteger.ONE, "test".getBytes()));
        assertEquals("Reverted(0): "+"Only admin can call this function", e.getMessage());
    }

    @Test
    public void testSendMessage_unauthorized() {
        UserRevertedException e = assertThrows(UserRevertedException.class,
                () -> connection.invoke(user, "sendMessage", nidTarget, "xcall", BigInteger.ONE, "test".getBytes()));
        assertEquals("Reverted(0): " + "Only xCall can send messages", e.getMessage());
    }

    @Test
    public void testRecvMessage_duplicateMsg(){
        connection.invoke(source_relayer, "recvMessage",nidSource, BigInteger.ONE, "test".getBytes());

        UserRevertedException e = assertThrows(UserRevertedException.class,() -> connection.invoke(source_relayer, "recvMessage",
        nidSource, BigInteger.ONE, "test".getBytes()));
        assertEquals(e.getMessage(), "Reverted(0): "+"Duplicate Message");
    }

    @Test
    public void testRevertMessage() {

        connection.invoke(source_relayer, "revertMessage", BigInteger.ONE);
    }

    @Test
    public void testRevertMessage_unauthorized(){
        UserRevertedException e = assertThrows(UserRevertedException.class, ()->connection.invoke(user, "revertMessage", BigInteger.ONE));
        assertEquals("Reverted(0): "+"Only admin can call this function", e.getMessage());
        
    }

    @Test
    public void testSetFeesUnauthorized(){
        UserRevertedException e = assertThrows(UserRevertedException.class,() -> connection.invoke(user, "setFee", "0xevm",
        BigInteger.TEN, BigInteger.TEN));
        assertEquals("Reverted(0): "+"Only admin can call this function", e.getMessage());
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
        assertEquals(e.getMessage(), "Reverted(0): "+"Only admin can call this function");
    }

    public MockedStatic.Verification value() {
        return () -> Context.getValue();
    }

    @Test
    public void testGetReceipt(){
        assertEquals(connection.call("getReceipts", nidSource, BigInteger.ONE),
        false);

        connection.invoke(source_relayer, "recvMessage",nidSource, BigInteger.ONE, "test".getBytes());

        assertEquals(connection.call("getReceipts", nidSource, BigInteger.ONE),
        true);
    }

    @Test
    public void testRecvMessageWithSignatures() throws Exception{
        byte[] data = "test".getBytes();
        byte[] messageHash = keccak256(data);
        byte[][] byteArray = new byte[1][];
        KeyWallet wallet = KeyWallet.create();
        byteArray[0] = wallet.sign(messageHash);
        connection.invoke(source_relayer, "addSigner", Address.fromString(wallet.getAddress().toString()));
        connection.invoke(source_relayer, "setRequiredSignerCount", BigInteger.ONE);
        connection.invoke(source_relayer, "recvMessageWithSignatures", nidSource, BigInteger.ONE, data, byteArray);
        verify(callservice.mock).handleMessage(eq(nidSource), eq("test".getBytes()));
    }

    @Test
    public void testRecvMessageWithMultiSignatures() throws Exception{
        byte[] data = "test".getBytes();
        byte[] messageHash = keccak256(data);
        byte[][] byteArray = new byte[2][];
        KeyWallet wallet = KeyWallet.create();
        KeyWallet wallet2 = KeyWallet.create();
        byteArray[0] = wallet.sign(messageHash);
        byteArray[1] = wallet2.sign(messageHash);
        connection.invoke(source_relayer, "addSigner", Address.fromString(wallet.getAddress().toString()));
        connection.invoke(source_relayer, "addSigner", Address.fromString(wallet2.getAddress().toString()));
        connection.invoke(source_relayer, "setRequiredSignerCount", BigInteger.TWO);
        connection.invoke(source_relayer, "recvMessageWithSignatures", nidSource, BigInteger.ONE, data, byteArray);
        verify(callservice.mock).handleMessage(eq(nidSource), eq("test".getBytes()));
    }

    @Test
    public void testRecvMessageWithSignaturesNotEnoughSignatures() throws Exception{
        byte[] data = "test".getBytes();
        byte[] messageHash = keccak256(data);
        KeyWallet wallet = KeyWallet.create();
        byte[][] byteArray = new byte[1][];
        byteArray[0] = wallet.sign(messageHash);
        connection.invoke(source_relayer, "addSigner", Address.fromString(wallet.getAddress().toString()));
        connection.invoke(source_relayer, "setRequiredSignerCount", BigInteger.TWO);
        connection.invoke(source_relayer, "recvMessageWithSignatures", nidSource, BigInteger.ONE, data, byteArray);
        verifyNoInteractions(callservice.mock);
    }


    // Hash the message with Keccak-256
    public static byte[] keccak256(byte[] input) {
        Keccak.Digest256 keccak256 = new Keccak.Digest256();
        return keccak256.digest(input);
    }

    @Test
    public void testAddSigners() throws Exception{
        KeyWallet wallet = KeyWallet.create();
        connection.invoke(source_relayer, "addSigner", Address.fromString(wallet.getAddress().toString()));
        Address[] signers = connection.call(Address[].class,"listSigners");
        assertEquals(signers.length, 2);
    }

    @Test
    public void testAddNRemoveSigners() throws Exception{
        KeyWallet wallet = KeyWallet.create();
        KeyWallet wallet2 = KeyWallet.create();
        KeyWallet wallet3 = KeyWallet.create();
        connection.invoke(source_relayer, "addSigner", Address.fromString(wallet.getAddress().toString()));
        connection.invoke(source_relayer, "addSigner", Address.fromString(wallet2.getAddress().toString()));
        Address[] signers = connection.call(Address[].class,"listSigners");
        assertEquals(signers.length, 3);

        connection.invoke(source_relayer, "removeSigner", Address.fromString(wallet3.getAddress().toString()));
        signers = connection.call(Address[].class,"listSigners");
        assertEquals(signers.length, 3);

        connection.invoke(source_relayer, "removeSigner", Address.fromString(wallet2.getAddress().toString()));
        signers = connection.call(Address[].class,"listSigners");
        assertEquals(signers.length, 2);
    }

}