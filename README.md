# Ink! Token Contract

A simple ERC-20 like token contract implemented in ink! for Substrate blockchains.

## Features

- **Token Minting**: Contract owner can mint new tokens
- **Token Transfers**: Users can transfer tokens to other accounts
- **Balance Checking**: Query token balances for any account
- **Total Supply**: Track the total supply of tokens
- **Events**: Emits events for token transfers and mints

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (latest stable version)
- [cargo-contract](https://github.com/paritytech/cargo-contract) - For building ink! contracts

## Building

To build the smart contract, run:

```bash
cargo contract build
```

This will create a `target/ink` directory containing the compiled WebAssembly (`.wasm`) file and metadata (`.json`) file.

## Testing

To run the tests:

```bash
cargo test
```

## Usage

### Deployment

1. Build the contract with `cargo contract build`
2. Deploy the contract to a Substrate node using the generated `.wasm` and `.json` files

### Contract Interface

#### Constructor

```rust
/// Creates a new token contract with the caller as the owner
#[ink(constructor)]
pub fn new() -> Self;
```

#### Messages

```rust
/// Mint new tokens (only callable by owner)
#[ink(message)]
pub fn mint(&mut self, to: AccountId, amount: Balance) -> Result<(), Error>;

/// Get the balance of an account
#[ink(message)]
pub fn balance_of(&self, account: AccountId) -> Balance;

/// Transfer tokens to another account
#[ink(message)]
pub fn transfer(&mut self, to: AccountId, amount: Balance) -> Result<(), Error>;

/// Get the total supply of tokens
#[ink(message)]
pub fn total_supply(&self) -> Balance;
```

#### Events

```rust
/// Emitted when tokens are transferred
#[ink(event)]
pub struct Transfer {
    from: Option<AccountId>,
    to: Option<AccountId>,
    value: Balance,
}

/// Emitted when new tokens are minted
#[ink(event)]
pub struct Mint {
    to: AccountId,
    value: Balance,
}
```

#### Errors

```rust
pub enum Error {
    /// Insufficient balance for the transfer
    InsufficientBalance,
    /// Caller is not the contract owner
    NotOwner,
}
```

## License

This project is licensed under the [MIT License](LICENSE).

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Acknowledgements

- [ink!](https://use.ink/) - Rust-based eDSL for writing smart contracts for Substrate
- [Substrate](https://substrate.io/) - Blockchain development framework
- [Polkadot](https://polkadot.network/) - Multi-chain network
