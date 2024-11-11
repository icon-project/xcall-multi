package intent_contracts;

import com.iconloop.score.test.Account;
import com.iconloop.score.test.Score;
import com.iconloop.score.test.ServiceManager;
import com.iconloop.score.test.TestBase;

import intent_contracts.mocks.MockToken;
import network.icon.intent.Intent;
import network.icon.intent.constants.Constant;
import network.icon.intent.structs.Cancel;
import network.icon.intent.structs.OrderFill;
import network.icon.intent.structs.OrderMessage;
import network.icon.intent.structs.SwapOrder;
import network.icon.intent.utils.SwapOrderData;
import score.Context;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import score.UserRevertedException;

import java.math.BigInteger;

import static java.math.BigInteger.TEN;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.Mockito.when;

public class IntentTest extends TestBase {
    private static final ServiceManager sm = getServiceManager();
    private static final Account deployer = sm.createAccount();
    private static final Account feeHandler = sm.createAccount();
    private static final Account relayAddress = sm.createAccount();
    private static final Account user1 = sm.createAccount();
    private static final Account user2 = sm.createAccount();
    private static final Account solver = sm.createAccount();

    private static Score intent;
    private MockToken token;

    private final String srcNid = "Network-1";
    private final String destinationNetwork = "Network-2";
    private final String toToken = "0x7891";
    private final BigInteger amount = BigInteger.valueOf(500).multiply(TEN.pow(18));
    private final BigInteger toAmount = BigInteger.valueOf(400).multiply(TEN.pow(18));
    private final byte[] data = "".getBytes();
    private final BigInteger protocolFee = BigInteger.valueOf(50);
    private static final BigInteger initialSupply = BigInteger.valueOf(1000);
    private static final BigInteger totalSupply = initialSupply.multiply(TEN.pow(18));

    @BeforeEach
    void setup() throws Exception {
        // Deploy a mock token contract
        token = new MockToken(sm, deployer);

        // Deploy Intent contract with correct parameters
        intent = sm.deploy(deployer, Intent.class, "Network-1", protocolFee, feeHandler.getAddress(),
                relayAddress.getAddress());
    }

    @Test
    void testSwap() {
        // Set mock behavior for the initial balances
        when(token.tokenContract.mock.balanceOf(deployer.getAddress())).thenReturn(totalSupply);
        when(token.tokenContract.mock.balanceOf(user1.getAddress())).thenReturn(BigInteger.ZERO);
        when(token.tokenContract.mock.balanceOf(user2.getAddress())).thenReturn(BigInteger.ZERO);

        // Assert deployer has total supply initially
        assertEquals(totalSupply, token.tokenContract.mock.balanceOf(deployer.getAddress()));

        // Simulate transfer from deployer to user1
        boolean success = token.tokenContract.mock.transfer(user1.getAddress(), totalSupply);
        assertTrue(success);

        // Assert user1 now has the total supply
        when(token.tokenContract.mock.balanceOf(user1.getAddress())).thenReturn(totalSupply);
        assertEquals(totalSupply, token.tokenContract.mock.balanceOf(user1.getAddress()));
        when(token.tokenContract.mock.balanceOf(deployer.getAddress())).thenReturn(BigInteger.ZERO);
        assertEquals(BigInteger.ZERO, token.tokenContract.mock.balanceOf(deployer.getAddress()));

        // Create SwapOrderData and set the required parameters
        SwapOrderData swapOrder = new SwapOrderData();
        swapOrder.id = BigInteger.valueOf(1);
        swapOrder.emitter = intent.getAddress().toString();
        swapOrder.srcNID = srcNid;
        swapOrder.dstNID = destinationNetwork;
        swapOrder.creator = user1.getAddress().toString();
        swapOrder.destinationAddress = user2.getAddress().toString();
        swapOrder.token = token.tokenContract.getAddress().toString();
        swapOrder.amount = amount;
        swapOrder.toToken = toToken;
        swapOrder.toAmount = toAmount;
        swapOrder.data = data;

        // Invoke the swap function on the Intent contract
        intent.invoke(user1, "swap", swapOrder);

        // Simulate token balance changes post-swap
        when(token.tokenContract.mock.balanceOf(user1.getAddress())).thenReturn(totalSupply.subtract(amount));
        when(token.tokenContract.mock.balanceOf(intent.getAddress())).thenReturn(amount);

        // Assert the balances after the swap
        assertEquals(totalSupply.subtract(amount), token.tokenContract.mock.balanceOf(user1.getAddress()));
        assertEquals(amount, token.tokenContract.mock.balanceOf(intent.getAddress()));
    }

