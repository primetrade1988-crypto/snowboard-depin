
**Projected Emission Curve:**

| MAU (30-day active) | Reward per Achievement | Daily Emission | Annual Emission | Pool Lasts |
|---|---|---|---|---|
| 1,000 | 0.5 MIII (cap) | 15,000 MIII | 5.5M MIII | 64 years |
| 10,000 | 0.15 MIII | 45,000 MIII | 16.4M MIII | 21 years |
| 100,000 | 0.05 MIII | 150,000 MIII | 54.8M MIII | 6.4 years |
| 1,000,000 | 0.01 MIII (floor) | 300,000 MIII | 109.5M MIII | 3.2 years |

**Key insight:** The pool never "runs out" prematurely because emission scales inversely with adoption. Early users earn more, late users earn less, but the pool lives 8+ years.

---

## 🔥 5. Burn Mechanics (Deflationary Pressure)

A mining pool of 35% creates inflationary pressure. Three burn sources compensate:

| Source | Burn % | Trigger |
|---|---|---|
| **B2B verification fee (in MIII)** | 50% burned, 50% to mining pool | Resort/brand pays for verified data |
| **Fraud penalty** | 100% of stake burned | AI + multisig detects fake data |
| **Unclaimed sponsor escrow** | 100% burned after 90 days | Challenge not claimed by any rider |
| **Transfer Hook (1%)** | 50% burned, 50% to treasury | Every secondary transfer of MIII |

**Net effect:** burn grows with ecosystem activity. When burn > emission → deflationary.

**Example at scale:**
- 5 resorts × $10,000/mo B2B fee = $50,000/mo
- 50% in MIII at $0.01 = 2,500,000 MIII burned monthly
- Mining emission at 10,000 MAU = 1,350,000 MIII monthly
- **Net: -1,150,000 MIII/month → deflationary**

---

## 🔒 6. Staking & Protocol Access

Resorts, brands, and startups must stake MIII to write data to the protocol.

| Parameter | Value |
|---|---|
| Minimum stake (resort) | 100,000 MIII |
| Minimum stake (startup / brand) | 50,000 MIII |
| Stake lock | 90 days minimum |
| Reward | Share of 1% Transfer Hook + priority data access |
| Slashing | 100% stake burn on verified fraud |
| Exit | 30-day cooldown, then unstake |

**Why stake?**
Without stake, anyone writes fake data. With stake, fraud costs real money.
Economic security > cryptographic security alone.

**What stakers get:**
1. Right to write data to the protocol
2. Share of 1% Transfer Hook revenue (pro-rata)
3. Priority access to aggregated datasets
4. Governance vote on protocol parameters

---

## 💰 7. Revenue Model (Company, not Token)

MIII Protocol is a **hardware company** first. Token is a growth layer.

| Revenue Stream | Unit Economics | Year 1 (3,000 devices) | Year 2 (10,000 devices) |
|---|---|---|---|
| Hardware | $199 sale / $100 cost → $99 margin | $597k rev / $297k margin | $1.99M rev / $990k margin |
| Subscription | $30/mo, 50% attach | $540k | $2.88M |
| B2B verification | 1–3% of partner revenue | $0 (in talks) | $240k |
| Aggregated data | $10k–100k per dataset | $0 (phase 2) | $150k |
| **Total** | | **~$1.1M** | **~$5.1M** |

**Token is not the primary revenue source.** It's the coordination layer.

---

## ⚖️ 8. Legal & Compliance

### Data Ownership Model

| Data Type | Controller | Processor | What We Can Do |
|---|---|---|---|
| Our hardware users | MIII Protocol | — | Sell aggregated datasets |
| Resort / partner users | Resort / partner | MIII Protocol | Process under DPA; sell aggregates only with consent |

**We never sell personal data.** Only aggregated, anonymized datasets.

### Regulatory Posture

- **GDPR (EU):** data stored in EU, right to deletion, no third-party transfer without consent
- **Belarus Law No. 99-Z:** consent-based processing, data subject rights honored
- **No medical data:** no pulse, no blood pressure, no rehab data in v1
- **No GPS storage:** ZK-proof of location only (phase 2)
- **No minors:** age gate at 18+, parental consent flow planned for phase 2
- **Legal opinion:** utility token classification (MiCA / SEC) in progress

### Required Documents (in `/legal`)

