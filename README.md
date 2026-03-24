# TalentTrust Contracts

Soroban smart contracts for the TalentTrust decentralized freelancer escrow protocol on the Stellar network.

## What's in this repo

- **Escrow contract** (`contracts/escrow`): Holds funds in escrow, supports milestone-based payments, and includes a two-party client identity migration flow with explicit confirmations.

## Current Escrow Capabilities

- Stateful escrow creation with stored client, freelancer, milestone schedule, and aggregate funding state.
- Milestone funding and release controls gated by the current client address.
- Safe client identity migration:
  - Current client requests migration to a new address.
  - Proposed client explicitly confirms acceptance.
  - Current client explicitly finalizes the handover.
  - Pending migrations cannot be overwritten and may be cancelled before finalization.

## Reviewer Guide

- Contract entrypoints live in [contracts/escrow/src/lib.rs](/home/json/Desktop/Drips/Talenttrust-Contracts/contracts/escrow/src/lib.rs).
- The test suite is organized by behavior in [contracts/escrow/src/test.rs](/home/json/Desktop/Drips/Talenttrust-Contracts/contracts/escrow/src/test.rs) and `contracts/escrow/src/test/*`.
- Escrow-specific implementation notes and threat assumptions are documented in:
  - [docs/escrow/README.md](/home/json/Desktop/Drips/Talenttrust-Contracts/docs/escrow/README.md)
  - [docs/escrow/security.md](/home/json/Desktop/Drips/Talenttrust-Contracts/docs/escrow/security.md)

## Prerequisites

- [Rust](https://rustup.rs/) (stable, 1.75+)
- `rustfmt`: `rustup component add rustfmt`
- Optional: [Stellar CLI](https://developers.stellar.org/docs/tools/stellar-cli) for deployment

## Setup

```bash
# Clone (or you're already in the repo)
git clone <your-repo-url>
cd talenttrust-contracts

# Build
cargo build

# Run tests
cargo test

# Run escrow contract tests only
cargo test -p escrow

# Check formatting
cargo fmt --all -- --check

# Format code
cargo fmt --all
```

## Contributing

1. Fork the repo and create a branch from `main`.
2. Make changes; keep tests and formatting passing:
   - `cargo fmt --all`
   - `cargo test`
   - `cargo build`
3. Open a pull request. CI runs `cargo fmt --all -- --check`, `cargo build`, and `cargo test` on push/PR to `main`.

## CI/CD

On every push and pull request to `main`, GitHub Actions:

- Checks formatting (`cargo fmt --all -- --check`)
- Builds the workspace (`cargo build`)
- Runs tests (`cargo test`)

Ensure these pass locally before pushing.

## License

MIT