    @Test
    void testSwapInvalidCreator() {
        when(token.tokenContract.mock.balanceOf(user1.getAddress())).thenReturn(totalSupply);

        SwapOrderData swapOrder = new SwapOrderData();
        swapOrder.id = BigInteger.valueOf(1);
        swapOrder.emitter = intent.getAddress().toString();
        swapOrder.srcNID = srcNid;
        swapOrder.dstNID = destinationNetwork;
        swapOrder.creator = intent.getAddress().toString();
        swapOrder.destinationAddress = user2.getAddress().toString();
        swapOrder.token = token.tokenContract.getAddress().toString();
        swapOrder.amount = amount;
        swapOrder.toToken = toToken;
        swapOrder.toAmount = toAmount;
        swapOrder.data = data;

        UserRevertedException exception = assertThrows(UserRevertedException.class, () -> {
            intent.invoke(deployer, "swap", swapOrder); // This should revert
        });

        assertEquals("Reverted(0): Creator must be sender", exception.getMessage());
    }

    @Test
    void testSwapInvalidSrcNid() {
        when(token.tokenContract.mock.balanceOf(user1.getAddress())).thenReturn(totalSupply);

        SwapOrderData swapOrder = new SwapOrderData();
        swapOrder.id = BigInteger.valueOf(1);
        swapOrder.emitter = intent.getAddress().toString();
        swapOrder.srcNID = "dummy";
        swapOrder.dstNID = destinationNetwork;
        swapOrder.creator = deployer.getAddress().toString();
        swapOrder.destinationAddress = user2.getAddress().toString();
        swapOrder.token = token.tokenContract.getAddress().toString();
        swapOrder.amount = amount;
        swapOrder.toToken = toToken;
        swapOrder.toAmount = toAmount;
        swapOrder.data = data;

        UserRevertedException exception = assertThrows(UserRevertedException.class, () -> {
            intent.invoke(deployer, "swap", swapOrder);
        });

        assertEquals("Reverted(0): NID is misconfigured", exception.getMessage());
    }

    @Test
    void testSwapInvalidEmitter() {
        when(token.tokenContract.mock.balanceOf(user1.getAddress())).thenReturn(totalSupply);

        SwapOrderData swapOrder = new SwapOrderData();
        swapOrder.id = BigInteger.valueOf(1);
        swapOrder.emitter = user1.getAddress().toString();
        swapOrder.srcNID = srcNid;
        swapOrder.dstNID = destinationNetwork;
        swapOrder.creator = deployer.getAddress().toString();
        swapOrder.destinationAddress = user2.getAddress().toString();
        swapOrder.token = token.tokenContract.getAddress().toString();
        swapOrder.amount = amount;
        swapOrder.toToken = toToken;
        swapOrder.toAmount = toAmount;
        swapOrder.data = data;

        UserRevertedException exception = assertThrows(UserRevertedException.class, () -> {
            intent.invoke(deployer, "swap", swapOrder);
        });

        assertEquals("Reverted(0): Emitter specified is not this", exception.getMessage());
    }

