package foundation.icon.xcall;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.AdditionalMatchers.aryEq;
import static org.mockito.ArgumentMatchers.anyBoolean;
import static org.mockito.ArgumentMatchers.anyString;
import static org.mockito.ArgumentMatchers.eq;
import static org.mockito.ArgumentMatchers.any;
import static org.mockito.Mockito.doThrow;
import static org.mockito.Mockito.spy;
import static org.mockito.Mockito.times;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;

import java.math.BigInteger;


import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.function.Executable;

import com.iconloop.score.test.Account;
import com.iconloop.score.test.Score;
import com.iconloop.score.test.ServiceManager;
import com.iconloop.score.test.TestBase;

import score.Address;
import foundation.icon.xcall.messages.CallMessageWithRollback;
import foundation.icon.xcall.messages.Message;
import foundation.icon.xcall.messages.PersistentMessage;
import foundation.icon.xcall.messages.XCallEnvelope;
import score.Context;
import score.UserRevertedException;
import xcall.icon.test.MockContract;

public class CallServiceTest extends TestBase {
    protected final ServiceManager sm = getServiceManager();
    protected final Account owner = sm.createAccount();
    protected final Account user = sm.createAccount();

    protected Score xcall;
    protected CallServiceImpl xcallSpy;
    protected static String nid = "0x2.ICON";
    protected static String ethNid = "0x1.ETH";

    protected NetworkAddress ethDapp = new NetworkAddress(ethNid, "0xa");
    protected NetworkAddress iconDappAddress;
    protected String baseEthConnection = "0xb";
    protected MockContract<CallServiceReceiver> dapp;
    protected MockContract<Connection> baseConnection;
    protected Score responseContract;

    String[] baseSource;
    String[] baseDestination;
    public static class ResponseContract implements CallServiceReceiver {
        public static String to = "";
        public static byte[] data = new byte[0];

        public ResponseContract() {

        }
        public void handleCallMessage(String _from, byte[] _data, String[] protocols) {
            Context.call(Context.getCaller(), "sendCall", to, data);

        }
    }

    @BeforeEach
    public void setup() throws Exception {
        dapp = new MockContract<>(CallServiceReceiverScoreInterface.class, CallServiceReceiver.class, sm, owner);
        iconDappAddress = new NetworkAddress(nid, dapp.getAddress().toString());
        baseConnection = new MockContract<>(ConnectionScoreInterface.class, Connection.class, sm, owner);
        baseSource = new String[] {baseConnection.getAddress().toString()};
        baseDestination = new String[] {baseEthConnection};
        when(baseConnection.mock.getFee(anyString(), anyBoolean())).thenReturn(BigInteger.ZERO);
        xcall = sm.deploy(owner, CallServiceImpl.class, nid);
        xcallSpy = (CallServiceImpl) spy(xcall.getInstance());
        xcall.setInstance(xcallSpy);
        responseContract = sm.deploy(owner, ResponseContract.class);

    }

    @Test
    public void sendMessage_singleProtocol() {
        // Arrange
        byte[] data = "test".getBytes();

        // Act
        xcall.invoke(dapp.account, "sendCallMessage", ethDapp.toString(), data, null, baseSource, baseDestination);

        // Assert
        CSMessageRequest request = new CSMessageRequest(iconDappAddress.toString(), ethDapp.account.toString(), BigInteger.ONE, 0, data, baseDestination);

        CSMessage msg = new CSMessage(CSMessage.REQUEST, request.toBytes());
        verify(baseConnection.mock).sendMessage(eq(ethNid), eq(CallService.NAME), eq(BigInteger.ZERO), aryEq(msg.toBytes()));
        verify(xcallSpy).CallMessageSent(dapp.getAddress(), ethDapp.toString(), BigInteger.ONE);
    }

    @Test
    public void sendMessage_defaultProtocol() {
        // Arrange
        byte[] data = "test".getBytes();
        xcall.invoke(owner, "setDefaultConnection", ethDapp.net(), baseConnection.getAddress());

        // Act
        xcall.invoke(dapp.account, "sendCallMessage", ethDapp.toString(), data);

        // Assert
        CSMessageRequest request = new CSMessageRequest(iconDappAddress.toString(), ethDapp.account.toString(), BigInteger.ONE, 0, data, null);
        CSMessage msg = new CSMessage(CSMessage.REQUEST, request.toBytes());
        verify(baseConnection.mock).sendMessage(eq(ethNid), eq(CallService.NAME), eq(BigInteger.ZERO), aryEq(msg.toBytes()));
        verify(xcallSpy).CallMessageSent(dapp.getAddress(), ethDapp.toString(), BigInteger.ONE);
    }

    @Test
    public void sendMessage_defaultProtocol_notSet() {
        // Arrange
        byte[] data = "test".getBytes();

        // Act & Assert
        UserRevertedException e = assertThrows(UserRevertedException.class, ()->  xcall.invoke(dapp.account, "sendCallMessage", ethDapp.toString(), data));
        assertEquals("Reverted(0): NoDefaultConnection", e.getMessage());
    }

    @Test
    public void sendMessage_multiProtocol() throws Exception {
        // Arrange
        byte[] data = "test".getBytes();
        MockContract<Connection>  connection1 = new MockContract<>(ConnectionScoreInterface.class, Connection.class, sm, owner);
        MockContract<Connection>  connection2 = new MockContract<>(ConnectionScoreInterface.class, Connection.class, sm, owner);
        when(connection1.mock.getFee(anyString(), anyBoolean())).thenReturn(BigInteger.ZERO);
        when(connection2.mock.getFee(anyString(), anyBoolean())).thenReturn(BigInteger.ZERO);

        String[] destinations = {"0x1eth", "0x2eth"};
        String[] sources = {connection1.getAddress().toString(), connection2.getAddress().toString()};

        // Act
        xcall.invoke(dapp.account, "sendCallMessage", ethDapp.toString(), data, null, sources, destinations);

        // Assert
        CSMessageRequest request = new CSMessageRequest(iconDappAddress.toString(), ethDapp.account.toString(), BigInteger.ONE, 0, data, destinations);
        CSMessage msg = new CSMessage(CSMessage.REQUEST, request.toBytes());
        verify(connection1.mock).sendMessage(eq(ethNid), eq(CallService.NAME), eq(BigInteger.ZERO), aryEq(msg.toBytes()));
        verify(connection2.mock).sendMessage(eq(ethNid), eq(CallService.NAME), eq(BigInteger.ZERO), aryEq(msg.toBytes()));
        verify(xcallSpy).CallMessageSent(dapp.getAddress(), ethDapp.toString(), BigInteger.ONE);
    }

