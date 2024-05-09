## Solidity XCall Contracts

This repo contains the smart contracts for EVM CallService contracts in solidity.

### Build and Deployment Setups

1. **Prerequisites**:
   - Install Foundry and Node.js (v21.1.1) on your machine.

2. **Setup**
    ```bash
    git clone git@github.com:icon-project/xcall-multi.git
    cd xcall-multi/contracts/evm
    ```

3. **Install the project dependencies**
    ```bash
    forge install
    ```

4. **Compile your Solidity contracts**

    ```bash
    $ forge build
    ```

5. **Testing**
    Before deploying, it's recommended to run tests to ensure your contracts work as expected:

    ```bash
    $ forge test -vv
    ```

6. **Deployment Environment Setups**

    - Create a new `.env` file by copying the `.env.example` file:
     ```bash
        cp .env.example .env
    ```

    - Open the .env file and update the following values:
    ```env
    PRIVATE_KEY: Your private key with the "0x" prefix.
    
    ADMIN: The address that should be set as the admin for the contracts.
    ```

    When deploying adapters/mock_dapps
    * <CHAIN>_XCALL: xcall address of specific chains you are gonna deploy
    * <CHAIN>_CENTRALIZED_RELAYER: address of centralized relayer(only when ddeploying centralized adapter)

    You can also set other environment variables as needed, such as RPC URLs, NID (Network IDs), and contract addresses.

    *Note: Have a complete look at the env once, you will understand the need to create and populate the `.env` file or set the required environment variables before deploying the contracts.*

7. **Deployment**

    ```shell
    ./deploy_script.sh --contract <contract> --<action> --env <environment> --chain <chain1> <chain2> ... --version <filename-version>
    ```

    Adapter(Wormhole and Layerzero) Configuration between 2 chains
    ```shell
    ./deploy_script.sh --contract <contract> --configure --env <environment> --chain <chain1> <chain2> 
    ```
    Replace the placeholders with your specific values:

    - `<contract>`: Contract to deploy or upgrade
    - `<action>`: Choose either "--deploy" to deploy contracts or "--upgrade" to upgrade existing contracts.
    - `<environment>`: Select the deployment environment ("mainnet," "testnet," or "local").
    - `<chain1>`, `<chain2>`, ...: Specify one or more chains for deployment. Use "all" to deploy to all valid chains for the environment.
    - `filename-version`: filename of new contract to upgrade like, CallServiceV2.sol (only needed in upgrade)

    Valid Options

    - *Actions*: "deploy", "upgrade"
    - *Environments*: "mainnet", "testnet", "local"
    - *Contract Types*: "callservice" "wormhole" "layerzero" "centralized" "mock"

    i. **Local Deployment**
    - Start a local Anvil node
        ```bash
        anvil
        ```
    - Make sure you have `PRIVATE_KEY` and `ADMIN` addresses provided by Avnil
    - In a new terminal window, navigate to your `xcall-multi/contracts/evm` and run the deployment script

        ```bash
        ./deploy_script.sh --contract callservice --deploy --env local --chain local
        ```
    ii. **Testnet Deployment**
    - Deploy the "callservice" contract to testnet on sepolia and optimism chains:

        ```shell
        ./deploy_script.sh --contract callservice --deploy --env testnet --chain sepolia optimism_sepolia
        ```

     *Testnet Chains Available*: "sepolia" "bsctest" "fuji" "base_sepolia" "optimism_sepolia" "arbitrum_sepolia" "all"

    iii. **Mainnet Deployment**

    - Deploy the "centralized" adapter contract to mainnet on Ethereum and Binance chains:

        ```shell
        ./deploy_script.sh --contract centralized --deploy --env mainnet --chain ethereum binance
        ```

    *Mainnet Chains Available: "ethereum" "binance" "avalanche" "base" "optimism" "arbitrum" "all"*

### Upgrading a Contract

To upgrade an existing contract, you need to create a new file with the updated changes and specify the previous contract version in the new contract file. Follow these steps:

