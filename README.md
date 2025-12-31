# Blueshift Pinocchio Vault

A lightweight, high-performance Solana Vault program built using the [Pinocchio](https://github.com/febo/pinocchio) framework. This program demonstrates a simple vault implementation focusing on minimal compute unit usage and binary size optimization through Pinocchio's `no_std` approach.

## 🚀 Features

- **Zero-Copy Serialization**: Utilizes Pinocchio's direct byte manipulation for account parsing and instruction data.
- **Low Compute Usage**: optimized for efficiency, bypassing standard Borsh/Anchor serialization overhead.
- **PDA-based Vaults**: Securely manages funds using Program Derived Addresses (PDAs).

## 🛠 Project Structure

- **`lib.rs`**: Entrypoint definition and instruction routing.
- **`instructions/deposit.rs`**: Logic for depositing SOL into a derived vault.
- **`instructions/withdraw.rs`**: Logic for withdrawing all SOL from the vault.

## 📜 Instructions

The program supports two main instructions:

### 1. Deposit (Discriminator: `0`)

Deposits a specified amount of SOL from the user's account into their dedicated vault PDA.

**Accounts:**

1. `[signer]` **Owner**: The account sending the SOL.
2. `[writable]` **Vault**: The PDA where funds are stored. Derived from `["vault", owner_pubkey]`.
3. `[]` **System Program**: Required for the transfer CPI.

**Data:**

- `amount` (u64): The amount of lamports to deposit.

### 2. Withdraw (Discriminator: `1`)

Withdraws **all** available lamports from the vault PDA back to the owner's account.

**Accounts:**

1. `[signer]` **Owner**: The account receiving the SOL. Must match the seed used to derive the vault.
2. `[writable]` **Vault**: The PDA holding the funds.
3. `[]` **System Program**: Required for the transfer CPI.

## 🔧 Building

To build the program using result:

```bash
cargo build-sbf
```

Ensure you have the Solana Rust SDK and tools installed.

## 🔐 Account Validation

The program manually implements strict validation checks:

- **Signer Checks**: Ensures the owner signed the transaction.
- **Owner Checks**: Verifies accounts are owned by the expected programs (System Program / This Program).
- **PDA Verification**: Re-derives the Vault address to ensure it matches the provided account.

## 📚 About Pinocchio

Pinocchio is a library for writing Solana programs with zero dependencies on the Rust standard library (`no_std`). It offers a significant reduction in binary size and compute unit consumption compared to standard Anchor or native Solana usage.
