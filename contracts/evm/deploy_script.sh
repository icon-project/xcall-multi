#!/bin/bash
source .env
# Define valid actions and environments
valid_actions=("deploy" "upgrade")
valid_contracts=("callservice" "mock" "wormhole" "layerzero" "centralized")
valid_environments=("mainnet" "testnet" "local")
valid_mainnet_chains=("ethereum" "binance" "avalanche" "arbitrum" "optimism" "base" "all")
valid_testnet_chains=("sepolia" "bsc_testnet" "fuji" "arbitrum_goerli" "optimism_goerli" "base_goerli" "all")
valid_local_chains=("local" "all")

# Initialize variables
action=""
env=""
chains=()
contract=""
contractVersion=""

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case "$1" in
        --contract)
            shift
            contract="$1"
            ;;
        --version)
            shift
            contractVersion="$1"
            ;;
        --deploy)
            action="deploy"
            ;;
        --upgrade)
            action="upgrade"
            ;;
        --env)
            shift
            env="$1"
            ;;
        --chain)
            shift
            chains=("$@")
            break
            ;;
        *)
            echo "Invalid option: $1"
            exit 1
            ;;
    esac
    shift
done

if [[ ! " ${valid_contracts[@]} " =~ " ${contract} " ]]; then
    echo "Invalid action. Allowed values are: ${valid_contracts[*]}"
    exit 1
fi

if [[ ! " ${valid_actions[@]} " =~ " ${action} " ]]; then
    echo "Invalid action. Allowed values are: ${valid_actions[*]}"
    exit 1
fi

if [[ ! " ${valid_environments[@]} " =~ " ${env} " ]]; then
    echo "Invalid env parameter. Allowed values are: ${valid_environments[*]}"
    exit 1
fi

if [ "$action" == "upgrade" ]; then
    if [ "$contractVersion" == "" ]; then
        echo "Missing contract version, add --version <contract> (like --version CallServiceV2.sol)"
        exit 1
    fi
fi

if [ ${#chains[@]} -eq 0 ]; then
    chains=("all")
fi

if [[ " ${chains[@]} " =~ "all" ]]; then
    if [ "$env" == "local" ]; then
        chains=("local")
    elif [ "$env" == "mainnet" ]; then
        chains=("ethereum" "binance" "avalanche" "arbitrum" "optimism" "base")
    elif [ "$env" == "testnet" ]; then
        chains=("sepolia" "binance_testnet" "fuji" "arbitrum_goerli" "optimism_goerli" "base_goerli")
    fi
fi

valid_chains=()
if [ "$env" == "mainnet" ]; then
    valid_chains=("${valid_mainnet_chains[@]}")
elif [ "$env" == "testnet" ]; then
    valid_chains=("${valid_testnet_chains[@]}")
elif [ "$env" == "local" ]; then
    valid_chains=("${valid_local_chains[@]}")
fi

for chain in "${chains[@]}"; do
    if [[ ! " ${valid_chains[@]} " =~ " ${chain} " ]]; then
        echo "Invalid chain: $chain"
        exit 1
    fi
done

if [ "$action" == "deploy" ]; then
    echo "Deploying $contract on $env:"
    for chain in "${chains[@]}"; do
        echo "Deploying on $chain"
        rm -rf out
        forge script DeployCallService  -s "deployContract(string memory env, string memory chain, string memory contractA)" $env $chain $contract --fork-url $chain --broadcast --sender ${ADMIN} --verify --etherscan-api-key $chain --ffi
    done
elif [ "$action" == "upgrade" ]; then
    echo "Upgrading $contract on $env:"
    for chain in "${chains[@]}"; do
        rm -rf out
        echo "Upgrading on $chain"
        if [ "$contract" == "mock" ]; then
        echo "Mock Contract is not upgradeable!"
        else
        forge script DeployCallService  -s "upgradeContract(string memory chain, string memory contractName, string memory contractA)" $chain $contractVersion $contract --fork-url $chain --broadcast --sender ${ADMIN} --verify --etherscan-api-key $chain --ffi        
        fi
    done
fi
