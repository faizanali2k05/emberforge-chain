# Emberforge Chain

![Emberforge Chain logo](assets/emberforge-logo.png)

An educational blockchain implemented in Rust. Emberforge demonstrates the core mechanics behind a small proof-of-work chain without hiding the important parts behind a framework.

> This is a learning project, not production financial software. It has not been audited and should not be used with real funds.

## What it includes

- SHA-256 block hashing and proof-of-work mining
- Ed25519 wallet keys and signed transactions
- Account balances, nonces, pending transactions, and mining rewards
- Genesis balances and deterministic chain validation
- Tamper and forgery detection
- JSON persistence and reload
- Unit tests for the security-critical paths

## Quick start

```bash
cargo test
cargo run
```

The demo creates two wallets, funds Alice from genesis, signs a payment to Bob, mines a block, and prints the resulting balances and tip hash.

## Project layout

```text
src/lib.rs       Core blockchain, wallets, transactions, validation, persistence
src/main.rs      Runnable local demonstration
assets/          Project branding
.github/         Continuous integration workflow
```

## Design

Transactions contain a sender address, recipient address, amount, nonce, and Ed25519 signature. The sender address is the hex-encoded public key. A transaction is accepted only when its signature verifies and the sender has enough balance.

Blocks are mined by searching for a nonce whose SHA-256 hash begins with the configured number of zeroes. Each block commits to the previous block hash, so modifying an earlier block invalidates the chain from that point onward.

The current implementation is intentionally single-node and account-based. Networking, peer discovery, fork choice, durable databases, and production consensus are future milestones.

## Example API

```rust
let alice = Wallet::generate();
let bob = Wallet::generate();

let mut genesis = HashMap::new();
genesis.insert(alice.address(), 100);

let mut chain = Blockchain::new(2, genesis);
let payment = Transaction::new(alice.address(), bob.address(), 25, 0).sign(&alice);

chain.submit_transaction(payment)?;
chain.mine_pending(&alice.address())?;
assert!(chain.is_valid());
```

## Roadmap

1. Add a command-line wallet and chain state commands.
2. Enforce transaction nonce uniqueness and replay protection.
3. Add a mempool with fee-based prioritization.
4. Add peer-to-peer networking and block synchronization.
5. Replace JSON persistence with a database-backed state store.
6. Add a real consensus layer and security review before any public deployment.

## License

Licensed under the MIT License. See [LICENSE](LICENSE).