    @Test
    void testFillOrder() {

        when(token.tokenContract.mock.balanceOf(deployer.getAddress())).thenReturn(totalSupply);
        when(token.tokenContract.mock.balanceOf(user1.getAddress())).thenReturn(BigInteger.ZERO);
        when(token.tokenContract.mock.balanceOf(user2.getAddress())).thenReturn(BigInteger.ZERO);

        assertEquals(totalSupply, token.tokenContract.mock.balanceOf(deployer.getAddress()));

        boolean success = token.tokenContract.mock.transfer(user1.getAddress(), totalSupply);
        assertTrue(success);

        when(token.tokenContract.mock.balanceOf(user1.getAddress())).thenReturn(totalSupply);
        assertEquals(totalSupply, token.tokenContract.mock.balanceOf(user1.getAddress()));
        when(token.tokenContract.mock.balanceOf(deployer.getAddress())).thenReturn(BigInteger.ZERO);
        assertEquals(BigInteger.ZERO, token.tokenContract.mock.balanceOf(deployer.getAddress()));

        SwapOrderData swapOrderData = new SwapOrderData();
        swapOrderData.id = BigInteger.valueOf(1);
        swapOrderData.emitter = intent.getAddress().toString();
        swapOrderData.srcNID = srcNid;
        swapOrderData.dstNID = destinationNetwork;
        swapOrderData.creator = user1.getAddress().toString();
        swapOrderData.destinationAddress = user2.getAddress().toString();
        swapOrderData.token = token.tokenContract.getAddress().toString();
        swapOrderData.amount = amount;
        swapOrderData.toToken = token.tokenContract.getAddress().toString();
        swapOrderData.toAmount = toAmount;
        swapOrderData.data = data;

        intent.invoke(user1, "swap", swapOrderData);

        when(token.tokenContract.mock.balanceOf(solver.getAddress())).thenReturn(totalSupply);

        token.tokenContract.mock.approve(intent.getAddress(), amount);

        when(token.tokenContract.mock.allowance(user1.getAddress(), intent.getAddress())).thenReturn(amount);

        // SwapOrder swapOrder = new SwapOrder(
        // swapOrderData.id, swapOrderData.emitter, swapOrderData.srcNID,
        // swapOrderData.dstNID, swapOrderData.creator,
        // swapOrderData.destinationAddress,
        // swapOrderData.token, swapOrderData.amount, swapOrderData.toToken,
        // swapOrderData.toAmount, swapOrderData.data);

        // OrderFill orderFill = new OrderFill(swapOrder.id, swapOrder.toBytes(),
        // solver.getAddress().toString());
        // OrderMessage orderMessage = new OrderMessage(BigInteger.valueOf(1),
        // orderFill.toBytes());

        intent.invoke(user1, "fill", swapOrderData, solver.getAddress().toString());
    }

