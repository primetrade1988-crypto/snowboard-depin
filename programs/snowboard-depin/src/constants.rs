pub const CONFIG_SEED: &[u8] = b"config";
pub const DEVICE_SEED: &[u8] = b"device";
pub const TREASURY_SEED: &[u8] = b"treasury";
pub const TELEMETRY_SEED: &[u8] = b"telemetry";
pub const STAKE_POOL_SEED: &[u8] = b"stake_pool";
pub const STAKE_VAULT_SEED: &[u8] = b"stake_vault";
pub const STAKE_POS_SEED: &[u8] = b"stake";
pub const ADAPTER_SEED: &[u8] = b"adapter";
pub const FEE_VAULT_SEED: &[u8] = b"fee_vault";

pub const MAX_DEVICE_ID_LEN: usize = 64;
pub const MAX_PAYLOAD_LEN: usize = 256;

pub const DEFAULT_REWARD_PER_METER: u64 = 1_000;
pub const DEFAULT_REWARD_PER_DROP_CM: u64 = 2;
pub const DEFAULT_REWARD_PER_AIRTIME_MS: u64 = 5;
pub const DEFAULT_REWARD_PER_TRICK: u64 = 10_000;
pub const DEFAULT_MAX_PAYOUT: u64 = 50_000_000;
pub const DEFAULT_MAX_EPOCH_EMISSION: u64 = 500_000_000;
pub const DEFAULT_EPOCH_SLOTS: u64 = 216_000;

pub const MAX_DISTANCE_CM: u32 = 10_000_000;
pub const MAX_SPEED_CM_S: u32 = 5_000;
pub const MAX_ACCEL_MILLI_G: u32 = 8_000;
pub const MAX_IMU_DELTA: u32 = 12_000;
pub const MAX_SAMPLE_DT_SECS: i64 = 120;
pub const DIMINISH_K: u64 = 50_000;

pub const BADGE_SEED: &[u8] = b"badge";
pub const MOTION_SEED: &[u8] = b"motion";
pub const MAX_TRICK_AIRTIME_MS: u32 = 10_000;
pub const MAX_TRICK_ROTATION_DEG: u16 = 3600;
pub const TRICK_COOLDOWN_SECS: i64 = 10;

pub const DEFAULT_EMISSION_PER_SLOT: u64 = 100;
pub const DEFAULT_HALVING_INTERVAL: u64 = 15_768_000;
pub const DEFAULT_UNBOND_SLOTS: u64 = 21_600;
pub const DEFAULT_BURN_BPS: u16 = 3_000;
pub const DEFAULT_STAKER_SHARE_BPS: u16 = 7_000;
pub const BPS_DENOM: u64 = 10_000;
pub const INDEX_SCALE: u128 = 1_000_000_000_000;