    @Test
    public void sendMessage_persistent() {
        // Arrange
        byte[] data = "test".getBytes();
        Message message = new PersistentMessage(data);
        XCallEnvelope envelope = new XCallEnvelope(message);
        xcall.invoke(owner, "setDefaultConnection", ethDapp.net(), baseConnection.getAddress());

        // Act
        xcall.invoke(dapp.account, "sendCall", ethDapp.toString(), envelope.toBytes());

        // Assert
        CSMessageRequest request = new CSMessageRequest(iconDappAddress.toString(), ethDapp.account.toString(), BigInteger.ONE, PersistentMessage.TYPE, data, null);
        CSMessage msg = new CSMessage(CSMessage.REQUEST, request.toBytes());
        verify(baseConnection.mock).sendMessage(eq(ethNid), eq(CallService.NAME), eq(BigInteger.ZERO), aryEq(msg.toBytes()));
        verify(xcallSpy).CallMessageSent(dapp.getAddress(), ethDapp.toString(), BigInteger.ONE);
    }

    @Test
    public void sendMessage_response() {
        // Arrange
        xcall.invoke(owner, "setDefaultConnection", ethDapp.net(), baseConnection.getAddress());

        byte[] data1 = "test1".getBytes();
        byte[] data2 = "test2".getBytes();
        Message message = new PersistentMessage(data2);
        XCallEnvelope envelope = new XCallEnvelope(message, baseSource, baseDestination);
        CSMessageRequest request = new CSMessageRequest(ethDapp.toString(), responseContract.getAddress().toString(), BigInteger.ONE, CallMessageWithRollback.TYPE, data1, baseSource);
        CSMessage msg = new CSMessage(CSMessage.REQUEST, request.toBytes());
        xcall.invoke(baseConnection.account, "handleMessage", ethNid, msg.toBytes());

        CSMessageRequest expectedRequest = new CSMessageRequest(new NetworkAddress(nid, responseContract.getAddress()).toString(), ethDapp.account(), BigInteger.ONE, PersistentMessage.TYPE, data2, baseDestination);

        ResponseContract.to = ethDapp.toString();
        ResponseContract.data = envelope.toBytes();
        // Act
        xcall.invoke(user, "executeCall", BigInteger.ONE, data1);

        // Assert
        CSMessageResult result = new CSMessageResult(BigInteger.ONE, CSMessageResult.SUCCESS, expectedRequest.toBytes());
        CSMessage res = new CSMessage(CSMessage.RESULT, result.toBytes());
        verify(baseConnection.mock).sendMessage(eq(ethNid), eq(CallService.NAME), eq(BigInteger.ONE.negate()), aryEq(res.toBytes()));
        verify(xcallSpy).CallMessageSent(responseContract.getAddress(), ethDapp.toString(), BigInteger.ONE);
    }

    @Test
    public void sendMessage_response_anotherNetwork() {
        // Arrange
        String bscNid = "0x1.bsc";
        String to = "0x2212bsc";
        xcall.invoke(owner, "setDefaultConnection", ethDapp.net(), baseConnection.getAddress());
        xcall.invoke(owner, "setDefaultConnection", bscNid, baseConnection.getAddress());

        byte[] data1 = "test1".getBytes();
        byte[] data2 = "test2".getBytes();
        Message message = new PersistentMessage(data2);
        XCallEnvelope envelope = new XCallEnvelope(message, baseSource, baseDestination);
        CSMessageRequest request = new CSMessageRequest(ethDapp.toString(), responseContract.getAddress().toString(), BigInteger.ONE, CallMessageWithRollback.TYPE, data1, baseSource);
        CSMessage msg = new CSMessage(CSMessage.REQUEST, request.toBytes());
        xcall.invoke(baseConnection.account, "handleMessage", ethNid, msg.toBytes());

        CSMessageRequest expectedRequest = new CSMessageRequest(new NetworkAddress(nid, responseContract.getAddress()).toString(), to, BigInteger.ONE, PersistentMessage.TYPE, data2, baseDestination);

        ResponseContract.to = new NetworkAddress(bscNid, to).toString();
        ResponseContract.data = envelope.toBytes();

        // Act
        xcall.invoke(user, "executeCall", BigInteger.ONE, data1);

        // Assert
        CSMessageResult result = new CSMessageResult(BigInteger.ONE, CSMessageResult.SUCCESS, null);
        CSMessage res = new CSMessage(CSMessage.RESULT, result.toBytes());
        CSMessage req = new CSMessage(CSMessage.REQUEST, expectedRequest.toBytes());
        verify(baseConnection.mock).sendMessage(eq(ethNid), eq(CallService.NAME), eq(BigInteger.ONE.negate()), aryEq(res.toBytes()));
        verify(baseConnection.mock).sendMessage(eq(bscNid), eq(CallService.NAME), eq(BigInteger.ZERO), aryEq(req.toBytes()));
        verify(xcallSpy).CallMessageSent(responseContract.getAddress(), ResponseContract.to, BigInteger.ONE);
    }