    @Test
    void testFillOrderAlreadyFilled() {

        when(token.tokenContract.mock.balanceOf(deployer.getAddress())).thenReturn(totalSupply);
        when(token.tokenContract.mock.balanceOf(user1.getAddress())).thenReturn(BigInteger.ZERO);
        when(token.tokenContract.mock.balanceOf(user2.getAddress())).thenReturn(BigInteger.ZERO);

        assertEquals(totalSupply, token.tokenContract.mock.balanceOf(deployer.getAddress()));

        boolean success = token.tokenContract.mock.transfer(user1.getAddress(), totalSupply);
        assertTrue(success);

        when(token.tokenContract.mock.balanceOf(user1.getAddress())).thenReturn(totalSupply);
        assertEquals(totalSupply, token.tokenContract.mock.balanceOf(user1.getAddress()));
        when(token.tokenContract.mock.balanceOf(deployer.getAddress())).thenReturn(BigInteger.ZERO);
        assertEquals(BigInteger.ZERO, token.tokenContract.mock.balanceOf(deployer.getAddress()));

        SwapOrderData swapOrderData = new SwapOrderData();
        swapOrderData.id = BigInteger.valueOf(1);
        swapOrderData.emitter = intent.getAddress().toString();
        swapOrderData.srcNID = srcNid;
        swapOrderData.dstNID = destinationNetwork;
        swapOrderData.creator = user1.getAddress().toString();
        swapOrderData.destinationAddress = user2.getAddress().toString();
        swapOrderData.token = token.tokenContract.getAddress().toString();
        swapOrderData.amount = amount;
        swapOrderData.toToken = token.tokenContract.getAddress().toString();
        swapOrderData.toAmount = toAmount;
        swapOrderData.data = data;

        intent.invoke(user1, "swap", swapOrderData);

        when(token.tokenContract.mock.balanceOf(solver.getAddress())).thenReturn(totalSupply);

        token.tokenContract.mock.approve(intent.getAddress(), amount);

        when(token.tokenContract.mock.allowance(user1.getAddress(), intent.getAddress())).thenReturn(amount);

        // SwapOrder swapOrder = new SwapOrder(
        // swapOrderData.id, swapOrderData.emitter, swapOrderData.srcNID,
        // swapOrderData.dstNID, swapOrderData.creator,
        // swapOrderData.destinationAddress,
        // swapOrderData.token, swapOrderData.amount, swapOrderData.toToken,
        // swapOrderData.toAmount, swapOrderData.data);

        // OrderFill orderFill = new OrderFill(swapOrder.id, swapOrder.toBytes(),
        // solver.getAddress().toString());
        // OrderMessage orderMessage = new OrderMessage(BigInteger.valueOf(1),
        // orderFill.toBytes());

        intent.invoke(user1, "fill", swapOrderData, solver.getAddress().toString());

        UserRevertedException exception = assertThrows(UserRevertedException.class, () -> {
            intent.invoke(user1, "fill", swapOrderData, solver.getAddress().toString());
        });

        assertEquals("Reverted(0): Order has already been filled", exception.getMessage());
    }

    @Test
    void testFillOrderSameChain() {
        when(token.tokenContract.mock.balanceOf(deployer.getAddress())).thenReturn(totalSupply);
        when(token.tokenContract.mock.balanceOf(user1.getAddress())).thenReturn(BigInteger.ZERO);
        when(token.tokenContract.mock.balanceOf(user2.getAddress())).thenReturn(BigInteger.ZERO);

        assertEquals(totalSupply, token.tokenContract.mock.balanceOf(deployer.getAddress()));

        boolean success = token.tokenContract.mock.transfer(user1.getAddress(), totalSupply);
        assertTrue(success);

        when(token.tokenContract.mock.balanceOf(user1.getAddress())).thenReturn(totalSupply);
        assertEquals(totalSupply, token.tokenContract.mock.balanceOf(user1.getAddress()));
        when(token.tokenContract.mock.balanceOf(deployer.getAddress())).thenReturn(BigInteger.ZERO);
        assertEquals(BigInteger.ZERO, token.tokenContract.mock.balanceOf(deployer.getAddress()));

        SwapOrderData swapOrderData = new SwapOrderData();
        swapOrderData.id = BigInteger.valueOf(1);
        swapOrderData.emitter = intent.getAddress().toString();
        swapOrderData.srcNID = srcNid;
        swapOrderData.dstNID = srcNid;
        swapOrderData.creator = user1.getAddress().toString();
        swapOrderData.destinationAddress = user2.getAddress().toString();
        swapOrderData.token = token.tokenContract.getAddress().toString();
        swapOrderData.amount = amount;
        swapOrderData.toToken = token.tokenContract.getAddress().toString();
        swapOrderData.toAmount = toAmount;
        swapOrderData.data = data;

        intent.invoke(user1, "swap", swapOrderData);

        when(token.tokenContract.mock.balanceOf(solver.getAddress())).thenReturn(totalSupply);

        token.tokenContract.mock.approve(intent.getAddress(), amount);

        when(token.tokenContract.mock.allowance(user1.getAddress(), intent.getAddress())).thenReturn(amount);

        // SwapOrder swapOrder = new SwapOrder(
        // swapOrderData.id, swapOrderData.emitter, swapOrderData.srcNID,
        // swapOrderData.dstNID, swapOrderData.creator,
        // swapOrderData.destinationAddress,
        // swapOrderData.token, swapOrderData.amount, swapOrderData.toToken,
        // swapOrderData.toAmount, swapOrderData.data);

        // OrderFill orderFill = new OrderFill(swapOrder.id, swapOrder.toBytes(),
        // solver.getAddress().toString());
        // OrderMessage orderMessage = new OrderMessage(BigInteger.valueOf(1),
        // orderFill.toBytes());

        intent.invoke(user1, "fill", swapOrderData, solver.getAddress().toString());
    }

