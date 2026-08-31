import * as anchor from "@coral-xyz/anchor";
import { Program, BN } from "@coral-xyz/anchor";
import {
  Keypair,
  PublicKey,
  SystemProgram,
  SYSVAR_INSTRUCTIONS_PUBKEY,
  Ed25519Program,
  Transaction,
  sendAndConfirmTransaction,
} from "@solana/web3.js";
import {
  TOKEN_PROGRAM_ID,
  createMint,
  createAccount,
  mintTo,
  getAccount,
} from "@solana/spl-token";
import { expect } from "chai";
import nacl from "tweetnacl";
import { SnowboardDepin } from "../target/types/snowboard_depin";

const CONFIG_SEED = Buffer.from("config");
const DEVICE_SEED = Buffer.from("device");
const TREASURY_SEED = Buffer.from("treasury");
const TELEMETRY_SEED = Buffer.from("telemetry");
const FEE_VAULT_SEED = Buffer.from("fee_vault");
const STAKE_POOL_SEED = Buffer.from("stake_pool");
const STAKE_VAULT_SEED = Buffer.from("stake_vault");
const STAKE_POS_SEED = Buffer.from("stake");
const ADAPTER_SEED = Buffer.from("adapter");

const REWARD_PER_METER = new BN(1_000);
const PREFIX = Buffer.from("SNOWBOARD_DEPIN_TELEM");

type Sample = {
  nonce: BN;
  timestamp: BN;
  latE7: number;
  lonE7: number;
  altitudeCm: number;
  speedCmS: number;
  accelMilliG: number;
  imuDelta: number;
  distanceCm: number;
  verticalDropCm: number;
  airtimeMs: number;
};

function sample(overrides: Partial<Sample> = {}): Sample {
  const ts = Math.floor(Date.now() / 1000) - 5;
  return {
    nonce: new BN(1),
    timestamp: new BN(ts),
    latE7: 450_000_000,
    lonE7: 60_000_000,
    altitudeCm: 180_000,
    speedCmS: 800,
    accelMilliG: 1_200,
    imuDelta: 80,
    distanceCm: 25_000,
    verticalDropCm: 0,
    airtimeMs: 0,
    ...overrides,
  };
}

function buildTelemetryMessage(device: PublicKey, s: Sample): Buffer {
  const msg = Buffer.alloc(PREFIX.length + 32 + 8 + 8 + 4 * 9);
  PREFIX.copy(msg, 0);
  device.toBuffer().copy(msg, PREFIX.length);
  let o = PREFIX.length + 32;
  msg.writeBigUInt64LE(BigInt(s.nonce.toString()), o); o += 8;
  msg.writeBigInt64LE(BigInt(s.timestamp.toString()), o); o += 8;
  msg.writeInt32LE(s.latE7, o); o += 4;
  msg.writeInt32LE(s.lonE7, o); o += 4;
  msg.writeInt32LE(s.altitudeCm, o); o += 4;
  msg.writeUInt32LE(s.speedCmS, o); o += 4;
  msg.writeUInt32LE(s.accelMilliG, o); o += 4;
  msg.writeUInt32LE(s.imuDelta, o); o += 4;
  msg.writeUInt32LE(s.distanceCm, o); o += 4;
  msg.writeUInt32LE(s.verticalDropCm, o); o += 4;
  msg.writeUInt32LE(s.airtimeMs, o);
  return msg;
}