    @Test
    public void sendMessage_response_twoWayMessage() {
        // Arrange
        xcall.invoke(owner, "setDefaultConnection", ethDapp.net(), baseConnection.getAddress());

        byte[] data1 = "test1".getBytes();
        byte[] data2 = "test2".getBytes();
        Message message = new CallMessageWithRollback(data2, data2);
        XCallEnvelope envelope = new XCallEnvelope(message, baseSource, baseDestination);
        CSMessageRequest request = new CSMessageRequest(ethDapp.toString(), responseContract.getAddress().toString(), BigInteger.ONE, CallMessageWithRollback.TYPE, data1, baseSource);
        CSMessage msg = new CSMessage(CSMessage.REQUEST, request.toBytes());
        xcall.invoke(baseConnection.account, "handleMessage", ethNid, msg.toBytes());

        CSMessageRequest expectedRequest = new CSMessageRequest(new NetworkAddress(nid, responseContract.getAddress()).toString(),  ethDapp.account, BigInteger.ONE, CallMessageWithRollback.TYPE, data2, baseDestination);

        ResponseContract.to = ethDapp.toString();
        ResponseContract.data = envelope.toBytes();

        // Act
        xcall.invoke(user, "executeCall", BigInteger.ONE, data1);

        // Assert
        CSMessageResult result = new CSMessageResult(BigInteger.ONE, CSMessageResult.SUCCESS, null);
        CSMessage res = new CSMessage(CSMessage.RESULT, result.toBytes());
        CSMessage req = new CSMessage(CSMessage.REQUEST, expectedRequest.toBytes());
        verify(baseConnection.mock).sendMessage(eq(ethNid), eq(CallService.NAME), eq(BigInteger.ONE.negate()), aryEq(res.toBytes()));
        verify(baseConnection.mock).sendMessage(eq(ethNid), eq(CallService.NAME), eq(BigInteger.ONE), aryEq(req.toBytes()));
        verify(xcallSpy).CallMessageSent(responseContract.getAddress(), ResponseContract.to, BigInteger.ONE);
    }

    @Test
    public void handleResponse_singleProtocol() {
        // Arrange
        byte[] data = "test".getBytes();
        CSMessageRequest request = new CSMessageRequest(ethDapp.toString(), dapp.getAddress().toString(), BigInteger.ONE, 1, data, baseSource);
        CSMessage msg = new CSMessage(CSMessage.REQUEST, request.toBytes());

        // Act
        xcall.invoke(baseConnection.account, "handleBTPMessage", ethNid, CallService.NAME, BigInteger.ZERO, msg.toBytes());

        // Assert
        verify(xcallSpy).CallMessage(ethDapp.toString(), dapp.getAddress().toString(), BigInteger.ONE, BigInteger.ONE, data);
    }

    @Test
    public void handleResponse_singleProtocol_invalidSender() {
        // Arrange
        byte[] data = "test".getBytes();
        Account otherConnection = sm.createAccount();
        CSMessageRequest request = new CSMessageRequest(ethDapp.toString(), dapp.getAddress().toString(), BigInteger.ONE, 1, data, baseSource);
        CSMessage msg = new CSMessage(CSMessage.REQUEST, request.toBytes());

        // Act
        assertThrows(Exception.class, ()->xcall.invoke(otherConnection, "handleBTPMessage", ethNid, CallService.NAME, BigInteger.ZERO, msg.toBytes()));

        // Assert
        verify(xcallSpy, times(0)).CallMessage(anyString(), anyString(), any(BigInteger.class), any(BigInteger.class), any(byte[].class));
    }

    @Test
    public void handleResponse_defaultProtocol() {
        // Arrange
        byte[] data = "test".getBytes();
        xcall.invoke(owner, "setDefaultConnection", ethDapp.net(), baseConnection.getAddress());
        CSMessageRequest request = new CSMessageRequest(ethDapp.toString(), dapp.getAddress().toString(), BigInteger.ONE, 1, data, null);
        CSMessage msg = new CSMessage(CSMessage.REQUEST, request.toBytes());

        // Act
        xcall.invoke(baseConnection.account, "handleBTPMessage", ethNid, CallService.NAME, BigInteger.ZERO, msg.toBytes());

        // Assert
        verify(xcallSpy).CallMessage(ethDapp.toString(), dapp.getAddress().toString(), BigInteger.ONE, BigInteger.ONE, data);
    }

    @Test
    public void handleResponse_defaultProtocol_invalidSender() {
        // Arrange
        byte[] data = "test".getBytes();
        xcall.invoke(owner, "setDefaultConnection", ethDapp.net(), baseConnection.getAddress());
        CSMessageRequest request = new CSMessageRequest(ethDapp.toString(), dapp.getAddress().toString(), BigInteger.ONE, 1, data, null);
        CSMessage msg = new CSMessage(CSMessage.REQUEST, request.toBytes());
        Account invalidConnection  = sm.createAccount();

        // Act
        assertThrows(Exception.class, ()-> xcall.invoke(invalidConnection, "handleBTPMessage", ethNid, CallService.NAME, BigInteger.ZERO, msg.toBytes()));

        // Assert
        verify(xcallSpy, times(0)).CallMessage(anyString(), anyString(), any(BigInteger.class), any(BigInteger.class), any(byte[].class));
    }

    @Test
    public void handleResponse_multiProtocol() throws Exception {
        // Arrange
        byte[] data = "test".getBytes();
        MockContract<Connection>  connection1 = new MockContract<>(ConnectionScoreInterface.class, Connection.class, sm, owner);
        MockContract<Connection>  connection2 = new MockContract<>(ConnectionScoreInterface.class, Connection.class, sm, owner);
        String[] connections = {connection1.getAddress().toString(), connection2.getAddress().toString()};

        CSMessageRequest request = new CSMessageRequest(ethDapp.toString(), dapp.getAddress().toString(), BigInteger.ONE, 1, data, connections);
        CSMessage msg = new CSMessage(CSMessage.REQUEST, request.toBytes());

        // Act
        xcall.invoke(connection1.account, "handleBTPMessage", ethNid, CallService.NAME, BigInteger.ZERO, msg.toBytes());
        verify(xcallSpy, times(0)).CallMessage(anyString(), anyString(), any(BigInteger.class), any(BigInteger.class), any(byte[].class));
        xcall.invoke(connection2.account, "handleBTPMessage", ethNid, CallService.NAME, BigInteger.ZERO, msg.toBytes());

        // Assert
        verify(xcallSpy, times(1)).CallMessage(ethDapp.toString(), dapp.getAddress().toString(), BigInteger.ONE, BigInteger.ONE, data);
    }

