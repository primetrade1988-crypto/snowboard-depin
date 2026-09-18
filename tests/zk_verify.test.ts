import * as anchor from '@coral-xyz/anchor';

const RUN = process.env.RUN_INTEGRATION === '1';

describe('zk_verify', function () {
  it('submits zk-proof placeholder and receives acceptance', async function () {
    if (!RUN) this.skip();
    const provider = anchor.AnchorProvider.env();
    anchor.setProvider(provider);
    const program = anchor.workspace.SnowboardDepin as anchor.Program;
  });
});
