Centralized Connection Deployment
===
## Prerequisities
- git
- foundry
- goloop / gradle

## Steps to deploy centralized connection on mainnet

### Clone the xcall-repository
```sh
git clone https://github.com/icon-project/xcall-multi.git
cd xcall-multi
# checkout to correct version
git checkout v1.2.0
export PROJECT_ROOT=$PWD
```

> NOTE: Relayer is the admin of the centralized contract.


**Follow the instruction to deploy respective contracts**

### Solidity

Assuming you want to deploy centralized to {CHAIN_NAME} chain.

```sh
cd $PROEJCT_ROOT/contracts/evm
forge build
cp env.example .env
```
Edit the .env file. You need to change the following fields:
```env
PRIVATE_KEY=YOUR_PRIVATE_KEY
{CHAIN_NAME}_CENTRALIZED_RELAYER=YOUR_RELAYER_ADDRESS
```

Verify, the xcall address and RPC URL of {CHAIN_NAME} is correct.
The xcall address can be verified from [here](https://github.com/icon-project/xcall-multi/wiki/xCall-Deployment-Info)
```env
{CHAIN_NAME}_XCALL=
{CHAIN_NAME}_RPC=
```

Now, to deploy the centralized-connection contract:
```sh
# check ./deploy_script.sh options for CHAIN_NAME
# env can be mainnet or testnet or local
# ./deploy_script.sh --contract centralized --deploy --env testnet --chain base_goerli
./deploy_script.sh --contract centralized --deploy --env mainnet --chain {CHAIN_NAME} 
```

**Set fees on the connection contract**:

- This can be called only by the relayer.
```sh
cast send <connection_contract_address>  "setProtocolFee(string calldata networkId, uint256 messageFee, uint256 responseFee)" "0x1.icon" 10000000000000000 10000000000000000 --rpc-url <rpc_url> --private-key  <private-key>
```
**Change relayer address**:

- This can only be called by current relayer.
```sh
cast send <connection_contract_address> "setAdmin(address _address)" <new-relayer-address> --rpc-url <rpc_url> --private-key  <private-key>
```

### ICON
You can use one of the following methods to deploy the contract on ICON.

1. **Using gradlew**

```sh
cd $PROJECT_ROOT
cd contracts/javascore/centralized-connection
```

Update the constructor parameters in `build.gradle`. Put correct address on xCall and relayer field. 
```gradle
parameters {
    arg('_relayer', "<your-address>")
    arg('_xCall', "cxa07f426062a1384bdd762afa6a87d123fbc81c75")
}
```

Then, you can deploy it as:

```sh
cd $PROJECT_ROOT/contracts/javascore
./gradlew :centralized-connection:build
./gradlew :centralized-connection:optimizedJar
./gradlew :centralized-connection:deployToMainnet -PkeystoreName=<your_wallet_json> -PkeystorePass=<password>
```


2. **Using goloop**
```sh
# fetch jarfile from release
wget https://github.com/icon-project/xcall-multi/releases/download/v1.2.0/centralized-connection-0.1.0-optimized.jar

# deploy contract
goloop rpc sendtx deploy centralized-connection-0.1.0-optimized.jar \
    --content_type application/java \
    --uri https://ctz.solidwallet.io/api/v3  \
    --nid 1 \
    --step_limit 2200000000 \
    --to cx0000000000000000000000000000000000000000 \
    --param _relayer=<relayer-address> \
    --param _xCall=<xcall-address>\
    --key_store <your_wallet_json> \
    --key_password <password>
```

Now, that the contracts are deployed. You are now ready to setup the relay.
The guide to setup relay is [here](https://github.com/icon-project/centralized-relay/wiki/Installation)