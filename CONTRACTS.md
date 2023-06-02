# Contracts



## Creating a new contract

Use [`cosmwasm-template`](https://github.com/CosmWasm/cw-template) as a basis.

```bash
cd contracts/core-contracts #cd contracts/token-contracts
cargo generate --git https://github.com/CosmWasm/cw-template.git --name PROJECT_NAME
cd PROJECT_NAME
rm -rf .git
rm -rf .circleci
rm .gitignore
rm -rf .github
rm .editorconfig
rm .cargo-ok
rm LICENSE
rm NOTICE
```
