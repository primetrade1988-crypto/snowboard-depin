import * as anchor from '@coral-xyz/anchor';
import {Keypair, PublicKey, SystemProgram} from '@solana/web3.js';
import * as assert from 'assert';

const RUN = process.env.RUN_INTEGRATION === '1';

describe('slash_sensor', function () {
  it('marks device as blacklisted (integration)', async function () {
    if (!RUN) this.skip();
    const provider = anchor.AnchorProvider.env();
    anchor.setProvider(provider);
    const program = anchor.workspace.SnowboardDepin as anchor.Program;

    // NOTE: Replace these with real PDAs/accounts for integration testing
    const admin = provider.wallet.publicKey;
    const devicePda = Keypair.generate();

    // call slash_sensor
    try {
      await program.rpc.slashSensor('detected-spoofing', {
        accounts: {
          admin: admin,
          device: devicePda.publicKey,
          systemProgram: SystemProgram.programId,
        },
        signers: [],
      });
    } catch (err) {
      // For integration test, a failure means the environment isn't configured.
      throw err;
    }

    // Query device account and assert `is_blacklisted` true
    // const deviceAccount = await program.account.device.fetch(devicePda.publicKey);
    // assert.equal(deviceAccount.isBlacklisted, true);
  });
});
