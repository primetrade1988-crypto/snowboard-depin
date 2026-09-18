import * as anchor from '@coral-xyz/anchor';

const RUN = process.env.RUN_INTEGRATION === '1';

describe('sponsor_yield', function () {
  it('deposits escrow into yield pool (placeholder)', async function () {
    if (!RUN) this.skip();
    const provider = anchor.AnchorProvider.env();
    anchor.setProvider(provider);
    const program = anchor.workspace.SnowboardDepin as anchor.Program;
  });
});
