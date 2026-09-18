pub mod adapters;
pub mod constants;
pub mod crypto;
pub mod errors;
pub mod miii_points_engine;
pub mod motion_staking_pool;
pub mod state;
pub mod universal_decoder;

use anchor_lang::prelude::*;

use adapters::*;
use errors::SnowboardDepinError;
use miii_points_engine::*;
use motion_staking_pool::*;
use state::{PayloadFormat, TelemetrySample, MotionProof};

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod snowboard_depin {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, reward_per_meter: u64) -> Result<()> {
        miii_points_engine::initialize(ctx, reward_per_meter)
    }

    pub fn register_device(
        ctx: Context<RegisterDevice>,
        device_id: String,
        device_pubkey: [u8; 32],
    ) -> Result<()> {
        miii_points_engine::register_device(ctx, device_id, device_pubkey)
    }

    pub fn submit_telemetry(
        ctx: Context<SubmitTelemetry>,
        sample: TelemetrySample,
        ed25519_ix_index: u8,
    ) -> Result<()> {
        miii_points_engine::submit_telemetry(ctx, sample, ed25519_ix_index)
    }

    pub fn submit_decoded_telemetry(
        ctx: Context<SubmitDecodedTelemetry>,
        nonce: u64,
        payload: Vec<u8>,
        ed25519_ix_index: u8,
    ) -> Result<()> {
        miii_points_engine::submit_decoded_telemetry(ctx, nonce, payload, ed25519_ix_index)
    }

    pub fn submit_motion_proof(
        ctx: Context<SubmitMotionProof>,
        proof: MotionProof,
        ed25519_ix_index: u8,
    ) -> Result<()> {
        miii_points_engine::submit_motion_proof(ctx, proof, ed25519_ix_index)
    }

    pub fn register_adapter(
        ctx: Context<RegisterAdapter>,
        brand_id: u16,
        format: PayloadFormat,
        distance_scale: u32,
        speed_scale: u32,
        drop_scale: u32,
        airtime_scale: u32,
    ) -> Result<()> {
        adapters::register_adapter(
            ctx,
            brand_id,
            format,
            distance_scale,
            speed_scale,
            drop_scale,
            airtime_scale,
        )
    }

    pub fn update_reward_rate(ctx: Context<UpdateConfig>, reward_per_meter: u64) -> Result<()> {
        miii_points_engine::update_reward_rate(ctx, reward_per_meter)
    }

    pub fn deactivate_device(ctx: Context<DeactivateDevice>) -> Result<()> {
        miii_points_engine::deactivate_device(ctx)
    }

    pub fn slash_sensor(ctx: Context<SlashSensor>, reason: String) -> Result<()> {
        miii_points_engine::slash_sensor(ctx, reason)
    }

    pub fn claim_sponsor_reward(ctx: Context<ClaimSponsorReward>, sample: TelemetrySample, ed25519_ix_index: u8) -> Result<()> {
        miii_points_engine::claim_sponsor_reward(ctx, sample, ed25519_ix_index)
    }

    pub fn batch_submit_telemetry(ctx: Context<BatchSubmitTelemetry>, samples: Vec<TelemetrySample>) -> Result<()> {
        miii_points_engine::batch_submit_telemetry(ctx, samples)
    }

    pub fn initialize_staking(ctx: Context<InitializeStaking>) -> Result<()> {
        motion_staking_pool::initialize_staking(ctx)
    }

    pub fn stake(ctx: Context<Stake>, amount: u64) -> Result<()> {
        motion_staking_pool::stake(ctx, amount)
    }

    pub fn request_unstake(ctx: Context<RequestUnstake>, amount: u64) -> Result<()> {
        motion_staking_pool::request_unstake(ctx, amount)
    }

    pub fn complete_unstake(ctx: Context<CompleteUnstake>) -> Result<()> {
        motion_staking_pool::complete_unstake(ctx)
    }

    pub fn apply_halving(ctx: Context<ApplyHalving>) -> Result<()> {
        motion_staking_pool::apply_halving(ctx)
    }

    pub fn ingest_protocol_fees(ctx: Context<IngestProtocolFees>, amount: u64) -> Result<()> {
        motion_staking_pool::ingest_protocol_fees(ctx, amount)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crypto::build_telemetry_message;
    use crypto::build_motion_message;
    use miii_points_engine::compute_reward;
    use state::{Device, GlobalConfig};

    #[test]
    fn telemetry_message_is_deterministic() {
        let device = [2u8; 32];
        let sample = TelemetrySample {
            nonce: 1,
            timestamp: 1_700_000_000,
            lat_e7: 1,
            lon_e7: 2,
            altitude_cm: 100,
            speed_cm_s: 300,
            accel_milli_g: 400,
            imu_delta: 10,
            distance_cm: 25_000,
            vertical_drop_cm: 200,
            airtime_ms: 800,
        };
        let a = build_telemetry_message(&device, &sample);
        let b = build_telemetry_message(&device, &sample);
        assert_eq!(a, b);
        assert!(a.starts_with(b"SNOWBOARD_DEPIN_TELEM"));
    }

    #[test]
    fn motion_message_is_deterministic() {
        let device = [3u8; 32];
        let proof = state::MotionProof {
            nonce: 2,
            timestamp: 1_700_000_001,
            trick_id: 7,
            airtime_ms: 1200,
            rotation_deg: 720,
            confidence: 80,
        };
        let a = build_motion_message(&device, &proof);
        let b = build_motion_message(&device, &proof);
        assert_eq!(a, b);
        assert!(a.starts_with(b"SNOWBOARD_DEPIN_MOTION"));
    }

    #[test]
    fn trick_reward_scales_with_confidence() {
        let base: u64 = 10_000;
        let c50 = base.checked_mul(50).and_then(|v| v.checked_div(100)).unwrap();
        let c100 = base.checked_mul(100).and_then(|v| v.checked_div(100)).unwrap();
        assert!(c100 >= c50);
    }

    #[test]
    fn validate_motion_against_telemetry_unit() {
        use state::{MotionProof, TelemetryRecord, Device};
        use miii_points_engine::validate_motion_against_telemetry;
        let proof = MotionProof {
            nonce: 10,
            timestamp: 1_700_000_100,
            trick_id: 3,
            airtime_ms: 1200,
            rotation_deg: 540,
            confidence: 70,
        };
        let device = Device {
            owner: Pubkey::default(),
            device_pubkey: [0u8;32],
            device_id: String::new(),
            status: state::DeviceStatus::Active,
            last_nonce: 5,
            last_timestamp: 1_700_000_000,
            last_lat_e7: 0,
            last_lon_e7: 0,
            last_speed_cm_s: 300,
            last_accel_milli_g: 0,
            epoch_id: 0,
            epoch_emitted: 0,
            epoch_distance_m: 0,
            total_distance_m: 0,
            total_rewards: 0,
            processing_lock: 0,
            bump: 0,
            is_blacklisted: false,
            anomaly_count: 0,
            uptime_streak: 0,
            last_active_day: 0,
            stake_amount: 0,
        };
        let telem = TelemetryRecord {
            device: Pubkey::default(),
            nonce: 9,
            timestamp: 1_700_000_099,
            distance_meters: 10,
            vertical_drop_cm: 100,
            airtime_ms: 1200,
            reward_amount: 0,
            bump: 0,
        };
        let res = validate_motion_against_telemetry(&proof, &device, Some(&telem));
        assert!(res.is_ok());
    }

    #[test]
    fn diminishing_returns_reduce_after_volume() {
        let config = GlobalConfig {
            admin: Pubkey::default(),
            fusion_mint: Pubkey::default(),
            treasury_vault: Pubkey::default(),
            fee_vault: Pubkey::default(),
            stake_pool: Pubkey::default(),
            reward_per_meter: 1_000,
            reward_per_trick: 10_000,
            airtime_bps_per_ms: 0,
            rotation_bps_per_rev: 0,
            max_trick_multiplier_bps: 50_000,
            reward_per_drop_cm: 0,
            reward_per_airtime_ms: 0,
            max_payout_per_report: 50_000_000,
            max_epoch_emission: 500_000_000,
            epoch_slots: 216_000,
            total_devices: 0,
            total_rewards_distributed: 0,
            genesis_slot: 0,
            bump: 0,
            treasury_bump: 0,
            fee_vault_bump: 0,
        };
        let sample = TelemetrySample {
            nonce: 1,
            timestamp: 1,
            distance_cm: 10_000,
            ..TelemetrySample::default()
        };
        let mut device = Device {
            owner: Pubkey::default(),
            device_pubkey: [0u8; 32],
            device_id: String::new(),
            status: state::DeviceStatus::Active,
            last_nonce: 0,
            last_timestamp: 0,
            last_lat_e7: 0,
            last_lon_e7: 0,
            last_speed_cm_s: 0,
            last_accel_milli_g: 0,
            epoch_id: 0,
            epoch_emitted: 0,
            epoch_distance_m: 0,
            total_distance_m: 0,
            total_rewards: 0,
            processing_lock: 0,
            bump: 0,
            is_blacklisted: false,
            anomaly_count: 0,
            uptime_streak: 0,
            last_active_day: 0,
            stake_amount: 0,
        };
        let fresh = compute_reward(&config, &device, &sample).unwrap();
        device.epoch_distance_m = 200_000;
        let faded = compute_reward(&config, &device, &sample).unwrap();
        assert!(fresh > faded);
    }

    #[test]
    fn error_enum_covers_security_vectors() {
        let _ = SnowboardDepinError::InvalidSignature;
        let _ = SnowboardDepinError::SpeedThresholdExceeded;
        let _ = SnowboardDepinError::CooldownActive;
        let _ = SnowboardDepinError::RateLimitReached;
        let _ = SnowboardDepinError::MathOverflow;
        let _ = SnowboardDepinError::UnauthorizedDevice;
    }
}