    @Test
    public void handleRequest_same_network_id() {
        // Arrange
        byte[] data = "test".getBytes();
        CSMessageRequest request = new CSMessageRequest(ethDapp.toString(), dapp.getAddress().toString(), BigInteger.ONE, 1, data, new String[]{baseConnection.getAddress().toString()});
        CSMessage msg = new CSMessage(CSMessage.REQUEST, request.toBytes());

        // Act
        assertThrows(Exception.class, ()-> xcall.invoke(baseConnection.account, "handleBTPMessage", nid, CallService.NAME, BigInteger.ZERO, msg.toBytes()));


        // Assert
        verify(xcallSpy, times(0)).CallMessage(anyString(), anyString(), any(BigInteger.class), any(BigInteger.class), any(byte[].class));
    }

    @Test
    public void handleResult_evmEncoding() {
        // Arrange
        xcall.invoke(owner, "setDefaultConnection", ethDapp.net(), baseConnection.getAddress());

        byte[] data = "test".getBytes();
        CSMessageResult result = new CSMessageResult(BigInteger.ONE, CSMessageResult.SUCCESS, new byte[0]);
        CSMessage msg = new CSMessage(CSMessage.RESULT, result.toBytes());

        xcall.invoke(dapp.account, "sendCallMessage", ethDapp.toString(), data, data, baseSource, baseDestination);

        // Act
        xcall.invoke(baseConnection.account, "handleMessage", ethNid, msg.toBytes());

        // Assert
        verify(xcallSpy).ResponseMessage(BigInteger.ONE, CSMessageResult.SUCCESS);
    }

    @Test
    public void handleReply() {
        // Arrange
        xcall.invoke(owner, "setDefaultConnection", ethDapp.net(), baseConnection.getAddress());

        byte[] data = "test".getBytes();
        CSMessageRequest request = new CSMessageRequest(ethDapp.toString(), dapp.getAddress().toString(), BigInteger.ONE, PersistentMessage.TYPE, data, baseSource);
        CSMessageResult result = new CSMessageResult(BigInteger.ONE, CSMessageResult.SUCCESS, request.toBytes());
        CSMessage msg = new CSMessage(CSMessage.RESULT, result.toBytes());

        xcall.invoke(dapp.account, "sendCallMessage", ethDapp.toString(), data, data, baseSource, baseDestination);

        // Act
        xcall.invoke(baseConnection.account, "handleMessage", ethNid, msg.toBytes());

        // Assert
        verify(xcallSpy).ResponseMessage(BigInteger.ONE, CSMessageResult.SUCCESS);
        verify(xcallSpy).CallMessage(ethDapp.toString(), dapp.getAddress().toString(), BigInteger.ONE, BigInteger.ONE, data);
    }

    @Test
    public void handleReply_invalidTo() {
        // Arrange
        xcall.invoke(owner, "setDefaultConnection", ethDapp.net(), baseConnection.getAddress());

        byte[] data = "test".getBytes();
        CSMessageRequest request = new CSMessageRequest("otherNid/0x1", dapp.getAddress().toString(), BigInteger.ONE, PersistentMessage.TYPE, data, baseSource);
        CSMessageResult result = new CSMessageResult(BigInteger.ONE, CSMessageResult.SUCCESS, request.toBytes());
        CSMessage msg = new CSMessage(CSMessage.RESULT, result.toBytes());

        xcall.invoke(dapp.account, "sendCallMessage", ethDapp.toString(), data, data, baseSource, baseDestination);

        // Act
        Executable handleMessage = () -> xcall.invoke(baseConnection.account, "handleMessage", ethNid, msg.toBytes());

        // Assert
        Exception e = assertThrows(Exception.class, handleMessage);
        assertEquals("Reverted(0): Invalid Reply", e.getMessage());
    }

    @Test
    public void executeCall_singleProtocol() {
        // Arrange
        byte[] data = "test".getBytes();
        CSMessageRequest request = new CSMessageRequest(ethDapp.toString(), dapp.getAddress().toString(), BigInteger.ONE, 1, data, baseSource);
        CSMessage msg = new CSMessage(CSMessage.REQUEST, request.toBytes());
        xcall.invoke(baseConnection.account, "handleBTPMessage", ethNid, CallService.NAME, BigInteger.ZERO, msg.toBytes());

        // Act
        xcall.invoke(user, "executeCall", BigInteger.ONE, data);

        // Assert
        verify(dapp.mock).handleCallMessage(ethDapp.toString(), data, new String[]{baseConnection.getAddress().toString()});
        verify(xcallSpy).CallExecuted(BigInteger.ONE, 1, "");
    }

    @Test
    public void executeCall_defaultProtocol() throws Exception {
        // Arrange
        byte[] data = "test".getBytes();
        MockContract<DefaultCallServiceReceiver> defaultDapp = new MockContract<>(DefaultCallServiceReceiverScoreInterface.class, DefaultCallServiceReceiver.class, sm, owner);
        CSMessageRequest request = new CSMessageRequest(ethDapp.toString(), defaultDapp.getAddress().toString(), BigInteger.ONE, 1, data, null);
        CSMessage msg = new CSMessage(CSMessage.REQUEST, request.toBytes());

        xcall.invoke(owner, "setDefaultConnection", ethDapp.net(), baseConnection.getAddress());
        xcall.invoke(baseConnection.account, "handleBTPMessage", ethNid, CallService.NAME, BigInteger.ZERO, msg.toBytes());

        // Act
        xcall.invoke(user, "executeCall", BigInteger.ONE, data);

        // Assert
        verify(defaultDapp.mock).handleCallMessage(ethDapp.toString(), data);
        verify(xcallSpy).CallExecuted(BigInteger.ONE, 1, "");
    }

