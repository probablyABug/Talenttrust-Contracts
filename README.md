# TalentTrust Contracts

Soroban smart contracts for the TalentTrust decentralized freelancer escrow protocol on the Stellar network.

## What's in this repo

- **Escrow contract** (`contracts/escrow`): Holds funds in escrow, supports milestone-based payments, reputation credential issuance, and a dispute initiation workflow.

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

# Check formatting
cargo fmt --all -- --check

# Format code
cargo fmt --all
```

## Dispute Initiation Workflow

The escrow contract supports a formal on-chain dispute mechanism. Either the client or the freelancer may raise a dispute against a funded or completed escrow.

### State machine

```
Created ──(deposit)──► Funded ──(dispute)──► Disputed
                          │
                    (complete)
                          │
                          ▼
                      Completed ──(dispute)──► Disputed
```

### Functions

#### `initiate_dispute(env, contract_id, initiator, reason) -> Result<(), DisputeError>`

Raises a dispute on an existing escrow.

| Parameter     | Type     | Description                                      |
|---------------|----------|--------------------------------------------------|
| `contract_id` | `u32`    | Numeric ID of the escrow to dispute              |
| `initiator`   | `Address`| Client or freelancer raising the dispute         |
| `reason`      | `String` | Short human-readable description of the conflict |

**Authorization:** `initiator.require_auth()` is called before any state mutation.

**Errors:**

| Error                        | Meaning                                              |
|------------------------------|------------------------------------------------------|
| `DisputeError::NotFound`     | No escrow with `contract_id` exists                  |
| `DisputeError::Unauthorized` | `initiator` is not the client or freelancer          |
| `DisputeError::InvalidStatus`| Escrow is in `Created` status (not yet funded)       |
| `DisputeError::AlreadyDisputed` | A dispute record already exists for this escrow  |

#### `get_dispute(env, contract_id) -> Option<DisputeRecord>`

Returns the immutable dispute record for an escrow, or `None` if no dispute has been initiated.

### DisputeRecord fields

| Field       | Type      | Description                                         |
|-------------|-----------|-----------------------------------------------------|
| `initiator` | `Address` | Address that raised the dispute                     |
| `reason`    | `String`  | Human-readable reason provided at initiation        |
| `timestamp` | `u64`     | Ledger timestamp (seconds since Unix epoch) at creation |

### Security notes

- `require_auth()` is the first call in `initiate_dispute` — no state is read or written before authorization is verified.
- The `DisputeRecord` is written exactly once; a second call on the same escrow returns `AlreadyDisputed` before any write occurs.
- Only the client or freelancer stored at escrow creation time can initiate a dispute — the check uses the on-chain addresses, not caller-supplied values.

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
