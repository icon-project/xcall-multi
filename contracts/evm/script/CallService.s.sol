// SPDX-License-Identifier: MIT
pragma solidity 0.8.18;
import {Script} from "forge-std/Script.sol";
import {UUPSProxy} from "@xcall/contracts/upgradeable/UUPSProxy.sol";
import {console2} from "forge-std/console2.sol";

import "@xcall/contracts/xcall/CallService.sol";
import "@xcall/contracts/mocks/multi-protocol-dapp/MultiProtocolSampleDapp.sol";
import "@xcall/contracts/adapters/LayerZeroAdapter.sol";
import "@xcall/contracts/adapters/WormholeAdapter.sol";

contract DeployCallService is Script {
    CallService private proxyXcall;
    using Strings for string;

    UUPSProxy internal proxyContract;

    uint256 internal deployerPrivateKey;
    address internal ownerAddress;
    address internal proxyAddress;

    string internal nid;
    string internal iconNid;
    address internal connection;
    uint256 internal protocolFee;

    constructor() {
        deployerPrivateKey = vm.envUint("PRIVATE_KEY");
        ownerAddress = vm.envAddress("OWNER_ADDRESS");
    }

    modifier broadcast(uint256 privateKey) {
        vm.startBroadcast(privateKey);

        _;

        vm.stopBroadcast();
    }

    function capitalizeString(
        string memory input
    ) public pure returns (string memory) {
        bytes memory inputBytes = bytes(input);
        for (uint i = 0; i < inputBytes.length; i++) {
            if (uint8(inputBytes[i]) >= 97 && uint8(inputBytes[i]) <= 122) {
                inputBytes[i] = bytes1(uint8(inputBytes[i]) - 32);
            }
        }
        return string(inputBytes);
    }

    function deployContract(
        string memory env,
        string memory chain
    ) external broadcast(deployerPrivateKey) {
        env = capitalizeString(env);
        chain = capitalizeString(chain);
        iconNid = vm.envString(env.concat("_ICON_NID"));
        connection = vm.envAddress(env.concat("_BMC_ADDRESS"));
        nid = vm.envString(chain.concat("_NID"));

        CallService xcall = new CallService();
        proxyContract = new UUPSProxy(
            address(xcall),
            abi.encodeWithSelector(CallService.initialize.selector, nid)
        );
        console2.log("CallService address:", address(xcall), "\n");
        console2.log(
            "CallService Proxy address:",
            address(proxyContract),
            "\n"
        );

        proxyXcall = CallService(address(proxyContract));
        proxyXcall.setProtocolFee(protocolFee);
        proxyXcall.setProtocolFeeHandler(ownerAddress);
        proxyXcall.setDefaultConnection(iconNid, connection);
    }

    function deployMock(
        string memory chain
    ) public broadcast(deployerPrivateKey) {
        chain = capitalizeString(chain);
        address xcall = vm.envAddress(chain.concat("_XCALL"));

        MultiProtocolSampleDapp mockdapp = new MultiProtocolSampleDapp();

        UUPSProxy proxyMock = new UUPSProxy(
            address(mockdapp),
            abi.encodeWithSelector(
                MultiProtocolSampleDapp.initialize.selector,
                xcall
            )
        );
    }

    function deployLayerZero(
        string memory chain
    ) public broadcast(deployerPrivateKey) {
        chain = capitalizeString(chain);
        address xcall = vm.envAddress(chain.concat("_XCALL"));
        address layerzeroRelayer = vm.envAddress(
            chain.concat("_LAYERZERO_RELAYER")
        );
        LayerZeroAdapter layerzeroAdapter = new LayerZeroAdapter();

        UUPSProxy proxyLayerzeroAdapter = new UUPSProxy(
            address(layerzeroAdapter),
            abi.encodeWithSelector(
                LayerZeroAdapter.initialize.selector,
                layerzeroRelayer,
                xcall
            )
        );
    }

    function deployWormHole(
        string memory chain
    ) public broadcast(deployerPrivateKey) {
        chain = capitalizeString(chain);
        address wormholeRelayer = vm.envAddress(
            chain.concat("_WORMHOLE_RELAYER")
        );
        address xcall = vm.envAddress(chain.concat("_XCALL"));

        WormholeAdapter wormholeAdapter = new WormholeAdapter();

        UUPSProxy proxyWormholeAdapter = new UUPSProxy(
            address(wormholeAdapter),
            abi.encodeWithSelector(
                WormholeAdapter.initialize.selector,
                wormholeRelayer,
                xcall
            )
        );
    }

    // "callservice" "mock" "layerzero" "wormhole"
    function upgradeContract(
        string memory chain,
        string memory contractName
    ) external broadcast(deployerPrivateKey) {
        if (contractName.compareTo("callservice")) {
            proxyAddress = vm.envAddress(
                capitalizeString(chain).concat("_XCALL")
            );
            CallService xcall = new CallService();
            CallService wrappedProxy = CallService(proxyAddress);
            wrappedProxy.upgradeTo(address(xcall));
        } else if (contractName.compareTo("layerzero")) {
            proxyAddress = vm.envAddress(
                capitalizeString(chain).concat("_LAYERZERO_ADAPTER")
            );
            LayerZeroAdapter layerzero = new LayerZeroAdapter();
            LayerZeroAdapter wrappedProxy = LayerZeroAdapter(payable(proxyAddress));
            wrappedProxy.upgradeTo(address(layerzero));
        } else if (contractName.compareTo("wormhole")) {
            proxyAddress = vm.envAddress(
                capitalizeString(chain).concat("_WORMHOLE_ADAPTER")
            );
            WormholeAdapter wormhole = new WormholeAdapter();
            WormholeAdapter wrappedProxy = WormholeAdapter(proxyAddress);
            wrappedProxy.upgradeTo(address(wormhole));
        }
    }
}