    @Test
    public void executeCall_multiProtocol() throws Exception{
        // Arrange
        byte[] data = "test".getBytes();
        MockContract<Connection>  connection1 = new MockContract<>(ConnectionScoreInterface.class, Connection.class, sm, owner);
        MockContract<Connection>  connection2 = new MockContract<>(ConnectionScoreInterface.class, Connection.class, sm, owner);
        String[] connections = {connection1.getAddress().toString(), connection2.getAddress().toString()};

        CSMessageRequest request = new CSMessageRequest(ethDapp.toString(), dapp.getAddress().toString(), BigInteger.ONE, 1, data, connections);
        CSMessage msg = new CSMessage(CSMessage.REQUEST, request.toBytes());
        xcall.invoke(connection1.account, "handleBTPMessage", ethNid, CallService.NAME, BigInteger.ZERO, msg.toBytes());
        xcall.invoke(connection2.account, "handleBTPMessage", ethNid, CallService.NAME, BigInteger.ZERO, msg.toBytes());

        // Act
        xcall.invoke(user, "executeCall", BigInteger.ONE, data);

        // Assert
        verify(dapp.mock).handleCallMessage(ethDapp.toString(), data, connections);
        verify(xcallSpy).CallExecuted(BigInteger.ONE, 1, "");
    }

    @Test
    public void executeCall_multiProtocol_rollback() throws Exception {
        // Arrange
        byte[] data = "test".getBytes();
        MockContract<Connection>  connection1 = new MockContract<>(ConnectionScoreInterface.class, Connection.class, sm, owner);
        MockContract<Connection>  connection2 = new MockContract<>(ConnectionScoreInterface.class, Connection.class, sm, owner);
        String[] connections = {connection1.getAddress().toString(), connection2.getAddress().toString()};

        CSMessageRequest request = new CSMessageRequest(ethDapp.toString(), dapp.getAddress().toString(), BigInteger.ONE, 1, data, connections);
        CSMessage msg = new CSMessage(CSMessage.REQUEST, request.toBytes());
        xcall.invoke(connection1.account, "handleBTPMessage", ethNid, CallService.NAME, BigInteger.ZERO, msg.toBytes());
        xcall.invoke(connection2.account, "handleBTPMessage", ethNid, CallService.NAME, BigInteger.ZERO, msg.toBytes());

        // Act
        xcall.invoke(user, "executeCall", BigInteger.ONE, data);

        // Assert
        CSMessageResult msgRes = new CSMessageResult(BigInteger.ONE, CSMessageResult.SUCCESS, null);
        msg = new CSMessage(CSMessage.RESULT, msgRes.toBytes());

        verify(dapp.mock).handleCallMessage(ethDapp.toString(), data, connections);
        verify(connection1.mock).sendMessage(ethNid, CallService.NAME, BigInteger.ONE.negate(), msg.toBytes());
        verify(connection2.mock).sendMessage(ethNid, CallService.NAME, BigInteger.ONE.negate(), msg.toBytes());
        verify(xcallSpy).CallExecuted(BigInteger.ONE, 1, "");
    }

    @Test
    public void executeCall_defaultProtocol_rollback() throws Exception {
        // Arrange
        byte[] data = "test".getBytes();
        MockContract<DefaultCallServiceReceiver> defaultDapp = new MockContract<>(DefaultCallServiceReceiverScoreInterface.class, DefaultCallServiceReceiver.class, sm, owner);
        CSMessageRequest request = new CSMessageRequest(ethDapp.toString(), defaultDapp.getAddress().toString(), BigInteger.ONE, CallMessageWithRollback.TYPE, data, null);
        CSMessage msg = new CSMessage(CSMessage.REQUEST, request.toBytes());

        xcall.invoke(owner, "setDefaultConnection", ethDapp.net(), baseConnection.getAddress());
        xcall.invoke(baseConnection.account, "handleBTPMessage", ethNid, CallService.NAME, BigInteger.ZERO, msg.toBytes());

        // Act
        xcall.invoke(user, "executeCall", BigInteger.ONE, data);

        // Assert
        CSMessageResult msgRes = new CSMessageResult(BigInteger.ONE, CSMessageResult.SUCCESS, null);
        msg = new CSMessage(CSMessage.RESULT, msgRes.toBytes());

        verify(defaultDapp.mock).handleCallMessage(ethDapp.toString(), data);
        verify(xcallSpy).CallExecuted(BigInteger.ONE, 1, "");
        verify(baseConnection.mock).sendMessage(ethNid, CallService.NAME, BigInteger.ONE.negate(), msg.toBytes());
    }

    @Test
    public void executeCall_failedExecution() {
        // Arrange
        byte[] data = "test".getBytes();
        CSMessageRequest request = new CSMessageRequest(ethDapp.toString(), dapp.getAddress().toString(), BigInteger.ONE, CallMessageWithRollback.TYPE, data, baseSource);
        CSMessage msg = new CSMessage(CSMessage.REQUEST, request.toBytes());
        xcall.invoke(baseConnection.account, "handleBTPMessage", ethNid, CallService.NAME, BigInteger.ZERO, msg.toBytes());
        // Act
        doThrow(new UserRevertedException()).when(dapp.mock).handleCallMessage(ethDapp.toString(), data, new String[]{baseConnection.getAddress().toString()});
        xcall.invoke(user, "executeCall", BigInteger.ONE, data);

        // Assert
        CSMessageResult msgRes = new CSMessageResult(BigInteger.ONE, CSMessageResult.FAILURE, null);
        msg = new CSMessage(CSMessage.RESULT, msgRes.toBytes());
        verify(baseConnection.mock).sendMessage(ethNid, CallService.NAME, BigInteger.ONE.negate(), msg.toBytes());
        verify(xcallSpy).CallExecuted(BigInteger.ONE, 0, "score.RevertedException");
    }

