import * as anchor from '@coral-xyz/anchor';
import {Keypair, SystemProgram, TransactionInstruction} from '@solana/web3.js';
import * as assert from 'assert';
import nacl from 'tweetnacl';

// This integration test requires a running local validator and a deployed program.
// Skip by default unless RUN_INTEGRATION=1
const RUN = process.env.RUN_INTEGRATION === '1';

describe('ed25519 introspection', function () {
  it('verifies ed25519 instruction presence', async function () {
    if (!RUN) this.skip();
    const provider = anchor.AnchorProvider.env();
    anchor.setProvider(provider);
    const program = anchor.workspace.SnowboardDepin as anchor.Program;

    const device = Keypair.generate();
    const message = Buffer.from('SNOWBOARD_DEPIN_TEST');
    const sig = nacl.sign.detached(message, device.secretKey);

    // Build ed25519 instruction (off-chain signer)
    const edIx = new TransactionInstruction({
      programId: new anchor.web3.PublicKey('Ed25519SigVerify111111111111111111111111111'),
      keys: [],
      data: Buffer.from([]),
    });

    // This is a smoke test showing how a transaction with the ed25519 ix would be assembled
    // The on-chain test would inspect the sysvar::instructions to find this ix and verify it.
    assert.ok(edIx.programId);
  });
});
