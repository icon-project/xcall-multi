const hre = require("hardhat");
const accounts =  hre.userConfig.networks.hardhat.accounts.map((v) => {
    return {secretKey: v.privateKey, balance: v.balance}
})

module.exports = {
    istanbulFolder: 'build/hardhat/coverage',
    istanbulReporter: ['html','text'],
    providerOptions: {
        // default_balance_ether: 100,
        accounts: accounts
    },
    skipFiles: [
        // "CallService.sol",
        "interfaces/IFeeManage.sol",
        // "libraries/RLPDecodeStruct.sol",
        // "libraries/RLPEncodeStruct.sol",
        // "libraries/Types.sol",
        "test/DAppProxySample.sol",
        "test/LibRLPStruct.sol"
    ],
    mocha: {
        timeout: 600000
    }
};