    @Test
    public void executeCall_persistent_failedExecution() {
        // Arrange
        byte[] data = "test".getBytes();
        CSMessageRequest request = new CSMessageRequest(ethDapp.toString(), dapp.getAddress().toString(), BigInteger.ONE, PersistentMessage.TYPE, data, baseSource);
        CSMessage msg = new CSMessage(CSMessage.REQUEST, request.toBytes());
        xcall.invoke(baseConnection.account, "handleMessage", ethNid, msg.toBytes());
        // Act
        doThrow(new UserRevertedException()).when(dapp.mock).handleCallMessage(ethDapp.toString(), data, new String[]{baseConnection.getAddress().toString()});
        assertThrows(Exception.class, () -> xcall.invoke(user, "executeCall", BigInteger.ONE, data));
    }

    @Test
    public void executeCall_persistent() throws Exception {
        // Arrange
        byte[] data = "test".getBytes();
        MockContract<DefaultCallServiceReceiver> defaultDapp = new MockContract<>(DefaultCallServiceReceiverScoreInterface.class, DefaultCallServiceReceiver.class, sm, owner);
        CSMessageRequest request = new CSMessageRequest(ethDapp.toString(), defaultDapp.getAddress().toString(), BigInteger.ONE, PersistentMessage.TYPE, data, null);
        CSMessage msg = new CSMessage(CSMessage.REQUEST, request.toBytes());

        xcall.invoke(owner, "setDefaultConnection", ethDapp.net(), baseConnection.getAddress());
        xcall.invoke(baseConnection.account, "handleMessage", ethNid, msg.toBytes());

        // Act
        xcall.invoke(user, "executeCall", BigInteger.ONE, data);

        // Assert
        verify(defaultDapp.mock).handleCallMessage(ethDapp.toString(), data);
        verify(xcallSpy).CallExecuted(BigInteger.ONE, 1, "");
        Exception e = assertThrows(Exception.class, () -> xcall.invoke(user, "executeCall", BigInteger.ONE, data));
        assertEquals("Reverted(0): InvalidRequestId", e.getMessage());
    }

    @Test
    public void executeCall_reply() throws Exception {
        // Arrange
        MockContract<DefaultCallServiceReceiver> defaultDapp = new MockContract<>(DefaultCallServiceReceiverScoreInterface.class, DefaultCallServiceReceiver.class, sm, owner);
        xcall.invoke(owner, "setDefaultConnection", ethDapp.net(), baseConnection.getAddress());

        byte[] data = "test".getBytes();
        CSMessageRequest request = new CSMessageRequest(ethDapp.toString(), defaultDapp.getAddress().toString(), BigInteger.ONE, PersistentMessage.TYPE, data, baseSource);
        CSMessageResult result = new CSMessageResult(BigInteger.ONE, CSMessageResult.SUCCESS, request.toBytes());
        CSMessage msg = new CSMessage(CSMessage.RESULT, result.toBytes());

        xcall.invoke(dapp.account, "sendCallMessage", ethDapp.toString(), data, data, baseSource, baseDestination);
        xcall.invoke(baseConnection.account, "handleMessage", ethNid, msg.toBytes());

        // Act
         xcall.invoke(user, "executeCall", BigInteger.ONE, data);

        // Assert
        verify(defaultDapp.mock).handleCallMessage(ethDapp.toString(), data);
        verify(xcallSpy).CallExecuted(BigInteger.ONE, 1, "");
        Exception e = assertThrows(Exception.class, () -> xcall.invoke(user, "executeCall", BigInteger.ONE, data));
        assertEquals("Reverted(0): InvalidRequestId", e.getMessage());
    }

    @Test
    public void rollback_singleProtocol() {
        // Arrange
        byte[] data = "test".getBytes();
        byte[] rollback = "rollback".getBytes();
        xcall.invoke(dapp.account, "sendCallMessage", ethDapp.toString(), data, rollback, baseSource, baseDestination);

        // Act
        CSMessageResult msgRes = new CSMessageResult(BigInteger.ONE, CSMessageResult.FAILURE, null);
        CSMessage msg = new CSMessage(CSMessage.RESULT, msgRes.toBytes());
        xcall.invoke(baseConnection.account, "handleBTPMessage", ethNid, CallService.NAME, BigInteger.ONE, msg.toBytes());

        // Assert
        verify(xcallSpy).ResponseMessage(BigInteger.ONE, CSMessageResult.FAILURE);
        verify(xcallSpy).RollbackMessage(BigInteger.ONE);
        assertTrue(!xcall.call(Boolean.class, "verifySuccess", BigInteger.ONE));
    }

    @Test
    public void rollback_defaultProtocol() {
        // Arrange
        byte[] data = "test".getBytes();
        byte[] rollback = "rollback".getBytes();
        xcall.invoke(owner, "setDefaultConnection", ethDapp.net(), baseConnection.getAddress());
        xcall.invoke(dapp.account, "sendCallMessage", ethDapp.toString(), data, rollback);

        // Act
        CSMessageResult msgRes = new CSMessageResult(BigInteger.ONE, CSMessageResult.FAILURE, null);
        CSMessage msg = new CSMessage(CSMessage.RESULT, msgRes.toBytes());
        xcall.invoke(baseConnection.account, "handleBTPMessage", ethNid, CallService.NAME, BigInteger.ONE, msg.toBytes());

        // Assert
        verify(xcallSpy).ResponseMessage(BigInteger.ONE, CSMessageResult.FAILURE);
        verify(xcallSpy).RollbackMessage(BigInteger.ONE);
        assertTrue(!xcall.call(Boolean.class, "verifySuccess", BigInteger.ONE));
    }

