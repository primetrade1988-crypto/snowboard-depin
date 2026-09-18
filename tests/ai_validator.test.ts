import * as anchor from '@coral-xyz/anchor';

const RUN = process.env.RUN_INTEGRATION === '1';

describe('ai_validator', function () {
  it('allows ai validator to sign trick (placeholder)', async function () {
    if (!RUN) this.skip();
    const provider = anchor.AnchorProvider.env();
    anchor.setProvider(provider);
    const program = anchor.workspace.SnowboardDepin as anchor.Program;
  });
});
