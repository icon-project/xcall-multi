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
        // "LibRLP.sol",
        // "MockBMC.sol",
        // "MockBMV.sol",
        // "MockBSH.sol",
        "interfaces/IMockBMC.sol",
        "interfaces/IMockBMV.sol",
        "interfaces/IMockBSH.sol",
        // "libraries/Errors.sol",
        "test/TestBTPAddress.sol",
        "test/TestWeb3jABI.sol"
    ],
    mocha: {
        timeout: 600000
    }
};
