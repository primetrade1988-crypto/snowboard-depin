use anchor_lang::prelude::*;

use crate::constants::MAX_DEVICE_ID_LEN;

#[account]
#[derive(InitSpace)]
pub struct GlobalConfig {
    pub admin: Pubkey,
    pub fusion_mint: Pubkey,
    pub treasury_vault: Pubkey,
    pub fee_vault: Pubkey,
    pub stake_pool: Pubkey,
    pub reward_per_meter: u64,
    pub reward_per_trick: u64,
    pub airtime_bps_per_ms: u64,
    pub rotation_bps_per_rev: u64,
    pub max_trick_multiplier_bps: u64,
    pub reward_per_drop_cm: u64,
    pub reward_per_airtime_ms: u64,
    pub max_payout_per_report: u64,
    pub max_epoch_emission: u64,
    pub epoch_slots: u64,
    pub total_devices: u64,
    pub total_rewards_distributed: u64,
    pub genesis_slot: u64,
    pub bump: u8,
    pub treasury_bump: u8,
    pub fee_vault_bump: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, InitSpace)]
pub enum DeviceStatus {
    Active,
    Inactive,
}

#[account]
#[derive(InitSpace)]
pub struct Device {
    pub owner: Pubkey,
    pub device_pubkey: [u8; 32],
    #[max_len(MAX_DEVICE_ID_LEN)]
    pub device_id: String,
    pub status: DeviceStatus,
    pub last_nonce: u64,
    pub last_timestamp: i64,
    pub last_lat_e7: i32,
    pub last_lon_e7: i32,
    pub last_speed_cm_s: u32,
    pub last_accel_milli_g: u32,
    pub epoch_id: u64,
    pub epoch_emitted: u64,
    pub epoch_distance_m: u64,
    pub total_distance_m: u64,
    pub total_rewards: u64,
    pub processing_lock: u8,
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct TelemetryRecord {
    pub device: Pubkey,
    pub nonce: u64,
    pub timestamp: i64,
    pub distance_meters: u32,
    pub vertical_drop_cm: u32,
    pub airtime_ms: u32,
    pub reward_amount: u64,
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct MotionRecord {
    pub device: Pubkey,
    pub nonce: u64,
    pub timestamp: i64,
    pub trick_id: u16,
    pub airtime_ms: u32,
    pub rotation_deg: u16,
    pub reward_amount: u64,
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct Badge {
    pub owner: Pubkey,
    pub device: Pubkey,
    pub badge_id: u16,
    pub count: u64,
    pub last_awarded_at: i64,
    pub bump: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, InitSpace)]
pub enum StakeTier {
    Flex = 0,
    Core = 1,
    Elite = 2,
}

#[account]
#[derive(InitSpace)]
pub struct StakePool {
    pub admin: Pubkey,
    pub fusion_mint: Pubkey,
    pub stake_vault: Pubkey,
    pub total_staked: u64,
    pub reward_index: u128,
    pub emission_per_slot: u64,
    pub last_update_slot: u64,
    pub last_halving_slot: u64,
    pub halving_interval_slots: u64,
    pub unbond_slots: u64,
    pub burn_bps: u16,
    pub staker_share_bps: u16,
    pub bump: u8,
    pub vault_bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct StakePosition {
    pub owner: Pubkey,
    pub amount: u64,
    pub tier: StakeTier,
    pub reward_debt: u128,
    pub pending_unbond: u64,
    pub unbond_complete_slot: u64,
    pub bump: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, InitSpace)]
pub enum PayloadFormat {
    BinaryV1,
    TaggedMap,
}

#[account]
#[derive(InitSpace)]
pub struct HardwareAdapter {
    pub admin: Pubkey,
    pub brand_id: u16,
    pub format: PayloadFormat,
    pub distance_scale: u32,
    pub speed_scale: u32,
    pub drop_scale: u32,
    pub airtime_scale: u32,
    pub active: bool,
    pub bump: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Default)]
pub struct TelemetrySample {
    pub nonce: u64,
    pub timestamp: i64,
    pub lat_e7: i32,
    pub lon_e7: i32,
    pub altitude_cm: i32,
    pub speed_cm_s: u32,
    pub accel_milli_g: u32,
    pub imu_delta: u32,
    pub distance_cm: u32,
    pub vertical_drop_cm: u32,
    pub airtime_ms: u32,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct MotionProof {
    pub nonce: u64,
    pub timestamp: i64,
    pub trick_id: u16,
    pub airtime_ms: u32,
    pub rotation_deg: u16,
    pub confidence: u8,
}

impl StakeTier {
    pub fn min_amount(self) -> u64 {
        match self {
            StakeTier::Flex => 1,
            StakeTier::Core => 10_000_000,
            StakeTier::Elite => 100_000_000,
        }
    }

    pub fn multiplier_bps(self) -> u64 {
        match self {
            StakeTier::Flex => 10_000,
            StakeTier::Core => 12_500,
            StakeTier::Elite => 16_000,
        }
    }

    pub fn from_amount(amount: u64) -> Self {
        if amount >= StakeTier::Elite.min_amount() {
            StakeTier::Elite
        } else if amount >= StakeTier::Core.min_amount() {
            StakeTier::Core
        } else {
            StakeTier::Flex
        }
    }
}