    @Test
    public void rollback_defaultProtocol_invalidSender() {
        // Arrange
        byte[] data = "test".getBytes();
        byte[] rollback = "rollback".getBytes();
        xcall.invoke(owner, "setDefaultConnection", ethDapp.net(), baseConnection.getAddress());
        xcall.invoke(dapp.account, "sendCallMessage", ethDapp.toString(), data, rollback);
        Account invalidConnection  = sm.createAccount();

        // Act
        CSMessageResult msgRes = new CSMessageResult(BigInteger.ONE, CSMessageResult.FAILURE, null);
        CSMessage msg = new CSMessage(CSMessage.RESULT, msgRes.toBytes());
        assertThrows(Exception.class, ()->  xcall.invoke(invalidConnection, "handleBTPMessage", ethNid, CallService.NAME, BigInteger.ONE, msg.toBytes()));

        // Assert
        verify(xcallSpy, times(0)).ResponseMessage(BigInteger.ONE, CSMessageResult.FAILURE);
        verify(xcallSpy, times(0)).RollbackMessage(BigInteger.ONE);
        assertTrue(!xcall.call(Boolean.class, "verifySuccess", BigInteger.ONE));
    }

    @Test
    public void rollback_multiProtocol() throws Exception {
        // Arrange
        byte[] data = "test".getBytes();
        byte[] rollback = "rollback".getBytes();
        MockContract<Connection>  connection1 = new MockContract<>(ConnectionScoreInterface.class, Connection.class, sm, owner);
        MockContract<Connection>  connection2 = new MockContract<>(ConnectionScoreInterface.class, Connection.class, sm, owner);
        when(connection1.mock.getFee(anyString(), anyBoolean())).thenReturn(BigInteger.ZERO);
        when(connection2.mock.getFee(anyString(), anyBoolean())).thenReturn(BigInteger.ZERO);

        String[] destinations = {"0x1eth", "0x2eth"};
        String[] sources = {connection1.getAddress().toString(), connection2.getAddress().toString()};

        xcall.invoke(dapp.account, "sendCallMessage", ethDapp.toString(), data, rollback, sources, destinations);

        // Act
        CSMessageResult msgRes = new CSMessageResult(BigInteger.ONE, CSMessageResult.FAILURE, null);
        CSMessage msg = new CSMessage(CSMessage.RESULT, msgRes.toBytes());
        xcall.invoke(connection1.account, "handleBTPMessage", ethNid, CallService.NAME, BigInteger.ONE, msg.toBytes());
        verify(xcallSpy, times(0)).ResponseMessage(BigInteger.ONE, CSMessageResult.FAILURE);
        xcall.invoke(connection2.account, "handleBTPMessage", ethNid, CallService.NAME, BigInteger.ONE, msg.toBytes());

        // Assert
        verify(xcallSpy).ResponseMessage(BigInteger.ONE, CSMessageResult.FAILURE);
        verify(xcallSpy).RollbackMessage(BigInteger.ONE);
        assertTrue(!xcall.call(Boolean.class, "verifySuccess", BigInteger.ONE));
    }

    @Test
    public void rollback_success() throws Exception {
        // Arrange
        byte[] data = "test".getBytes();
        byte[] rollback = "rollback".getBytes();
        xcall.invoke(dapp.account, "sendCallMessage", ethDapp.toString(), data, rollback, baseSource, baseDestination);

        // Act
        CSMessageResult msgRes = new CSMessageResult(BigInteger.ONE, CSMessageResult.SUCCESS, null);
        CSMessage msg = new CSMessage(CSMessage.RESULT, msgRes.toBytes());
        xcall.invoke(baseConnection.account, "handleBTPMessage", ethNid, CallService.NAME, BigInteger.ONE, msg.toBytes());

        // Assert
        verify(xcallSpy).ResponseMessage(BigInteger.ONE, CSMessageResult.SUCCESS);
        verify(xcallSpy, times(0)).RollbackMessage(BigInteger.ONE);

        assertTrue(xcall.call(Boolean.class, "verifySuccess", BigInteger.ONE));
    }

    @Test
    public void executeRollback_singleProtocol() {
        // Arrange
        byte[] data = "test".getBytes();
        byte[] rollback = "rollback".getBytes();
        NetworkAddress xcallAddr = new NetworkAddress(nid, xcall.getAddress());

        xcall.invoke(dapp.account, "sendCallMessage", ethDapp.toString(), data, rollback, baseSource, baseDestination);

        CSMessageResult msgRes = new CSMessageResult(BigInteger.ONE, CSMessageResult.FAILURE, null);
        CSMessage msg = new CSMessage(CSMessage.RESULT, msgRes.toBytes());
        xcall.invoke(baseConnection.account, "handleBTPMessage", ethNid, CallService.NAME, BigInteger.ONE, msg.toBytes());

        // Act
        xcall.invoke(user, "executeRollback", BigInteger.ONE);

        // Assert
        verify(xcallSpy).RollbackExecuted(BigInteger.ONE);
        verify(dapp.mock).handleCallMessage(xcallAddr.toString(), rollback, new String[]{baseConnection.getAddress().toString()});
        assertTrue(!xcall.call(Boolean.class, "verifySuccess", BigInteger.ONE));
    }

    @Test
    public void executeRollback_defaultProtocol() throws Exception {
        // Arrange
        byte[] data = "test".getBytes();
        byte[] rollback = "rollback".getBytes();
        MockContract<DefaultCallServiceReceiver> defaultDapp = new MockContract<>(DefaultCallServiceReceiverScoreInterface.class, DefaultCallServiceReceiver.class, sm, owner);
        NetworkAddress xcallAddr = new NetworkAddress(nid, xcall.getAddress());

        xcall.invoke(owner, "setDefaultConnection", ethDapp.net(), baseConnection.getAddress());
        xcall.invoke(defaultDapp.account, "sendCallMessage", ethDapp.toString(), data, rollback);

        CSMessageResult msgRes = new CSMessageResult(BigInteger.ONE, CSMessageResult.FAILURE, null);
        CSMessage msg = new CSMessage(CSMessage.RESULT, msgRes.toBytes());
        xcall.invoke(baseConnection.account, "handleBTPMessage", ethNid, CallService.NAME, BigInteger.ONE, msg.toBytes());

        // Act
        xcall.invoke(user, "executeRollback", BigInteger.ONE);

        // Assert
        verify(xcallSpy).RollbackExecuted(BigInteger.ONE);
        verify(defaultDapp.mock).handleCallMessage(xcallAddr.toString(), rollback);
        assertTrue(!xcall.call(Boolean.class, "verifySuccess", BigInteger.ONE));
    }

