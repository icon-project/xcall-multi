package intent_contracts.mocks;

import com.iconloop.score.test.Account;
import com.iconloop.score.test.ServiceManager;
import intent_contracts.interfaces.TokenInterface;
import score.Address;

import static org.mockito.ArgumentMatchers.any;
import static org.mockito.Mockito.when;

import java.math.BigInteger;

import static java.math.BigInteger.TEN;

public class MockToken {
    public MockContract<TokenInterface> tokenContract;
    public BigInteger amount = pow(BigInteger.valueOf(500 * 10), 18);
    private static final BigInteger initialSupply = BigInteger.valueOf(1000);
    private static final BigInteger totalSupply = initialSupply.multiply(TEN.pow(18));

    public MockToken(ServiceManager sm, Account owner) throws Exception {
        tokenContract = new MockContract<>(TokenInterface.class);

        tokenContract.deploy(sm, owner);

        when(tokenContract.mock.symbol()).thenReturn("MYTOKEN");
        when(tokenContract.mock.name()).thenReturn("My Custom Token");
        when(tokenContract.mock.decimals()).thenReturn(BigInteger.valueOf(18));
        when(tokenContract.mock.totalSupply()).thenReturn(totalSupply);
        when(tokenContract.mock.balanceOf(any(Address.class))).thenReturn(BigInteger.ZERO);
        when(tokenContract.mock.transfer(any(Address.class), any(BigInteger.class))).thenReturn(true);
        when(tokenContract.mock.approve(any(Address.class), any(BigInteger.class))).thenReturn(true);
        when(tokenContract.mock.allowance(any(Address.class), any(Address.class))).thenReturn(BigInteger.ZERO);
        when(tokenContract.mock.transferFrom(any(Address.class), any(Address.class), any(BigInteger.class)))
                .thenReturn(true);
    }

    private static BigInteger pow(BigInteger base, int exponent) {
        BigInteger result = BigInteger.ONE;
        for (int i = 0; i < exponent; i++) {
            result = result.multiply(base);
        }
        return result;
    }
}