1. **Rename the Old Contract File**:
  - Rename the old version of the contract file. For example, if you have a contract named `CallService.sol`, rename it to `CallServiceV1.sol`.

2. **Create a New Contract File**:
  - Create a new file with the updated contract changes. For example, create a new file named `CallService.sol`.

3. **Add the Upgrade Annotation**:
  - In the new contract file (`CallService.sol`), add the following line just above the contract declaration:

  ```solidity
  /// @custom:oz-upgrades-from contracts/xcall/CallServiceV1.sol:CallServiceV1
  contract CallService is IBSH, ICallService, IFeeManage, Initializable {
      // Contract code goes here
  }
  ```
  This annotation specifies the previous contract version from which the upgrade is being performed.

4. **Ensure Proxy Contract Addresses:**
 - Make sure you have the proxy contract addresses for the chains and contracts you want to upgrade in your .env file.

5. **Run the Upgrade Script:**
 - With the admin account you provided during the initial deployment, run the following script:

```shell
./deploy_script.sh --contract <contract> --upgrade --env <environment> --chain <chain1> <chain2> ... --version <filename-version>
```

#### Upgrade the "callservice" contract to testnet on sepolia and fuji:

```shell
./deploy_script.sh --contract callservice --upgrade --env testnet --chain sepolia fuji --version CallService.sol
```

### xCall Configurations

```shell
cast send <contract_address>  "setProtocolFee(uint256 _value)" <value> --rpc-url <rpc_url> --private-key  <private-key>
```

```shell
cast send <contract_address>  "setProtocolFeeHandler(address _addr)" <addr> --rpc-url <rpc_url> --private-key <private-key>
```

```shell
cast send <contract_address>  "setDefaultConnection(string memory _nid,address connection)" <nid> <connection> --rpc-url <rpc_url> --private-key <private-key>
```

### xCall Flow Test between 2 Chains (Only in EVM)

#### Step 0: Copy Environment File

Start by copying the `.env.example` file to `.env`:

```bash
cp .env.example .env
```

#### Step 1: Deploy Contracts

```bash
#deploy xcall
./deploy_script.sh --contract callservice --deploy --env testnet --chain <source_chain> <destination_chain> 
```

```bash
#deploy adapter (wormhole layerzero centralized)
./deploy_script.sh --contract wormhole --deploy --env testnet --chain <source_chain> <destination_chain> 
```

```bash
#deploy dapp
./deploy_script.sh --contract mock --deploy --env testnet --chain <source_chain> <destination_chain> 
```
#### Step 3: Configure Connections

If you are using Wormhole or Layerzero for cross-chain communication, you will need to configure the connection. Execute the provided script to set up the connection. 

```bash
./deploy_script.sh --contract <adapter> --configure --env testnet --chain <source_chain> <destination_chain>
```
- *adapter*: "layerzero", "wormhole", "centralized"

#### Step 4: Add Connections in Dapp

In your dapp, add the connections for cross-chain communication.

```bash
forge script DeployCallService  -s "addConnection(string memory chain1, string memory chain2)" <source_chain> <destination_chain> --fork-url <source_chain> --broadcast        
forge script DeployCallService  -s "addConnection(string memory chain1, string memory chain2)" <destination_chain> <source_chain> --fork-url <destination_chain> --broadcast   
```

#### Step 5: Execute Test
```bash
$ ./test_xcall_flow.sh --src <source_chain> --dest <destination_chain> --fee <value>
```

- `--fee <value>`: Sets the transaction fee (in wei). The value must be a number.
- `--src <source_chain>`: Sets the source chain for the transaction. Valid chain options are `fuji`, `bsctest`, `arbitrum_sepolia`, `optimism_sepolia`.
- `--dest <destination_chain>`: Sets the destination chain for the transaction. Valid chain options are `fuji`, `bsctest`, `base_sepolia`, `optimism_sepolia`, and `arbitrum_sepolia`.
