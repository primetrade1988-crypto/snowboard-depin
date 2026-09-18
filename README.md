# 🏂 MIII Protocol — DePIN & Proof-of-Motion for Action Sports

[![Solana](https://img.shields.io/badge/Solana-Mainnet%20Ready-00FFA3?style=for-the-badge&logo=solana)](https://solana.com)
[![Anchor Framework](https://img.shields.io/badge/Anchor-0.30+-2C2D35?style=for-the-badge&logo=rust)](https://www.anchor-lang.com/)
[![Token-2022](https://img.shields.io/badge/Token--2022-Transfer%20Hooks-blueviolet?style=for-the-badge)](https://spl.solana.com/token-2022)
[![Pyth Oracles](https://img.shields.io/badge/Pyth-Weather%20Oracles-E6DAFE?style=for-the-badge)](https://pyth.network)
[![Hackathon](https://img.shields.io/badge/Colosseum-Hackathon%20Submission-FF2A6D?style=for-the-badge)](https://colosseum.org)

> **Hardware-Rooted Action Sports Network.** MIII Protocol validates physical motion (snowboarding, surfing, skateboarding) via wearable sensors, verifying on-chain activity on Solana with zero-knowledge privacy, Pyth oracle multipliers, and AI agent validation.

---

## 🏛 Architecture Overview

```
 ┌────────────────┐     Ed25519      ┌─────────────────────────┐
 │ Hardware IMU   │ ────────────────>│ Solana Anchor Program   │
 │ Sensor (Board) │   Hardware Proof │ (snowboard-depin)       │
 └────────────────┘                  └────────────┬────────────┘
                                                  │
       ┌──────────────────┬───────────────────────┼───────────────────────┬──────────────────┐
       │                  │                       │                       │                  │
 ┌─────▼──────┐    ┌──────▼──────┐         ┌──────▼──────┐         ┌──────▼──────┐    ┌──────▼──────┐
 │ Pyth       │    │ Token-2022  │         │ Yield-      │         │ ZK Proof    │    │ AI Agent    │
 │ Weather    │    │ Transfer    │         │ Bearing     │         │ Location    │    │ Multi-Sig   │
 │ Multiplier │    │ Hook (1%)   │         │ Sponsor     │         │ Privacy     │    │ Validator   │
 │ (Powder)   │    │ Protocol    │         │ Escrow      │         │ (Light ZK)  │    │ (ElizaOS)   │
 └────────────┘    └─────────────┘         └─────────────┘         └─────────────┘    └─────────────┘
```

---

## ⚡ Key Features (Bugatti-Tier Primitives)

1. **Hardware-Rooted Proof-of-Motion (Ed25519 Validation):**  
   Direct cryptographic verification of IMU sensor metrics (G-force, rotation speed, airtime) signed on-hardware to prevent GPS/telemetry spoofing.

2. **Pyth Weather Oracles ("Powder Multiplier"):**  
   On-chain weather feed validation. When extreme winter conditions or heavy snowfall are detected via Pyth, riders automatically receive a **2x reward multiplier** for active sessions.

3. **Token-2022 Transfer Hooks:**  
   Enforced 1% protocol royalty fee on token transfers built natively into the SPL Token-2022 extension, funding the community treasury directly on-chain.

4. **Yield-Bearing Sponsor Escrow:**  
   Sponsor prize pools are deposited into yield-generating escrow accounts, auto-compounding interest via Solana DeFi protocols while waiting to be claimed by top riders.

5. **ZK-Proof Location Verification:**  
   Zero-Knowledge SNARK placeholders (Light Protocol compatible) allowing riders to prove they were inside a specific resort/snowpark without exposing private GPS telemetry tracks.

6. **AI Agent Multi-Sig Validator:**  
   Autonomous AI Agent role for verifying complex or ambiguous trick executions (e.g., Quad Corks, Switch Backside spin profiles) using machine-learning pattern matching.

---

## 🛠 Repository Structure

```text
snowboard-depin/
├── programs/
│   └── snowboard-depin/
│       └── src/
│           └── lib.rs          # Core Anchor Program (Pyth, Token-2022, ZK, Escrow)
├── tests/
│   └── snowboard-depin.ts      # TypeScript Integration Tests
├── Anchor.toml                 # Anchor Configuration & Cluster Settings
└── Cargo.toml                  # Rust Dependencies & Solita Setup
```

---

## 🚀 Quickstart & Local Testing

### Prerequisites
* Rust v1.75+
* Solana CLI v1.18+
* Anchor CLI v0.30+
* Node.js v18+ & Yarn

### Installation

```bash
# Clone the repository
git clone [https://github.com/primetrade1988-crypto/snowboard-depin.git](https://github.com/primetrade1988-crypto/snowboard-depin.git)
cd snowboard-depin

# Install JS dependencies
yarn install

# Build Anchor program
anchor build

# Run integration tests
anchor test
```

---

## 📜 License

Distributed under the MIT License. See `LICENSE` for more information.