    @Test
    void testCancelOrder() {
        when(token.tokenContract.mock.balanceOf(deployer.getAddress())).thenReturn(totalSupply);
        when(token.tokenContract.mock.balanceOf(user1.getAddress())).thenReturn(BigInteger.ZERO);
        when(token.tokenContract.mock.balanceOf(user2.getAddress())).thenReturn(BigInteger.ZERO);

        assertEquals(totalSupply, token.tokenContract.mock.balanceOf(deployer.getAddress()));

        boolean success = token.tokenContract.mock.transfer(user1.getAddress(), totalSupply);
        assertTrue(success);

        when(token.tokenContract.mock.balanceOf(user1.getAddress())).thenReturn(totalSupply);
        assertEquals(totalSupply, token.tokenContract.mock.balanceOf(user1.getAddress()));
        when(token.tokenContract.mock.balanceOf(deployer.getAddress())).thenReturn(BigInteger.ZERO);
        assertEquals(BigInteger.ZERO, token.tokenContract.mock.balanceOf(deployer.getAddress()));

        SwapOrderData swapOrderData = new SwapOrderData();
        swapOrderData.id = BigInteger.valueOf(1);
        swapOrderData.emitter = intent.getAddress().toString();
        swapOrderData.srcNID = srcNid;
        swapOrderData.dstNID = destinationNetwork;
        swapOrderData.creator = user1.getAddress().toString();
        swapOrderData.destinationAddress = user2.getAddress().toString();
        swapOrderData.token = token.tokenContract.getAddress().toString();
        swapOrderData.amount = amount;
        swapOrderData.toToken = token.tokenContract.getAddress().toString();
        swapOrderData.toAmount = toAmount;
        swapOrderData.data = data;

        intent.invoke(user1, "swap", swapOrderData);

        BigInteger beforeCancelConn = (BigInteger) intent.call("getConnSn");

        intent.invoke(user1, "cancel", swapOrderData.id);

        BigInteger afterCancelConn = (BigInteger) intent.call("getConnSn");

        assertEquals(beforeCancelConn, afterCancelConn.subtract(BigInteger.valueOf(1)));
    }

