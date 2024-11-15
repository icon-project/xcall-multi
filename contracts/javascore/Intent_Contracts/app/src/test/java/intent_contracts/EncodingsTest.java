package intent_contracts;

import java.math.BigInteger;
import java.util.Arrays;

import network.icon.intent.structs.Cancel;
import org.junit.jupiter.api.Test;

import com.iconloop.score.test.TestBase;

import network.icon.intent.constants.Constant;
import network.icon.intent.structs.OrderFill;
import network.icon.intent.structs.OrderMessage;
import network.icon.intent.structs.SwapOrder;

import static org.junit.jupiter.api.Assertions.*;

public class EncodingsTest extends TestBase {

    public static byte[] hexStringToByteArray(String hexString) {
        int length = hexString.length();
        byte[] byteArray = new byte[length / 2];

        for (int i = 0; i < length; i += 2) {
            byteArray[i / 2] = (byte) ((Character.digit(hexString.charAt(i), 16) << 4)
                    + Character.digit(hexString.charAt(i + 1), 16));
        }
        return byteArray;
    }

    private static BigInteger pow(BigInteger base, int exponent) {
        BigInteger result = BigInteger.ONE;
        for (int i = 0; i < exponent; i++) {
            result = result.multiply(base);
        }
        return result;
    }

    public static String byteArrayToHex(byte[] byteArray) {
        StringBuilder hexString = new StringBuilder();
        for (byte b : byteArray) {
            hexString.append(String.format("%02X", b));
        }
        return hexString.toString();
    }

    @Test
    void testSwapOrder() {

        BigInteger id = BigInteger.valueOf(1);
        String emitter = "0xbe6452d4d6c61cee97d3";
        String srcNID = "Ethereum";
        String dstNID = "Polygon";
        String creator = "0x3e36eddd65e239222e7e67";
        String destinationAddress = "0xd2c6218b875457a41b6fb7964e";
        String token = "0x14355340e857912188b7f202d550222487";
        BigInteger amount = BigInteger.valueOf(1000);
        String toToken = "0x91a4728b517484f0f610de7b";
        BigInteger toAmount = BigInteger.valueOf(900);
        String data = "";

        SwapOrder swapOrder1 = new SwapOrder(id, emitter, srcNID, dstNID, creator, destinationAddress, token, amount,
                toToken, toAmount, data);

        byte[] expectedBytes = hexStringToByteArray(
                "f8a601963078626536343532643464366336316365653937643388457468657265756d87506f6c79676f6e983078336533366564646436356532333932323265376536379c30786432633632313862383735343537613431623666623739363465a43078313433353533343065383537393132313838623766323032643535303232323438378203e89a307839316134373238623531373438346630663631306465376282038480");

        assertTrue(Arrays.equals(expectedBytes, swapOrder1.toBytes()));

        SwapOrder order2 = new SwapOrder(
                BigInteger.valueOf(1),
                "0xbe6452d4d6c61cee97d3",
                "Ethereum",
                "Polygon",
                "0x3e36eddd65e239222e7e67",
                "0xd2c6218b875457a41b6fb7964e",
                "0x14355340e857912188b7f202d550222487",
                BigInteger.valueOf(100000).pow(22),
                "0x91a4728b517484f0f610de7b",
                BigInteger.valueOf(900).pow(7),
                "hello1");
        String expectedBytes2 = "f8df01963078626536343532643464366336316365653937643388457468657265756d87506f6c79676f6e983078336533366564646436356532333932323265376536379c30786432633632313862383735343537613431623666623739363465a4307831343335353334306538353739313231383862376632303264353530323232343837ae2a94608f8d29fbb7af52d1bc1667f505440cc75cababdc5961bfcc9f54dadd1a40000000000000000000000000009a30783931613437323862353137343834663066363130646537628919edb3f06ca68840008668656c6c6f31";
        assertEquals(expectedBytes2.toUpperCase(), byteArrayToHex(order2.toBytes()));
    }

    @Test
    void testOrderMessage() {
        OrderMessage orderMessage = new OrderMessage(Constant.FILL,
                hexStringToByteArray("6c449988e2f33302803c93f8287dc1d8cb33848a"));

        byte[] expectedBytes = hexStringToByteArray("d601946c449988e2f33302803c93f8287dc1d8cb33848a");
        assertArrayEquals(expectedBytes, orderMessage.toBytes());

        OrderMessage cancelMessage = new OrderMessage(Constant.CANCEL,
                hexStringToByteArray("6c449988e2f33302803c93f8287dc1d8cb33848a"));

        expectedBytes = hexStringToByteArray("d602946c449988e2f33302803c93f8287dc1d8cb33848a");
        assertTrue(Arrays.equals(expectedBytes, cancelMessage.toBytes()));
    }

    @Test
    void testOrderFill() {
        OrderFill orderFill = new OrderFill(BigInteger.valueOf(1),
                hexStringToByteArray("6c449988e2f33302803c93f8287dc1d8cb33848a"),
                "0xcb0a6bbccfccde6be9f10ae781b9d9b00d6e63");

        byte[] expectedBytes = hexStringToByteArray(
                "f83f01946c449988e2f33302803c93f8287dc1d8cb33848aa830786362306136626263636663636465366265396631306165373831623964396230306436653633");
        assertTrue(Arrays.equals(expectedBytes, orderFill.toBytes()));

        OrderFill orderFill2 = new OrderFill(BigInteger.valueOf(2),
                hexStringToByteArray("cb0a6bbccfccde6be9f10ae781b9d9b00d6e63"),
                "0x6c449988e2f33302803c93f8287dc1d8cb33848a");

        expectedBytes = hexStringToByteArray(
                "f8400293cb0a6bbccfccde6be9f10ae781b9d9b00d6e63aa307836633434393938386532663333333032383033633933663832383764633164386362333338343861");
        assertTrue(Arrays.equals(expectedBytes, orderFill2.toBytes()));
    }

    @Test
    void testOrderCancel() {
        Cancel cancel = new Cancel();
        cancel.orderBytes = hexStringToByteArray("6c449988e2f33302803c93f8287dc1d8cb33848a");

        byte[] expectedBytes = hexStringToByteArray("d5946c449988e2f33302803c93f8287dc1d8cb33848a");
        assertTrue(Arrays.equals(expectedBytes, cancel.toBytes()));
    }
}