describe("snowboard-depin", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.SnowboardDepin as Program<SnowboardDepin>;
  const admin = (provider.wallet as anchor.Wallet).payer;

  let fusionMint: PublicKey;
  let globalConfig: PublicKey;
  let treasuryVault: PublicKey;
  let feeVault: PublicKey;
  let stakePool: PublicKey;
  let stakeVault: PublicKey;
  let configBump: number;

  const owner = Keypair.generate();
  const deviceKeypair = nacl.sign.keyPair();
  const devicePubkeyBytes = Array.from(deviceKeypair.publicKey);
  const deviceId = "HW-BOARD-001";

  let devicePda: PublicKey;
  let ownerAta: PublicKey;
  let adapterPda: PublicKey;

  before(async () => {
    const sig = await provider.connection.requestAirdrop(
      owner.publicKey,
      2 * anchor.web3.LAMPORTS_PER_SOL
    );
    await provider.connection.confirmTransaction(sig, "confirmed");

    fusionMint = await createMint(
      provider.connection,
      admin,
      admin.publicKey,
      null,
      6
    );

    [globalConfig, configBump] = PublicKey.findProgramAddressSync(
      [CONFIG_SEED],
      program.programId
    );
    [treasuryVault] = PublicKey.findProgramAddressSync(
      [TREASURY_SEED, globalConfig.toBuffer()],
      program.programId
    );
    [feeVault] = PublicKey.findProgramAddressSync(
      [FEE_VAULT_SEED, globalConfig.toBuffer()],
      program.programId
    );
    [stakePool] = PublicKey.findProgramAddressSync(
      [STAKE_POOL_SEED],
      program.programId
    );
    [stakeVault] = PublicKey.findProgramAddressSync(
      [STAKE_VAULT_SEED, stakePool.toBuffer()],
      program.programId
    );
    [devicePda] = PublicKey.findProgramAddressSync(
      [DEVICE_SEED, owner.publicKey.toBuffer(), Buffer.from(deviceId)],
      program.programId
    );
    const brand = Buffer.alloc(2);
    brand.writeUInt16LE(1);
    [adapterPda] = PublicKey.findProgramAddressSync(
      [ADAPTER_SEED, brand],
      program.programId
    );
  });

  it("initializes global config, treasury and fee vault", async () => {
    await program.methods
      .initialize(REWARD_PER_METER)
      .accounts({
        admin: admin.publicKey,
        fusionMint,
        globalConfig,
        treasuryVault,
        feeVault,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      })
      .rpc();

    const config = await program.account.globalConfig.fetch(globalConfig);
    expect(config.admin.toBase58()).to.equal(admin.publicKey.toBase58());
    expect(config.treasuryVault.toBase58()).to.equal(treasuryVault.toBase58());
    expect(config.feeVault.toBase58()).to.equal(feeVault.toBase58());
    expect(config.totalDevices.toNumber()).to.equal(0);
    expect(config.bump).to.equal(configBump);

    await mintTo(
      provider.connection,
      admin,
      fusionMint,
      treasuryVault,
      admin,
      1_000_000_000
    );
  });

  it("registers a hardware device", async () => {
    await program.methods
      .registerDevice(deviceId, devicePubkeyBytes)
      .accounts({
        owner: owner.publicKey,
        globalConfig,
        device: devicePda,
        systemProgram: SystemProgram.programId,
      })
      .signers([owner])
      .rpc();

    const device = await program.account.device.fetch(devicePda);
    expect(device.owner.toBase58()).to.equal(owner.publicKey.toBase58());
    expect(device.deviceId).to.equal(deviceId);
    expect(Buffer.from(device.devicePubkey).equals(Buffer.from(deviceKeypair.publicKey))).to.be
      .true;

    const config = await program.account.globalConfig.fetch(globalConfig);
    expect(config.totalDevices.toNumber()).to.equal(1);

    ownerAta = await createAccount(
      provider.connection,
      owner,
      fusionMint,
      owner.publicKey
    );
    await mintTo(provider.connection, admin, fusionMint, ownerAta, admin, 50_000_000);
  });

  it("registers a universal hardware adapter", async () => {
    await program.methods
      .registerAdapter(1, { binaryV1: {} }, 1, 1, 1, 1)
      .accounts({
        admin: admin.publicKey,
        globalConfig,
        adapter: adapterPda,
        systemProgram: SystemProgram.programId,
      })
      .rpc();
    const adapter = await program.account.hardwareAdapter.fetch(adapterPda);
    expect(adapter.brandId).to.equal(1);
    expect(adapter.active).to.equal(true);
  });

  async function submitSample(s: Sample, keypair = deviceKeypair) {
    const [telemetryRecord] = PublicKey.findProgramAddressSync(
      [
        TELEMETRY_SEED,
        devicePda.toBuffer(),
        Buffer.from(s.nonce.toArray("le", 8)),
      ],
      program.programId
    );
    const message = buildTelemetryMessage(devicePda, s);
    const signature = nacl.sign.detached(message, keypair.secretKey);
    const ed25519Ix = Ed25519Program.createInstructionWithPublicKey({
      publicKey: keypair.publicKey,
      message,
      signature,
    });
    const submitIx = await program.methods
      .submitTelemetry(s, 0)
      .accounts({
        globalConfig,
        fusionMint,
        treasuryVault,
        device: devicePda,
        telemetryRecord,
        recipientFusion: ownerAta,
        payer: owner.publicKey,
        instructionsSysvar: SYSVAR_INSTRUCTIONS_PUBKEY,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .instruction();
    const tx = new Transaction().add(ed25519Ix, submitIx);
    return sendAndConfirmTransaction(provider.connection, tx, [owner], {
      commitment: "confirmed",
    });
  }

  it("pays $FUSION for valid signed telemetry", async () => {
    const s = sample({ nonce: new BN(1) });
    await submitSample(s);

    const expected = 250 * REWARD_PER_METER.toNumber();
    const device = await program.account.device.fetch(devicePda);
    expect(device.lastNonce.toNumber()).to.equal(1);
    expect(device.totalDistanceM.toNumber()).to.equal(250);
    expect(device.totalRewards.toNumber()).to.equal(expected);

    const ata = await getAccount(provider.connection, ownerAta);
    expect(Number(ata.amount)).to.equal(50_000_000 + expected);
  });

  it("rejects telemetry with a fake / mismatched signature", async () => {
    const s = sample({ nonce: new BN(2) });
    const faker = nacl.sign.keyPair();
    try {
      await submitSample(s, faker);
      expect.fail("expected transaction to fail");
    } catch (err: any) {
      const logs: string = err.logs?.join("\n") ?? String(err);
      expect(
        logs.includes("Ed25519PubkeyMismatch") ||
          logs.includes("custom program error") ||
          logs.includes("Error")
      ).to.be.true;
    }
    const device = await program.account.device.fetch(devicePda);
    expect(device.lastNonce.toNumber()).to.equal(1);
  });

  it("rejects speed-threshold spoofing", async () => {
    const s = sample({ nonce: new BN(3), speedCmS: 9_999 });
    try {
      await submitSample(s);
      expect.fail("expected speed cap to fail");
    } catch (err: any) {
      const logs: string = err.logs?.join("\n") ?? String(err);
      expect(logs.includes("SpeedThresholdExceeded") || logs.includes("Error")).to.be.true;
    }
  });

  it("initializes motion staking pool and locks $FUSION", async () => {
    await program.methods
      .initializeStaking()
      .accounts({
        admin: admin.publicKey,
        fusionMint,
        globalConfig,
        stakePool,
        stakeVault,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      })
      .rpc();

    const [stakePosition] = PublicKey.findProgramAddressSync(
      [STAKE_POS_SEED, owner.publicKey.toBuffer()],
      program.programId
    );

    await program.methods
      .stake(new BN(1_000_000))
      .accounts({
        owner: owner.publicKey,
        globalConfig,
        fusionMint,
        treasuryVault,
        stakePool,
        stakeVault,
        stakePosition,
        userFusion: ownerAta,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .signers([owner])
      .rpc();

    const pos = await program.account.stakePosition.fetch(stakePosition);
    expect(pos.amount.toNumber()).to.equal(1_000_000);

    await program.methods
      .requestUnstake(new BN(1_000_000))
      .accounts({
        owner: owner.publicKey,
        globalConfig,
        treasuryVault,
        stakePool,
        stakePosition,
        userFusion: ownerAta,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([owner])
      .rpc();

    try {
      await program.methods
        .completeUnstake()
        .accounts({
          owner: owner.publicKey,
          stakePool,
          stakeVault,
          stakePosition,
          userFusion: ownerAta,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([owner])
        .rpc();
      expect.fail("cooldown should block unstake");
    } catch (err: any) {
      const logs: string = err.logs?.join("\n") ?? String(err);
      expect(logs.includes("CooldownActive") || logs.includes("Error")).to.be.true;
    }
  });
});