    @Test
    void testCancelOrderOnlyCreator() {
        when(token.tokenContract.mock.balanceOf(deployer.getAddress())).thenReturn(totalSupply);
        when(token.tokenContract.mock.balanceOf(user1.getAddress())).thenReturn(BigInteger.ZERO);
        when(token.tokenContract.mock.balanceOf(user2.getAddress())).thenReturn(BigInteger.ZERO);

        assertEquals(totalSupply, token.tokenContract.mock.balanceOf(deployer.getAddress()));

        boolean success = token.tokenContract.mock.transfer(user1.getAddress(), totalSupply);
        assertTrue(success);

        when(token.tokenContract.mock.balanceOf(user1.getAddress())).thenReturn(totalSupply);
        assertEquals(totalSupply, token.tokenContract.mock.balanceOf(user1.getAddress()));
        when(token.tokenContract.mock.balanceOf(deployer.getAddress())).thenReturn(BigInteger.ZERO);
        assertEquals(BigInteger.ZERO, token.tokenContract.mock.balanceOf(deployer.getAddress()));

        SwapOrderData swapOrderData = new SwapOrderData();
        swapOrderData.id = BigInteger.valueOf(1);
        swapOrderData.emitter = intent.getAddress().toString();
        swapOrderData.srcNID = srcNid;
        swapOrderData.dstNID = destinationNetwork;
        swapOrderData.creator = user1.getAddress().toString();
        swapOrderData.destinationAddress = user2.getAddress().toString();
        swapOrderData.token = token.tokenContract.getAddress().toString();
        swapOrderData.amount = amount;
        swapOrderData.toToken = token.tokenContract.getAddress().toString();
        swapOrderData.toAmount = toAmount;
        swapOrderData.data = data;

        intent.invoke(user1, "swap", swapOrderData);

        UserRevertedException exception = assertThrows(UserRevertedException.class, () -> {
            intent.invoke(deployer, "cancel", swapOrderData.id);
        });

        assertEquals("Reverted(0): Only creator can cancel this order", exception.getMessage());
    }

    @Test
    void testResolveOrder() {
        when(token.tokenContract.mock.balanceOf(deployer.getAddress())).thenReturn(totalSupply);
        when(token.tokenContract.mock.balanceOf(user1.getAddress())).thenReturn(BigInteger.ZERO);
        when(token.tokenContract.mock.balanceOf(user2.getAddress())).thenReturn(BigInteger.ZERO);

        assertEquals(totalSupply, token.tokenContract.mock.balanceOf(deployer.getAddress()));

        boolean success = token.tokenContract.mock.transfer(user1.getAddress(), totalSupply);
        assertTrue(success);

        when(token.tokenContract.mock.balanceOf(user1.getAddress())).thenReturn(totalSupply);
        assertEquals(totalSupply, token.tokenContract.mock.balanceOf(user1.getAddress()));
        when(token.tokenContract.mock.balanceOf(deployer.getAddress())).thenReturn(BigInteger.ZERO);
        assertEquals(BigInteger.ZERO, token.tokenContract.mock.balanceOf(deployer.getAddress()));

        SwapOrderData swapOrderData = new SwapOrderData();
        swapOrderData.id = BigInteger.valueOf(1);
        swapOrderData.emitter = intent.getAddress().toString();
        swapOrderData.srcNID = srcNid;
        swapOrderData.dstNID = destinationNetwork;
        swapOrderData.creator = user1.getAddress().toString();
        swapOrderData.destinationAddress = user2.getAddress().toString();
        swapOrderData.token = token.tokenContract.getAddress().toString();
        swapOrderData.amount = amount;
        swapOrderData.toToken = token.tokenContract.getAddress().toString();
        swapOrderData.toAmount = toAmount;
        swapOrderData.data = data;

        intent.invoke(user1, "swap", swapOrderData);

        when(token.tokenContract.mock.balanceOf(solver.getAddress())).thenReturn(totalSupply);

        token.tokenContract.mock.approve(intent.getAddress(), amount);

        when(token.tokenContract.mock.allowance(user1.getAddress(), intent.getAddress())).thenReturn(amount);

        SwapOrder swapOrder = new SwapOrder(
                swapOrderData.id, swapOrderData.emitter, swapOrderData.srcNID,
                swapOrderData.dstNID, swapOrderData.creator, swapOrderData.destinationAddress,
                swapOrderData.token, swapOrderData.amount, swapOrderData.toToken,
                swapOrderData.toAmount, swapOrderData.data);

        OrderFill orderFill = new OrderFill(swapOrder.id, swapOrder.toBytes(), solver.getAddress().toString());
        OrderMessage orderMessage = new OrderMessage(BigInteger.valueOf(1), orderFill.toBytes());

        intent.invoke(user1, "fill", swapOrderData, solver.getAddress().toString());

        intent.invoke(relayAddress, "recvMessage", destinationNetwork, 1, orderMessage.toBytes());

        BigInteger conn = (BigInteger) intent.call("getConnSn");

        assertEquals(intent.call("getReceipt", destinationNetwork, conn), true);
    }

