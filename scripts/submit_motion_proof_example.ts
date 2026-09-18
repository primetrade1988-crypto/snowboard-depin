import * as anchor from '@coral-xyz/anchor';
import { PublicKey, Keypair, Transaction } from '@solana/web3.js';
import * as bs58 from 'bs58';
import * as nacl from 'tweetnacl';

// Example usage:
// - Replace RPC_URL, payerKeypair, and deviceKeypair with real values
// - Run: `npm install` then `npx ts-node scripts/submit_motion_proof_example.ts`

const RPC_URL = process.env.RPC_URL || 'https://api.devnet.solana.com';

async function main() {
  const provider = anchor.AnchorProvider.local(RPC_URL);
  anchor.setProvider(provider);

  // Load program from IDL (assumes Anchor idl available)
  const idl = require('../target/idl/snowboard_depin.json');
  const programId = new PublicKey(idl.metadata.address || process.env.PROGRAM_ID);
  const program = new anchor.Program(idl, programId, provider);

  // Payer (must be funded)
  const payer = Keypair.fromSecretKey(bs58.decode(process.env.PAYER_SK || ''));

  // Device Ed25519 keypair (private key must sign motion message)
  const device = Keypair.fromSecretKey(bs58.decode(process.env.DEVICE_SK || ''));

  // Motion proof payload
  const proof = {
    nonce: 1,
    timestamp: Math.floor(Date.now() / 1000),
    trick_id: 42,
    airtime_ms: 800,
    rotation_deg: 720,
    confidence: 85,
  };

  // Build motion message (must match on-chain layout)
  const prefix = Buffer.from('SNOWBOARD_DEPIN_MOTION');
  const devicePub = device.publicKey.toBytes();
  const buf = Buffer.concat([
    prefix,
    Buffer.from(devicePub),
    Buffer.from(new BN(proof.nonce).toArray('le', 8)),
    Buffer.from(new BN(proof.timestamp).toArray('le', 8)),
    Buffer.from(new BN(proof.trick_id).toArray('le', 2)),
    Buffer.from(new BN(proof.airtime_ms).toArray('le', 4)),
    Buffer.from(new BN(proof.rotation_deg).toArray('le', 2)),
    Buffer.from([proof.confidence]),
  ]);

  // Sign message with device Ed25519 private key
  const signature = nacl.sign.detached(new Uint8Array(buf), device.secretKey.slice(0, 32));

  // Create Ed25519 verify instruction
  const ed25519Ix = anchor.web3.Ed25519Program.createInstructionWithPublicKey({
    publicKey: device.publicKey.toBuffer(),
    message: buf,
    signature: Buffer.from(signature),
  });

  // Build Anchor instruction
  const submitIx = await program.methods
    .submitMotionProof(proof, 0) // ed25519_ix_index will be 0 (first instruction)
    .accounts({
      globalConfig: await program.account.globalConfig.associatedAddress(),
      fusionMint: /* fill */ new PublicKey(process.env.FUSION_MINT || ''),
      treasuryVault: /* fill */ new PublicKey(process.env.TREASURY_VAULT || ''),
      device: /* device account PDA */ new PublicKey(process.env.DEVICE_ACCOUNT || ''),
      motionRecord: /* PDA (will be created) */ new PublicKey(process.env.MOTION_RECORD || ''),
      badge: /* PDA or will be created */ new PublicKey(process.env.BADGE_ACCOUNT || ''),
      recipientFusion: /* recipient token account */ new PublicKey(process.env.RECIPIENT_FUSION || ''),
      payer: payer.publicKey,
      instructionsSysvar: anchor.web3.SYSVAR_INSTRUCTIONS_PUBKEY,
      tokenProgram: anchor.web3.TOKEN_PROGRAM_ID,
      systemProgram: anchor.web3.SystemProgram.programId,
    })
    .instruction();

  // Compose transaction with ed25519 verify first
  const tx = new Transaction().add(ed25519Ix as any, submitIx as any);
  tx.feePayer = payer.publicKey;
  tx.recentBlockhash = (await provider.connection.getLatestBlockhash()).blockhash;

  // Signers: payer + device (device signature already embedded in ed25519 instruction)
  await provider.sendAndConfirm(tx, [payer]);

  console.log('submitted motion proof');
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