- [ ] `terms-of-service.md`
- [ ] `privacy-policy.md`
- [ ] `dpa-template.md`
- [ ] `dpo-appointment.md`

### Token Classification

MIII is a **utility token**, not a security:
- No expectation of profit from efforts of others
- Used for protocol access, staking, verification
- Not marketed as investment
- Legal opinion to be obtained before TGE

---

## 💸 9. Use of Funds — $250k Seed

| Allocation | Amount | Purpose |
|---|---|---|
| Hardware production (first 500 units) | $50,000 | Supply chain validation |
| Smart contract audit | $40,000 | Required for DEX listing |
| Legal (DPA, ToS, GDPR, legal opinion) | $30,000 | Data business compliance |
| Team salaries (6 months) | $80,000 | Runway to next milestone |
| Marketing + community | $50,000 | Cold start |
| **Total** | **$250,000** | |

### $25k Tranche Unlock Conditions

- 300+ devices shipped
- 3+ resorts/startups integrated into protocol
- Smart contract audit passed
- First on-chain burn record
- 5,000+ active users

---

## 🗺️ 10. Roadmap

### Phase 1 — Foundation (Current)
- [x] Hardware prototype (IMU sensor, Ed25519 signing)
- [x] Solana Anchor program (snowboard-depin)
- [x] Pyth Weather Oracle integration
- [x] Token-2022 Transfer Hook (1% royalty)
- [x] Sponsor Escrow with yield
- [x] Integration tests passing

### Phase 2 — Launch (Q1–Q2 2027)
- [ ] Mainnet deployment (Solana)
- [ ] Token generation event (TGE) + DEX listing
- [ ] First 500 devices shipped
- [ ] 3 resort/startup integrations signed
- [ ] Smart contract audit (CertiK / OtterSec)
- [ ] Legal docs finalized (ToS, Privacy, DPA, DPO)
- [ ] Legal opinion on token classification

### Phase 3 — Scale (Q3–Q4 2027)
- [ ] 10,000 active users
- [ ] ZK location proof (Light Protocol)
- [ ] AI Agent validator (multisig, not sole authority)
- [ ] Aggregated data marketplace v1
- [ ] Resort dashboard (B2B SaaS)

### Phase 4 — Expansion (2028+)
- [ ] Multi-sport support (surf, skate, wake)
- [ ] Rehab vertical (with clinical partner, medical certification)
- [ ] DAO transition (governance for stakers)
- [ ] Cross-chain bridge (if justified by volume)

---

## 🥊 11. Competitive Positioning

| Feature | Carv | STEPN | **MIII** |
|---|---|---|---|
| Sport | Ski only | Running | Snowboard, surf, skate |
| Hardware root | Ski boot sensor | None | IMU + Ed25519 |
| Token model | Subscription | Move-to-earn | Proof-of-motion + gamification |
| Data ownership | Centralized | N/A | User-controlled, DPA-compliant |
| Weather oracle | No | No | Pyth Powder Multiplier |
| Yield escrow | No | No | Sponsor escrow with DeFi yield |
| Burn mechanics | No | No | 4 burn sources |
| Staking for access | No | No | Yes (resorts, startups) |

**MIII is not "Carv for snowboard."** MIII is the verification layer for all action sports Carv cannot cover.

---

## ⚠️ 12. Key Risks (Honest Disclosure)

| Risk | Mitigation |
|---|---|
| STEPN-style collapse | Mining is gamification, not salary. Hardware revenue is primary. |
| Low liquidity at TGE | Market maker secured; 50M liquidity pool; phased unlock |
| Regulatory (token = security) | Utility token design; legal opinion pre-TGE |
| Data privacy violation | DPA + ToS + anonymization + DPO |
| Hardware spoofing | Ed25519 hardware signing; impossible to fake without physical device |
| Pyth oracle failure | Multi-oracle planned; weather only affects bonus, not base reward |
| AI validator errors | AI is assistant, not judge; multisig override |
| Mining pool depletion | Dynamic emission scales inversely with MAU |

---

## 📞 13. Contact & Links

- GitHub: [primetrade1988-crypto/snowboard-depin](https://github.com/primetrade1988-crypto/snowboard-depin)
- Website: TBD
- Twitter/X: TBD
- Contact: TBD

---

*MIII Protocol — Hardware-Rooted Action Sports Network.*
*Built on Solana. Verified on-chain. Owned by riders.*
