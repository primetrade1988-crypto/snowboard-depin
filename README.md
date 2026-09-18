# Snowboard DePIN — Motion Proof & Rewards

This repo implements motion-proof validation and rewards for a snowboard DePIN program (Anchor/Solana).

Key features
- `submit_motion_proof` instruction: verifies an Ed25519-signed motion proof (trick) and mints rewards from the treasury.
- Cross-checking of motion proofs against last telemetry to mitigate spoofing.
- `Badge` PDA tracking per-trick achievements.

Quick dev commands

Build Rust program and run unit tests:

```bash
cd programs/snowboard-depin
cargo build
cargo test
```

Generate Anchor IDL (requires `anchor`):

```powershell
cd programs/snowboard-depin
anchor build
```
Or run the provided script:

```powershell
.\scripts\generate_idl.ps1
```

Example: submit a motion proof (TypeScript)
- See `scripts/submit_motion_proof_example.ts` for a runnable example that:
  - builds a motion payload,
  - signs it using an Ed25519 keypair (device),
  - creates the required Ed25519 verify instruction and calls `submit_motion_proof`.

Real example: building and signing an Ed25519 motion proof (TypeScript)

```ts
import { Keypair, Transaction } from '@solana/web3.js';
import nacl from 'tweetnacl';

const prefix = Buffer.from('SNOWBOARD_DEPIN_MOTION');
const deviceKeypair = Keypair.fromSecretKey(Uint8Array.from(/* 64-byte secret */));
const proof = {
  nonce: 1n,
  timestamp: BigInt(Math.floor(Date.now() / 1000)),
  trick_id: 42,
  airtime_ms: 800,
  rotation_deg: 720,
  confidence: 85,
};

// Build message exactly like on-chain `build_motion_message`
const parts: Buffer[] = [prefix, Buffer.from(deviceKeypair.publicKey.toBytes())];
parts.push(Buffer.from(BigInt(proof.nonce).toString(16).padStart(16, '0'), 'hex'));
parts.push(Buffer.from(BigInt(proof.timestamp).toString(16).padStart(16, '0'), 'hex'));
parts.push(Buffer.from(new Uint8Array([proof.trick_id & 0xff, (proof.trick_id >> 8) & 0xff])));
parts.push(Buffer.from(new Uint8Array([proof.airtime_ms & 0xff, (proof.airtime_ms>>8)&0xff, (proof.airtime_ms>>16)&0xff, (proof.airtime_ms>>24)&0xff])));
parts.push(Buffer.from(new Uint8Array([proof.rotation_deg & 0xff, (proof.rotation_deg>>8)&0xff])));
parts.push(Buffer.from([proof.confidence]));
const message = Buffer.concat(parts);

// Sign with ed25519 (use first 32 bytes of secret key)
const sig = nacl.sign.detached(new Uint8Array(message), deviceKeypair.secretKey.slice(0,32));

// Create Ed25519 verify instruction and include before Anchor instruction in tx
// anchor.web3.Ed25519Program.createInstructionWithPublicKey({ publicKey, message, signature })

// Then call `submit_motion_proof` on-chain, passing the same `proof` payload and ed25519 ix index 0.
```

Notes
- Adjust constants in `programs/snowboard-depin/src/constants.rs` for tuning rewards and anti-fraud thresholds.
- The example contains placeholders for keypairs and RPC endpoints — replace them with real device keypairs and a funded payer for testing.
