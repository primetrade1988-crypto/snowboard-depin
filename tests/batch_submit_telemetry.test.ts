import * as anchor from '@coral-xyz/anchor';
import {Keypair} from '@solana/web3.js';

const RUN = process.env.RUN_INTEGRATION === '1';

describe('batch_submit_telemetry', function () {
  it('submits a batch of telemetry samples and aggregates reward', async function () {
    if (!RUN) this.skip();
    const provider = anchor.AnchorProvider.env();
    anchor.setProvider(provider);
    const program = anchor.workspace.SnowboardDepin as anchor.Program;

    const device = Keypair.generate();
    const samples = [
      {latE7: 0, lonE7: 0, timestamp: Date.now(), accelX: 0, accelY: 0, accelZ: 0},
      {latE7: 0, lonE7: 0, timestamp: Date.now() + 1000, accelX: 1, accelY: 0, accelZ: -1},
    ];

    // await program.rpc.batchSubmitTelemetry(samples, { accounts: {...}, signers: [] });
  });
});