    @Test
    void testResolveOrderMisMatch() {
        when(token.tokenContract.mock.balanceOf(deployer.getAddress())).thenReturn(totalSupply);
        when(token.tokenContract.mock.balanceOf(user1.getAddress())).thenReturn(BigInteger.ZERO);
        when(token.tokenContract.mock.balanceOf(user2.getAddress())).thenReturn(BigInteger.ZERO);

        assertEquals(totalSupply, token.tokenContract.mock.balanceOf(deployer.getAddress()));

        boolean success = token.tokenContract.mock.transfer(user1.getAddress(), totalSupply);
        assertTrue(success);

        when(token.tokenContract.mock.balanceOf(user1.getAddress())).thenReturn(totalSupply);
        assertEquals(totalSupply, token.tokenContract.mock.balanceOf(user1.getAddress()));
        when(token.tokenContract.mock.balanceOf(deployer.getAddress())).thenReturn(BigInteger.ZERO);
        assertEquals(BigInteger.ZERO, token.tokenContract.mock.balanceOf(deployer.getAddress()));

        SwapOrderData swapOrderData = new SwapOrderData();
        swapOrderData.id = BigInteger.valueOf(1);
        swapOrderData.emitter = intent.getAddress().toString();
        swapOrderData.srcNID = srcNid;
        swapOrderData.dstNID = destinationNetwork;
        swapOrderData.creator = user1.getAddress().toString();
        swapOrderData.destinationAddress = user2.getAddress().toString();
        swapOrderData.token = token.tokenContract.getAddress().toString();
        swapOrderData.amount = amount;
        swapOrderData.toToken = token.tokenContract.getAddress().toString();
        swapOrderData.toAmount = toAmount;
        swapOrderData.data = data;

        intent.invoke(user1, "swap", swapOrderData);

        when(token.tokenContract.mock.balanceOf(solver.getAddress())).thenReturn(totalSupply);

        token.tokenContract.mock.approve(intent.getAddress(), amount);

        when(token.tokenContract.mock.allowance(user1.getAddress(), intent.getAddress())).thenReturn(amount);

        SwapOrder swapOrder = new SwapOrder(
                swapOrderData.id, swapOrderData.emitter, swapOrderData.srcNID,
                swapOrderData.dstNID, swapOrderData.creator, swapOrderData.destinationAddress,
                swapOrderData.token, swapOrderData.amount, swapOrderData.toToken,
                swapOrderData.toAmount, swapOrderData.data);

        OrderFill orderFill = new OrderFill(swapOrder.id, swapOrder.toBytes(), solver.getAddress().toString());
        OrderMessage orderMessage = new OrderMessage(BigInteger.valueOf(1), orderFill.toBytes());

        intent.invoke(user1, "fill", swapOrderData, solver.getAddress().toString());

        UserRevertedException exception = assertThrows(UserRevertedException.class, () -> {
            intent.invoke(relayAddress, "recvMessage", "dummy", 1, orderMessage.toBytes());
        });

        assertEquals("Reverted(0): Invalid Network", exception.getMessage());

    }

