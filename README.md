[![Project Status: Initial Release](https://img.shields.io/badge/repo%20status-active-green.svg?style=flat-square)](https://www.repostatus.org/#active)
[![License: Apache-2.0](https://img.shields.io/github/license/icon-project/xcall-multi.svg?style=flat-square)](https://github.com/icon-project/xcall-multi/blob/main/LICENSE)
[![Lines Of Code](https://img.shields.io/tokei/lines/github/icon-project/xcall-multi?style=flat-square)](https://github.com/icon-project/xcall-multi)
[![Version](https://img.shields.io/github/tag/icon-project/xcall-multi.svg?style=flat-square)](https://github.com/icon-project/xcall-multi)
![GitHub Workflow Status - cosmwasm](https://github.com/icon-project/xcall-multi/actions/workflows/build-and-publish-cosmwasm.yml/badge.svg)
![GitHub Workflow Status - javascore](https://github.com/icon-project/xcall-multi/actions/workflows/build-and-publish-javascore.yml/badge.svg)
| Language                            | Code Coverage                                              |
| ----------------------------------- | ---------------------------------------------------------- |
| [Java](./contracts/javascore)       | [![Java Cov][java-cov-badge]][java-cov-link]               |
| [Rust](./contracts/cosmwasm-vm)     | [![Rust Cov][rust-cov-badge]][rust-cov-link]               |
| [Solidity](./contracts/evm)         | [![Solidity Cov][solidity-cov-badge]][solidity-cov-link]   |

[java-cov-link]: https://app.codecov.io/gh/icon-project/xcall-multi/tree/main/contracts/javascore
[rust-cov-link]: https://app.codecov.io/gh/icon-project/xcall-multi/tree/main/contracts/cosmwasm-vm
[solidity-cov-link]: https://app.codecov.io/gh/icon-project/xcall-multi/tree/main/contracts/evm
[java-cov-badge]: https://codecov.io/gh/icon-project/xcall-multi/branch/main/graph/badge.svg?token=KWDB59JITE&flag=java
[rust-cov-badge]: https://codecov.io/gh/icon-project/xcall-multi/branch/main/graph/badge.svg?token=KWDB59JITE&flag=rust
[solidity-cov-badge]: https://codecov.io/gh/icon-project/xcall-multi/branch/main/graph/badge.svg?token=KWDB59JITE&flag=solidity

# xcall-multi
xcall-multi is a cross chain messaging service built to mimic regular transaction flows across any interoperable solution.

For full xcall-multi specification see [xcall-multi Spec](./docs/adr/xcall.md).

## Building with xcall-multi
For building dapps with xcall-multi see official developer [docs](https://www.xcall.dev/).

## xcall-multi Contract Address for Repective Chain
| Icon     | Address                                                              |
| -------- | -------------------------------------------------------------------- | 
| Mainnet  | <pre><code> cxa07f426062a1384bdd762afa6a87d123fbc81c75 </pre></code> | 
| Berlin   | <pre><code> cx5b0bd4bb62e2b511aa29586c1e8a21749425d474 </pre></code> |
| Lisbon   | <pre><code> cx15a339fa60bd86225050b22ea8cd4a9d7cd8bb83 </pre></code> |

| Archway        | Address                                                                                      |
| -------------- | -------------------------------------------------------------------------------------------- | 
| archway-1      | <pre><code> archway19hzhgd90etqc3z2qswumq80ag2d8het38r0al0r4ulrly72t20psdrpna6 </pre></code> | 
| constantine-3  | <pre><code> archway1kenxz0wuczr04mc9q3gwjuyzd6ft4zqm5wach846gghfjupvlncshvchs2 </pre></code> |
| constantine-3  | <pre><code> archway1h04c8eqr99dnsw6wqx80juj2vtuxth70eh65cf6pnj4zan6ms4jqshc5wk </pre></code> |


| Neutron        | Address                                                                                      |
| -------------- | -------------------------------------------------------------------------------------------- | 
| neutron-1      | <pre><code> neutron1g28ca6axwkar5fuhhfcgua2807njh795nvrz6qf75u3xpv805y7sugakf9 </pre></code> | 
| pion-1         | <pre><code> neutron164uehrt5zp6y9atz3x595zwad2vtk7gr5tpvmnj8nqqlc9a0g4xs4sqg0m </pre></code> |

## IBC Relayer Path Configuration
| Relayer  | Path           | Source Chain ID  | Destination Chain ID | 
| -------- | -------------- | ---------------- | -------------------- |
| Mainnet  | icon-archway   | mainnet          | archway-1            |
|          | icon-neutron   | mainnet          | neutron-1            |
| Berlin   | icon-archway   | ibc-icon         | constantine-3        |
| Lisbon   | icon-archway   | lisbon           | constantine-3        |
|          | icon-neutron   | lisbon           | pion-1               |

### Project Structure
| Directory | Description |
|:----------|:------------|
| [/contracts/cosmwasm-vm](./contracts/cosmwasm-vm) | Includes contracts for cosmwasm based chains |
| [/contracts/evm](./contracts/evm) | Includes contracts for evm based chains |
| [/contracts/javascore](./contracts/javascore) | Includes contracts for ICON chain |
| [/docs](./docs) | Documentation |
| [/scripts](./scripts) | Scripts to automate task in project, for example build scripts, deploy scripts. |


## Available Connection implementations
* [IBC](https://github.com/icon-project/IBC-Integration/blob/main/docs/adr/xcall-multi_IBC_Connection.md)
   * [Rust](https://github.com/icon-project/IBC-Integration/tree/main/contracts/cosmwasm-vm/cw-xcall-ibc-connection)
   * [Java](https://github.com/icon-project/IBC-Integration/tree/main/contracts/javascore/xcall-connection)
* [BTP](https://github.com/icon-project/btp2) is supported natively and does not need a connection contract.

## Building a xcall-multi connection
If xcall-multi is deployed, anyone can create a new connection contract to relay messages between xcall-multi contracts.
To do this a connection contract has to be developed and deployed on both sides.

The base design for a connection can be found in the [xcall-multi docs](./docs/adr/xcall.md#Connections)