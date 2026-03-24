# Escrow Contract Review Notes

## Scope

This contract implements a compact, storage-backed escrow flow for TalentTrust with one security-sensitive addition: safe client identity migration for live contracts.

The implementation is intentionally narrow:

- The freelancer identity is immutable after contract creation.
- The client identity is mutable only through a pending migration handshake.
- Milestones are defined up front and summed into a fixed escrow total.
- The contract must be fully funded before milestone releases begin.

## Storage Model

- `NextContractId`: monotonically increasing contract identifier counter.
- `Contract(id)`: persisted escrow record containing parties, milestone state, funding totals, and status.
- `PendingClientMigration(id)`: persisted migration record containing:
  - current client snapshot
  - proposed client
  - proposed-client confirmation flag

## Client Identity Migration

### Flow

1. `request_client_migration(contract_id, proposed_client)`
   - Requires current client authorization.
   - Rejects self-migration and migration to the freelancer address.
   - Rejects duplicate in-flight migration requests.

2. `confirm_client_migration(contract_id)`
   - Requires authorization by the proposed client.
   - Records explicit acceptance without yet transferring authority.

3. `finalize_client_migration(contract_id)`
   - Requires current client authorization.
   - Succeeds only after proposed-client confirmation.
   - Replaces the stored client authority and deletes the pending request.

4. `cancel_client_migration(contract_id)`
   - Requires current client authorization.
   - Deletes the pending request without transferring authority.

### Why this is safer than a single-step reassignment

- A typo in the proposed client address does not immediately transfer control.
- The new address must explicitly prove it can participate before handover.
- The old client must explicitly finalize after seeing the new address accept.
- Pending requests cannot be silently replaced, which prevents stale approvals from being repurposed.

## Escrow Lifecycle

- `create_contract` stores parties and milestone schedule after validating distinct roles and positive milestone amounts.
- `deposit_funds` only allows positive deposits and prevents overfunding above the milestone total.
- `release_milestone` requires full funding, rejects invalid milestone indexes, and blocks duplicate releases.
- Contract status transitions:
  - `Created` after creation and during partial funding
  - `Funded` once total escrow balance matches milestone sum
  - `Completed` once all milestones are released

## Test Layout

- `hello.rs`: keeps CI smoke coverage for the generated client.
- `lifecycle.rs`: covers storage persistence, ID allocation, funding, and completion behavior.
- `client_migration.rs`: covers the full request/confirm/finalize/cancel migration handshake.
- `security.rs`: covers invalid inputs, overfunding, invalid milestone releases, duplicate migration requests, and migration rejection on completed contracts.
