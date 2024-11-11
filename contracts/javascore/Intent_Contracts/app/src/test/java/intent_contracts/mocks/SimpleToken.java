package intent_contracts.mocks;

import score.*;
import score.annotation.EventLog;
import score.annotation.External;
import score.annotation.Optional;

import java.math.BigInteger;

public class SimpleToken {
    private final String name;
    private final String symbol;
    private final int decimals;
    private final VarDB<BigInteger> totalSupply = Context.newVarDB("totalSupply", BigInteger.class);
    private final DictDB<Address, BigInteger> balances = Context.newDictDB("balances", BigInteger.class);
    final static BranchDB<Address, DictDB<Address, BigInteger>> deposit = Context.newBranchDB("deposit",
            BigInteger.class);

    public SimpleToken(String _name, String _symbol, BigInteger _decimals, BigInteger _initialSupply) {
        this.name = _name;
        this.symbol = _symbol;
        this.decimals = _decimals.intValue();

        Context.require(this.decimals >= 0);
        Context.require(this.decimals <= 21);

        Context.require(_initialSupply.compareTo(BigInteger.ZERO) >= 0);

        BigInteger _totalSupply;
        if (_initialSupply.compareTo(BigInteger.ZERO) > 0) {
            BigInteger oneToken = pow(BigInteger.TEN, this.decimals);
            _totalSupply = oneToken.multiply(_initialSupply);
        } else {
            _totalSupply = BigInteger.ZERO;
        }

        this.totalSupply.set(_totalSupply);
        this.balances.set(Context.getCaller(), _totalSupply);
    }

    private static BigInteger pow(BigInteger base, int exponent) {
        BigInteger result = BigInteger.ONE;
        for (int i = 0; i < exponent; i++) {
            result = result.multiply(base);
        }
        return result;
    }

    @External(readonly = true)
    public String name() {
        return name;
    }

    @External(readonly = true)
    public String symbol() {
        return symbol;
    }

    @External(readonly = true)
    public int decimals() {
        return decimals;
    }

    @External(readonly = true)
    public BigInteger totalSupply() {
        return totalSupply.getOrDefault(BigInteger.ZERO);
    }

    @External(readonly = true)
    public BigInteger balanceOf(Address _owner) {
        return safeGetBalance(_owner);
    }

    @External
    public void transfer(Address _to, BigInteger _value, @Optional byte[] _data) {
        Address _from = Context.getCaller();

        Context.require(_value.compareTo(BigInteger.ZERO) >= 0);
        Context.require(safeGetBalance(_from).compareTo(_value) <= 0);

        safeSetBalance(_from, safeGetBalance(_from).subtract(_value));
        safeSetBalance(_to, safeGetBalance(_to).add(_value));

        byte[] dataBytes = (_data == null) ? new byte[0] : _data;
        if (_to.isContract()) {
            Context.call(_to, "fallback", _from, _value, dataBytes);
        }

        Transfer(_from, _to, _value, dataBytes);
    }

    private BigInteger safeGetBalance(Address owner) {
        return balances.getOrDefault(owner, BigInteger.ZERO);
    }

    private void safeSetBalance(Address owner, BigInteger amount) {
        balances.set(owner, amount);
    }

    @EventLog(indexed = 3)
    public void Transfer(Address _from, Address _to, BigInteger _value, byte[] _data) {
    }
}