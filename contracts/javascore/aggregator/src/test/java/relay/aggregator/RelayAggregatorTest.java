package relay.aggregator;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.function.Executable;

import score.Address;
import score.UserRevertedException;

import com.iconloop.score.test.Account;
import com.iconloop.score.test.Score;
import com.iconloop.score.test.ServiceManager;
import com.iconloop.score.test.TestBase;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;

class RelayAggregatorTest extends TestBase {
    protected final ServiceManager sm = getServiceManager();

    protected final Account adminAc = sm.createAccount();
    protected final Account relayerAcOne = sm.createAccount();
    protected final Account relayerAcTwo = sm.createAccount();

    protected Score aggregator;

    @BeforeEach
    void setup() throws Exception {
        Address[] relayers = new Address[]{relayerAcOne.getAddress(), relayerAcTwo.getAddress()};
        aggregator = sm.deploy(adminAc, RelayAggregator.class, adminAc.getAddress(), relayers);
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
    
        assertEquals("Reverted(0): Unauthorized to call this method", e.getMessage());
    }
}
