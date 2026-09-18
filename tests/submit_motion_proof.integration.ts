import * as anchor from '@coral-xyz/anchor';
import { Keypair, Transaction } from '@solana/web3.js';
import nacl from 'tweetnacl';
import { expect } from 'chai';

// Integration test for submit_motion_proof.
// By default this test is skipped unless RUN_INTEGRATION=1 and PROGRAM_ID provided.

describe('submit_motion_proof integration', function () {
  it('builds motion message and (optionally) submits tx', async function () {
    if (!process.env.RUN_INTEGRATION) {
      this.skip();
      return;
    }

    const provider = anchor.AnchorProvider.local();
    anchor.setProvider(provider);
    const idl = require('../programs/snowboard-depin/target/idl/snowboard_depin.json');
    const programId = new anchor.web3.PublicKey(process.env.PROGRAM_ID as string);
    const program = new anchor.Program(idl, programId, provider);

    const payer = provider.wallet.payer as Keypair;
    const device = Keypair.generate();

    const proof = {
      nonce: 1,
      timestamp: Math.floor(Date.now() / 1000),
      trick_id: 7,
      airtime_ms: 800,
      rotation_deg: 720,
      confidence: 85,
    } as any;

    const prefix = Buffer.from('SNOWBOARD_DEPIN_MOTION');
    const parts: Buffer[] = [prefix, Buffer.from(device.publicKey.toBytes())];
    const bufNonce = Buffer.alloc(8);
    bufNonce.writeBigUInt64LE(BigInt(proof.nonce));
    parts.push(bufNonce);
    const bufTs = Buffer.alloc(8);
    bufTs.writeBigUInt64LE(BigInt(proof.timestamp));
    parts.push(bufTs);
    const bufTrick = Buffer.alloc(2);
    bufTrick.writeUInt16LE(proof.trick_id);
    parts.push(bufTrick);
    const bufAirtime = Buffer.alloc(4);
    bufAirtime.writeUInt32LE(proof.airtime_ms);
    parts.push(bufAirtime);
    const bufRot = Buffer.alloc(2);
    bufRot.writeUInt16LE(proof.rotation_deg);
    parts.push(bufRot);
    parts.push(Buffer.from([proof.confidence]));

    const message = Buffer.concat(parts);
    const sig = nacl.sign.detached(new Uint8Array(message), device.secretKey.slice(0, 32));

    // Build ed25519 verify instruction
    const ed25519Ix = anchor.web3.Ed25519Program.createInstructionWithPublicKey({
      publicKey: device.publicKey.toBuffer(),
      message,
      signature: Buffer.from(sig),
    });

    // Compose anchor instruction to call submit_motion_proof
    // Requires program deployed and correct account PDAs. Provide env vars for those.
    const submitIx = await program.methods
      .submitMotionProof(proof, 0)
      .accounts({
        globalConfig: new anchor.web3.PublicKey(process.env.GLOBAL_CONFIG || ''),
        fusionMint: new anchor.web3.PublicKey(process.env.FUSION_MINT || ''),
        treasuryVault: new anchor.web3.PublicKey(process.env.TREASURY_VAULT || ''),
        device: new anchor.web3.PublicKey(process.env.DEVICE_ACCOUNT || ''),
        motionRecord: new anchor.web3.PublicKey(process.env.MOTION_RECORD || ''),
        badge: new anchor.web3.PublicKey(process.env.BADGE_ACCOUNT || ''),
        recipientFusion: new anchor.web3.PublicKey(process.env.RECIPIENT_FUSION || ''),
        payer: payer.publicKey,
        instructionsSysvar: anchor.web3.SYSVAR_INSTRUCTIONS_PUBKEY,
        tokenProgram: anchor.web3.TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .instruction();

    const tx = new Transaction().add(ed25519Ix as any, submitIx as any);
    tx.feePayer = payer.publicKey;
    tx.recentBlockhash = (await provider.connection.getLatestBlockhash()).blockhash;

    // send tx only if RUN_INTEGRATION=1 and env provides account PDAs and program is deployed
    if (process.env.SEND_TX) {
      const sigTx = await provider.sendAndConfirm(tx, [payer]);
      expect(sigTx).to.be.a('string');
    } else {
      // just validate that message/signature sizes are non-zero
      expect(message.length).to.be.greaterThan(0);
      expect(sig.length).to.equal(64);
    }
  });
});
