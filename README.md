[![Project Status: Initial Release](https://img.shields.io/badge/repo%20status-active-green.svg?style=flat-square)](https://www.repostatus.org/#active)
[![License: Apache-2.0](https://img.shields.io/github/license/icon-project/xCall.svg?style=flat-square)](https://github.com/icon-project/xCall/blob/main/LICENSE)
[![Lines Of Code](https://img.shields.io/tokei/lines/github/icon-project/xCall?style=flat-square)](https://github.com/icon-project/xCall)
[![Version](https://img.shields.io/github/tag/icon-project/xCall.svg?style=flat-square)](https://github.com/icon-project/xCall)
![GitHub Workflow Status - cosmwasm](https://github.com/icon-project/xCall/actions/workflows/build-and-publish-cosmwasm.yml/badge.svg)
![GitHub Workflow Status - javascore](https://github.com/icon-project/xCall/actions/workflows/build-and-publish-javascore.yml/badge.svg)
| Language                            | Code Coverage                                  |
| ----------------------------------- | ---------------------------------------------- |
| [Java](./contracts/javascore)       | [![Java Cov][java-cov-badge]][java-cov-link]   |
| [Rust](./contracts/cosmwasm-vm)     | [![Rust Cov][rust-cov-badge]][rust-cov-link]   |

[java-cov-link]: https://app.codecov.io/gh/icon-project/xCall/tree/main/contracts/javascore
[rust-cov-link]: https://app.codecov.io/gh/icon-project/xCall/tree/main/contracts/cosmwasm-vm
[java-cov-badge]: https://codecov.io/gh/icon-project/xCall/branch/main/graph/badge.svg?token=KWDB59JITE&flag=java
[rust-cov-badge]: https://codecov.io/gh/icon-project/xCall/branch/main/graph/badge.svg?token=KWDB59JITE&flag=rust

# XCall

### Project Structure
| Directory | Description |
|:----------|:------------|
| [/contracts/cosmwasm-vm](./contracts/cosmwasm-vm) | Includes contracts for cosmwasm based chains |
| [/contracts/evm](./contracts/evm) | Includes contracts for evm based chains |
| [/contracts/javascore](./contracts/javascore) | Includes contracts for ICON chain |
| [/docs](./docs) | Documentation |
| [/scripts](./scripts) | Scripts to automate task in project, for example build scripts, deploy scripts. |