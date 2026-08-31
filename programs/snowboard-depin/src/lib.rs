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
use state::{PayloadFormat, TelemetrySample};

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
    fn diminishing_returns_reduce_after_volume() {
        let config = GlobalConfig {
            admin: Pubkey::default(),
            fusion_mint: Pubkey::default(),
            treasury_vault: Pubkey::default(),
            fee_vault: Pubkey::default(),
            stake_pool: Pubkey::default(),
            reward_per_meter: 1_000,
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
