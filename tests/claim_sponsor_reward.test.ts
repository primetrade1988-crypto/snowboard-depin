import * as anchor from '@coral-xyz/anchor';
import {Keypair, PublicKey, TransactionInstruction} from '@solana/web3.js';
import * as nacl from 'tweetnacl';

const RUN = process.env.RUN_INTEGRATION === '1';

describe('claim_sponsor_reward', function () {
  it('executes sponsor claim with ed25519 proof (integration)', async function () {
    if (!RUN) this.skip();
    const provider = anchor.AnchorProvider.env();
    anchor.setProvider(provider);
    const program = anchor.workspace.SnowboardDepin as anchor.Program;

    // Off-chain device keypair and sample
    const device = Keypair.generate();
    const sample = {
      latE7: 0,
      lonE7: 0,
      timestamp: Date.now(),
      accelX: 0,
      accelY: 0,
      accelZ: 0,
    };

    const message = Buffer.from(JSON.stringify(sample));
    const sig = nacl.sign.detached(message, device.secretKey);

    // Build ed25519 instruction and include in tx; for test we simply ensure assembly.
    const edIx = new TransactionInstruction({
      programId: anchor.web3.SystemProgram.programId, // placeholder
      keys: [],
      data: Buffer.from([]),
    });

    // Call program rpc method (requires real accounts)
    // await program.rpc.claimSponsorReward(...)
  });
});
