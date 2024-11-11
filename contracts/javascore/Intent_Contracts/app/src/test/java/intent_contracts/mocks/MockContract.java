package intent_contracts.mocks;

import com.iconloop.score.test.Account;
import com.iconloop.score.test.ServiceManager;
import com.iconloop.score.test.Score;
import org.mockito.Mockito;
import score.Address;

public class MockContract<T> {
    public final T mock;
    private Score deployedScore;

    public MockContract(Class<? extends T> classToMock) {
        mock = Mockito.mock(classToMock);
    }

    public void deploy(ServiceManager sm, Account admin) throws Exception {
        deployedScore = sm.deploy(admin, mock.getClass());
        deployedScore.setInstance(mock);
    }

    public Address getAddress() {
        return deployedScore.getAddress();
    }
}