    @Test
    public void executeRollback_multiProtocol() throws Exception {
        // Arrange
        byte[] data = "test".getBytes();
        byte[] rollback = "rollback".getBytes();
        NetworkAddress xcallAddr = new NetworkAddress(nid, xcall.getAddress());

        MockContract<Connection>  connection1 = new MockContract<>(ConnectionScoreInterface.class, Connection.class, sm, owner);
        MockContract<Connection>  connection2 = new MockContract<>(ConnectionScoreInterface.class, Connection.class, sm, owner);
        when(connection1.mock.getFee(anyString(), anyBoolean())).thenReturn(BigInteger.ZERO);
        when(connection2.mock.getFee(anyString(), anyBoolean())).thenReturn(BigInteger.ZERO);

        String[] destinations = {"0x1eth", "0x2eth"};
        String[] sources = {connection1.getAddress().toString(), connection2.getAddress().toString()};

        xcall.invoke(dapp.account, "sendCallMessage", ethDapp.toString(), data, rollback, sources, destinations);
        CSMessageResult msgRes = new CSMessageResult(BigInteger.ONE, CSMessageResult.FAILURE, null);
        CSMessage msg = new CSMessage(CSMessage.RESULT, msgRes.toBytes());
        xcall.invoke(connection1.account, "handleBTPMessage", ethNid, CallService.NAME, BigInteger.ONE, msg.toBytes());
        xcall.invoke(connection2.account, "handleBTPMessage", ethNid, CallService.NAME, BigInteger.ONE, msg.toBytes());

        // Act
        xcall.invoke(user, "executeRollback", BigInteger.ONE);


        // Assert
        verify(xcallSpy).RollbackExecuted(BigInteger.ONE);
        verify(dapp.mock).handleCallMessage(xcallAddr.toString(), rollback, sources);
        assertTrue(!xcall.call(Boolean.class, "verifySuccess", BigInteger.ONE));
    }

    @Test
    public void getFee() throws Exception {
        // Arrange
        String nid = "nid";
        BigInteger fee1 = BigInteger.valueOf(3);
        BigInteger fee2 = BigInteger.valueOf(5);
        BigInteger protocolFee = BigInteger.TEN;
        MockContract<Connection>  connection1 = new MockContract<>(ConnectionScoreInterface.class, Connection.class, sm, owner);
        MockContract<Connection>  connection2 = new MockContract<>(ConnectionScoreInterface.class, Connection.class, sm, owner);
        when(connection1.mock.getFee(nid, true)).thenReturn(fee1);
        when(connection2.mock.getFee(nid, true)).thenReturn(fee2);

        String[] sources = {connection1.getAddress().toString(), connection2.getAddress().toString()};
        xcall.invoke(owner, "setProtocolFee", protocolFee);

        // Act
        BigInteger fee = xcall.call(BigInteger.class, "getFee", nid, true, sources);


        // Assert
        assertEquals(fee1.add(fee2).add(protocolFee), fee);
    }

    @Test
    public void getFee_default() throws Exception {
        // Arrange
        String nid = "nid";
        BigInteger fee1 = BigInteger.valueOf(3);
        BigInteger protocolFee = BigInteger.TEN;
        when(baseConnection.mock.getFee(nid, true)).thenReturn(fee1);

        xcall.invoke(owner, "setDefaultConnection", nid, baseConnection.getAddress());
        xcall.invoke(owner, "setProtocolFee", protocolFee);

        // Act
        BigInteger fee = xcall.call(BigInteger.class, "getFee", nid, true);


        // Assert
        assertEquals(fee1.add(protocolFee), fee);
    }

    @Test
    public void setGetDefaultCOnnection() throws Exception {
        // Arrange
        String nid = "nid";

        xcall.invoke(owner, "setDefaultConnection", nid, baseConnection.getAddress());
        Address connection =  xcall.call(Address.class, "getDefaultConnection", nid);
        // Assert
        assertEquals(connection, baseConnection.getAddress());
    }

    @Test
    public void getFee_defaultProtocol_notSet() throws Exception {
        // Arrange
        String nid = "nid";

        // Act & Assert
        UserRevertedException e = assertThrows(UserRevertedException.class, ()-> xcall.call(BigInteger.class, "getFee", nid, true));
        assertEquals("Reverted(0): NoDefaultConnection", e.getMessage());


    }

    @Test
    public void entryPermissions() {
        String expectedErrorMessage = "Reverted(0): OnlyAdmin";
        Account nonAuthorized = sm.createAccount();
        UserRevertedException e;

        e = assertThrows(UserRevertedException.class,
            () -> xcall.invoke(nonAuthorized, "setAdmin", nonAuthorized.getAddress()));
        assertEquals(expectedErrorMessage, e.getMessage());

        e = assertThrows(UserRevertedException.class,
            () -> xcall.invoke(nonAuthorized, "setProtocolFee", BigInteger.ONE));
        assertEquals(expectedErrorMessage, e.getMessage());

        e = assertThrows(UserRevertedException.class,
            () -> xcall.invoke(nonAuthorized, "setProtocolFeeHandler", nonAuthorized.getAddress()));
        assertEquals(expectedErrorMessage, e.getMessage());

        e = assertThrows(UserRevertedException.class,
            () -> xcall.invoke(nonAuthorized, "setDefaultConnection", "nid", nonAuthorized.getAddress()));
        assertEquals(expectedErrorMessage, e.getMessage());
    }
}