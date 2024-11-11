package intent_contracts.interfaces;

import score.Address;
import java.math.BigInteger;

public interface TokenInterface {

    BigInteger totalSupply();

    String name();

    String symbol();

    BigInteger decimals();

    BigInteger balanceOf(Address account);

    boolean transfer(Address to, BigInteger amount);

    boolean approve(Address spender, BigInteger amount);

    BigInteger allowance(Address owner, Address spender);

    boolean transferFrom(Address from, Address to, BigInteger amount);

}
