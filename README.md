# 🌿 GreenBadge — On-Chain Sustainability Credentials on Stellar

> **Issue, verify, and manage tamper-proof sustainability badges for businesses — powered by a Soroban smart contract on the Stellar blockchain.**

---

## 📖 Project Description

GreenBadge is a decentralized credentialing protocol built on [Stellar](https://stellar.org) using [Soroban](https://soroban.stellar.org) smart contracts. It enables trusted sustainability auditors and certification bodies to issue on-chain badges to businesses that meet verified environmental standards.

Traditional sustainability certifications live in centralized databases — opaque, revocable without trace, and hard for third parties to verify. GreenBadge moves this process on-chain, making every credential **publicly auditable**, **cryptographically verifiable**, and **permanently logged** on the Stellar ledger.

Whether you're a retailer sourcing from eco-friendly suppliers, an investor screening ESG-compliant companies, or a consumer wanting to trust a brand's green claim — GreenBadge gives you a single, trustless source of truth.

---

## 🔍 What It Does

GreenBadge allows a contract **administrator** to appoint trusted **verifiers** (e.g. auditing firms, NGOs, certification agencies). Those verifiers can then:

1. **Issue** a tiered badge to any Stellar address representing a verified sustainable business.
2. **Upgrade** the badge if the business improves its practices.
3. **Revoke** the badge if the business no longer qualifies — keeping the revocation on-chain for transparency.

Anyone can **query** the contract to check whether a given Stellar address currently holds a valid, active badge — enabling composability with other dApps, marketplaces, or supply-chain tools.

```
Admin ──► appoints Verifiers
Verifiers ──► issue / upgrade / revoke Badges
Anyone ──► query is_verified(business_address) → bool
```

---

## ✨ Features

### 🏅 Tiered Badge System
Four badge tiers reflect different levels of sustainability commitment:

| Tier     | Meaning                                      |
|----------|----------------------------------------------|
| 🥉 Bronze   | Entry-level sustainable practices             |
| 🥈 Silver   | Intermediate commitment & third-party audit   |
| 🥇 Gold     | High-impact leader, measurable outcomes       |
| 💎 Platinum | Exceptional — verified net-zero or beyond     |

### 🔐 Role-Based Access Control
- **Admin** — deployed once; manages verifier allowlist; can transfer admin rights.
- **Verifiers** — approved addresses that can issue, upgrade, and revoke badges.
- **Public** — anyone can read badge data; no permission needed.

### ⏳ Expiry & Renewal
Badges can be issued with a `validity_days` parameter. Expired badges automatically fail the `is_verified` check, prompting businesses to renew their audit cycle. Pass `0` for a perpetual badge.

### 🗂️ Category Tagging
Each badge carries a freeform `category` field (e.g. `"Energy"`, `"Agriculture"`, `"Manufacturing"`, `"Retail"`) for off-chain filtering and discovery.

### 🔍 On-Chain Audit Trail
Every `issue`, `upgrade`, and `revoke` action emits a Soroban **event** that is permanently stored on the Stellar ledger — providing a full, immutable history of a business's credential lifecycle.

### 🚫 Duplicate Protection
A business can only hold one active badge at a time. Attempting to issue a second badge to the same address panics, preventing accidental double-credentialing.

### 🔄 Admin Transferability
The contract admin role can be transferred to a new address (e.g. a multisig or DAO) without redeployment.

### 📊 Global Counter
`total_issued()` returns the cumulative number of badges ever issued — useful for dashboards and impact reporting.

---

## 🗂️ Project Structure

```
green-badge/
├── Cargo.toml                          # Workspace manifest
└── contracts/
    └── green_badge/
        ├── Cargo.toml                  # Contract dependencies
        └── src/
            ├── lib.rs                  # Contract logic
            └── test.rs                 # Unit tests
```

---

## 🚀 Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (stable)
- Soroban CLI: `cargo install --locked soroban-cli`
- A funded Stellar testnet account ([Friendbot](https://friendbot.stellar.org))

### Build

```bash
cargo build --release --target wasm32-unknown-unknown
```

The compiled `.wasm` will appear at:
```
target/wasm32-unknown-unknown/release/green_badge.wasm
```

### Test

```bash
cargo test
```

All 6 unit tests cover: issuing, verifying, revoking, upgrading, expiry, and access-control enforcement.

### Deploy to Testnet

```bash
# Deploy the contract
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/green_badge.wasm \
  --source <YOUR_SECRET_KEY> \
  --network testnet

# Initialize with your admin address
soroban contract invoke \
  --id <CONTRACT_ID> \
  --source <YOUR_SECRET_KEY> \
  --network testnet \
  -- initialize \
  --admin <ADMIN_ADDRESS>
```

### Example Invocations

```bash
# Add a verifier
soroban contract invoke --id <CONTRACT_ID> --source <ADMIN_KEY> --network testnet \
  -- add_verifier --admin <ADMIN_ADDR> --verifier <VERIFIER_ADDR>

# Issue a Gold badge
soroban contract invoke --id <CONTRACT_ID> --source <VERIFIER_KEY> --network testnet \
  -- issue_badge \
  --verifier <VERIFIER_ADDR> \
  --business <BUSINESS_ADDR> \
  --business_name '"EcoGrow Ltd"' \
  --tier Gold \
  --category '"Agriculture"' \
  --validity_days 365

# Check if a business is verified
soroban contract invoke --id <CONTRACT_ID> --network testnet \
  -- is_verified --business <BUSINESS_ADDR>

# Revoke a badge
soroban contract invoke --id <CONTRACT_ID> --source <VERIFIER_KEY> --network testnet \
  -- revoke_badge --caller <VERIFIER_ADDR> --business <BUSINESS_ADDR>
```

---

## 📡 Contract Interface Summary

| Function           | Caller         | Description                                  |
|--------------------|----------------|----------------------------------------------|
| `initialize`       | Deployer       | Set the admin (once)                         |
| `add_verifier`     | Admin          | Approve a verifier address                   |
| `remove_verifier`  | Admin          | Revoke a verifier's access                   |
| `transfer_admin`   | Admin          | Hand off admin rights                        |
| `issue_badge`      | Verifier/Admin | Issue a new badge to a business              |
| `upgrade_badge`    | Verifier/Admin | Upgrade tier or extend validity              |
| `revoke_badge`     | Verifier/Admin | Mark a badge as revoked (kept for audit)     |
| `get_badge`        | Anyone         | Fetch the full badge struct                  |
| `is_verified`      | Anyone         | Returns `true` if badge is valid & unexpired |
| `total_issued`     | Anyone         | Total badges ever issued                     |
| `get_admin`        | Anyone         | Returns current admin address                |

---

## 🧪 Test Coverage

| Test | What It Covers |
|------|---------------|
| `test_issue_and_verify` | Happy-path issue + query |
| `test_revoke_badge` | Revocation clears verified status |
| `test_upgrade_badge` | Tier upgrade is reflected on-chain |
| `test_badge_expiry` | Expired badge fails `is_verified` |
| `test_duplicate_badge_panics` | Double-issue protection |
| `test_non_verifier_cannot_issue` | Access control enforcement |

---

## 🛣️ Roadmap Ideas

- [ ] Multi-badge support (one business, multiple category badges)
- [ ] NFT-style badge metadata with IPFS proof documents
- [ ] DAO-controlled verifier governance
- [ ] Cross-contract composability hooks for DeFi / marketplaces
- [ ] Frontend dApp with Freighter wallet integration

---

## 📄 License

MIT — see [LICENSE](LICENSE) for details.

---

> Built with 💚 on [Stellar](https://stellar.org) · Soroban SDK `v21`
wallet address : GCDXXPDMNNDIFK42XH2OMDCN45AZXNXEEPPLJEP4I3Y23N7QJGIEO5Y2
>
contract address : CDRI4WBNKZKBQBX47CLEEI2RMZYTCNC6JGYTIDWYWI7QGTPGI7E44XAJ

https://stellar.expert/explorer/testnet/contract/CDRI4WBNKZKBQBX47CLEEI2RMZYTCNC6JGYTIDWYWI7QGTPGI7E44XAJ

<img width="1366" height="768" alt="Screenshot (27)" src="https://github.com/user-attachments/assets/02daef8a-0736-4325-b4dd-b15f6abe34aa" />



















































