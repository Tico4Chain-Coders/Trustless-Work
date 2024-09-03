<h1 style="text-align: center;">Trustless Work</h1>

## Contents

- [Installing Rust](#installing-rust)
- [Install the Stellar CLI](#install-stellar-cli)
- [Configuring the CLI for Testnet](#configuring-the-cli-for-testnet)
- [Configure an idenity](#configure-an-identity)
- [Run project](#run-project)

## Installing Rust

### Linux, macOS, or Unix-like Systems

If you're using macOS, Linux, or any other Unix-like system, the simplest method to install Rust is by using `rustup`. Install it with the following command:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Windows

On Windows, download and run `rustup-init.exe`. You can proceed with the default setup by pressing `Enter`.

You can also follow the official Rust guide [here](https://www.rust-lang.org/tools/install).

### Install the wasm32 target.

After installing Rust, add the `wasm32-unknown-unknown` target:

```bash
rustup target add wasm32-unknown-unknown
```



## Install Stellar CLI

There are a few ways to install the [latest released version](https://github.com/stellar/stellar-cli/releases) of Stellar CLI.

The toolset installed with Rust allows you to use the `cargo` command in the terminal to install the Stellar CLI.

### Install with cargo from source:

```sh
cargo install --locked stellar-cli --features opt
```

### Install with cargo-binstall:

```sh
cargo install --locked cargo-binstall
cargo binstall -y stellar-cli
```

### Install with Homebrew (macOS, Linux):

```sh
brew install stellar-cli
```



## Configuring the CLI for Testnet

Stellar has a test network called Testnet that you can use to deploy  and test your smart contracts. It's a live network, but it's not the  same as the Stellar public network. It's a separate network that is used for development and testing, so you can't use it for production apps.  But it's a great place to test your contracts before you deploy them to  the public network.

To configure your CLI to interact with Testnet, run the following command:

### macOS/Linux

```sh
stellar network add \
  --global testnet \
  --rpc-url https://soroban-testnet.stellar.org:443 \
  --network-passphrase "Test SDF Network ; September 2015"
```

### Windows (PowerShell)

```sh
stellar network add `
  --global testnet `
  --rpc-url https://soroban-testnet.stellar.org:443 `
  --network-passphrase "Test SDF Network ; September 2015"
```

Note the `--global` flag. This creates a file in your home folder's `~/.config/soroban/network/testnet.toml` with the settings you specified. This means that you can use the `--network testnet` flag in any Stellar CLI command to use this network from any directory or filepath on your system.

If you want project-specific network configurations, you can omit the `--global` flag, and the networks will be added to your working directory's `.soroban/network` folder instead.

###  Configure an Identity

When you deploy a smart contract to a network, you need to specify an identity that will be used to sign the transactions.

Let's configure an identity called `alice`. You can use any name you want, but it might be nice to have some named identities that you can use for testing, such as [`alice`, `bob`, and `carol`](https://en.wikipedia.org/wiki/Alice_and_Bob). 

```sh
stellar keys generate --global alice --network testnet
```

You can see the public key of `alice` with:

```sh
stellar keys address alice
```

You can use this [link](https://stellar.expert/explorer/testnet) to verify the identity you create for the testnet.

## Run Project