    @Test
    void testResolveCancel() {
        when(token.tokenContract.mock.balanceOf(deployer.getAddress())).thenReturn(totalSupply);
        when(token.tokenContract.mock.balanceOf(user1.getAddress())).thenReturn(BigInteger.ZERO);
        when(token.tokenContract.mock.balanceOf(user2.getAddress())).thenReturn(BigInteger.ZERO);

        assertEquals(totalSupply, token.tokenContract.mock.balanceOf(deployer.getAddress()));

        boolean success = token.tokenContract.mock.transfer(user1.getAddress(), totalSupply);
        assertTrue(success);

        when(token.tokenContract.mock.balanceOf(user1.getAddress())).thenReturn(totalSupply);
        assertEquals(totalSupply, token.tokenContract.mock.balanceOf(user1.getAddress()));
        when(token.tokenContract.mock.balanceOf(deployer.getAddress())).thenReturn(BigInteger.ZERO);
        assertEquals(BigInteger.ZERO, token.tokenContract.mock.balanceOf(deployer.getAddress()));

        SwapOrderData swapOrderData = new SwapOrderData();
        swapOrderData.id = BigInteger.valueOf(1);
        swapOrderData.emitter = intent.getAddress().toString();
        swapOrderData.srcNID = srcNid;
        swapOrderData.dstNID = destinationNetwork;
        swapOrderData.creator = user1.getAddress().toString();
        swapOrderData.destinationAddress = user2.getAddress().toString();
        swapOrderData.token = token.tokenContract.getAddress().toString();
        swapOrderData.amount = amount;
        swapOrderData.toToken = token.tokenContract.getAddress().toString();
        swapOrderData.toAmount = toAmount;
        swapOrderData.data = data;

        intent.invoke(user1, "swap", swapOrderData);

        when(token.tokenContract.mock.balanceOf(solver.getAddress())).thenReturn(totalSupply);

        token.tokenContract.mock.approve(intent.getAddress(), amount);

        when(token.tokenContract.mock.allowance(user1.getAddress(), intent.getAddress())).thenReturn(amount);

        SwapOrder swapOrder = new SwapOrder(
                swapOrderData.id, swapOrderData.emitter, swapOrderData.srcNID,
                swapOrderData.dstNID, swapOrderData.creator, swapOrderData.destinationAddress,
                swapOrderData.token, swapOrderData.amount, swapOrderData.toToken,
                swapOrderData.toAmount, swapOrderData.data);

        Cancel cancel = new Cancel();
        cancel.orderBytes = swapOrder.toBytes();

        OrderMessage orderMessage = new OrderMessage(Constant.CANCEL, cancel.toBytes());

        // OrderFill orderFill = new OrderFill(swapOrder.id, swapOrder.toBytes(),
        // swapOrder.creator);

        // OrderMessage fillMessage = new OrderMessage(Constant.FILL,
        // orderFill.toBytes());

        intent.invoke(relayAddress, "recvMessage", srcNid, 1,
                orderMessage.toBytes());

        boolean isFinished = (boolean) intent.call("getFinishedorders",
                Context.hash("keccak-256", swapOrder.toBytes()));
        assertTrue(isFinished);
    }

    @Test
    void testSetFeeHandler() {
        Account feeHandler = sm.createAccount();
        intent.invoke(deployer, "setFeeHandler", feeHandler.getAddress());

        assertEquals(feeHandler.getAddress(), intent.call("getFeeHandler"));
    }

    @Test
    void testSetFeeHandlerNonAdmin() {
        Account feeHandler = sm.createAccount();

        UserRevertedException exception = assertThrows(UserRevertedException.class,
                () -> {
                    intent.invoke(user1, "setFeeHandler", feeHandler.getAddress());
                });

        assertEquals("Reverted(0): Not Owner", exception.getMessage());
    }

    @Test
    void testSetProtocol() {
        intent.invoke(deployer, "setProtocolFee", TEN);

        assertEquals(TEN, intent.call("getProtocolFee"));
    }

    @Test
    void testSetProtocolFeeNonAdmin() {

        UserRevertedException exception = assertThrows(UserRevertedException.class,
                () -> {
                    intent.invoke(user1, "setProtocolFee", TEN);
                });

        assertEquals("Reverted(0): Not Owner", exception.getMessage());
    }

}
