# Soroban Project

## Install Rust
Linux, macOS, or other Unix-like OS
If you use macOS, Linux, or another Unix-like OS, the simplest method to install a Rust toolchain is to install rustup. Install rustup with the following command.

`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`

### Windows
On Windows, download and run rustup-init.exe. You can continue with the default settings by pressing Enter.

### TIP
The Stellar CLI uses emojis in its output. To properly render them on Windows, it is recommended to use the Windows Terminal. See how to install Windows Terminal on Microsoft Learn. If the CLI is used in the built in Windows Command Prompt or Windows PowerShell the CLI will function as expected but the emojis will appear as question marks.

If you're already using WSL, you can also follow the instructions for Linux.

### Install the target
Install the wasm32-unknown-unknown target.

`rustup target add wasm32-unknown-unknown`

### Configure an Editor
Many editors have support for Rust. Visit the following link to find out how to configure your editor: https://www.rust-lang.org/tools

A popular editor is Visual Studio Code:

Visual Studio Code editor.
- Rust Analyzer for Rust language support.
- CodeLLDB for step-through-debugging.

## Install the Stellar CLI
The Stellar CLI can execute smart contracts on futurenet, testnet, mainnet, as well as in a local sandbox.

### Install
There are a few ways to install the latest released version of Stellar CLI.

### Install with cargo from source:

`cargo install --locked stellar-cli --features opt`

### Install with cargo-binstall:

`cargo install --locked cargo-binstall
cargo binstall -y stellar-cli`

### Install with Homebrew (macOS, Linux):

`brew install stellar-cli`

## Autocompletion
You can use stellar completion to generate shell completion for bash, elvish, fish, powershell, and zsh. You should absolutely try it out. It will feel like a super power!

To enable autocomplete in the current bash shell, run:

`source <(stellar completion --shell bash)`

To enable autocomplete permanently in future bash shells, run:

`echo "source <(stellar completion --shell bash)" >> ~/.bashrc`

Users of non-bash shells may need to adapt the above commands to suit their needs.

### Configuring the CLI for Testnet
Stellar has a test network called Testnet that you can use to deploy and test your smart contracts. It's a live network, but it's not the same as the Stellar public network. It's a separate network that is used for development and testing, so you can't use it for production apps. But it's a great place to test your contracts before you deploy them to the public network.

To configure your CLI to interact with Testnet, run the following command:

`macOS/Linux
Windows (PowerShell)
stellar network add 
  --global testnet 
  --rpc-url https://soroban-testnet.stellar.org:443 
  --network-passphrase "Test SDF Network ; September 2015"`

Note the --global flag. This creates a file in your home folder's ~/.config/soroban/network/testnet.toml with the settings you specified. This means that you can use the --network testnet flag in any Stellar CLI command to use this network from any directory or filepath on your system.

If you want project-specific network configurations, you can omit the --global flag, and the networks will be added to your working directory's .soroban/network folder instead.

### Configure an Identity
When you deploy a smart contract to a network, you need to specify an identity that will be used to sign the transactions.

Let's configure an identity called alice. You can use any name you want, but it might be nice to have some named identities that you can use for testing, such as alice, bob, and carol.

`stellar keys generate --global alice --network testnet`

You can see the public key of alice with:

`stellar keys address alice`

### Test
For execute any test, yo have you join in your terminal and go to the root of Trustless, then:
`cargo test`
