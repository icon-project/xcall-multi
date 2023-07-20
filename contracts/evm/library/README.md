# Library for BTP2 Solidity

It includes shared codes between BTP2 Solidity Contracts.

## Use this library from other contract

In the contract of the local repository, it refers this module directly.

For `yarn` package system, use the following command if it's located in
the root of the repository.

```shell
yarn add file:../library
```

Use proper relative path for it in other locations.

In the contract of other repositories, use absolute name to refer it.

```shell
yarn add @iconfoundation/btp2-solidity-library
```

## Development

### Setup

It uses `yarn` for package management. Use the following command to install
related packages.

```shell
yarn install
```

### Compile

To compile codes, run the following command.

```shell
npx hardhat compile
```

### Unit test

Add unit test cases in "test" directory
To run the test, run the following command.

```shell
npx hardhat test
```

To run specific test case in the file, then append the path of the file.
Here is the example to run test cases in `./test/RLPCodec.ts`

```shell
npx hardhat test ./test/RLPCodec.ts
```

### Test in other contracts

When you modify this library, you may need to apply this directly
to the other contracts. To do that. Use following

Run following command in the root of this module.
```shell
yarn link
````

Then run following command in the root of other repository
```shell
yarn link @iconfoundation/btp2-solidity-library
```