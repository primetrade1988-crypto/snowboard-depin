use anchor_lang::prelude::*;

#[error_code]
pub enum SnowboardDepinError {
    #[msg("Device id must be non-empty and within max length")]
    InvalidDeviceId,
    #[msg("Device is inactive")]
    DeviceInactive,
    #[msg("Unauthorized device")]
    UnauthorizedDevice,
    #[msg("Telemetry nonce must be strictly greater than the last recorded nonce")]
    InvalidNonce,
    #[msg("Distance must be within allowed bounds")]
    InvalidDistance,
    #[msg("Telemetry timestamp is invalid")]
    InvalidTimestamp,
    #[msg("Treasury vault has insufficient $FUSION balance")]
    InsufficientTreasury,
    #[msg("Reward recipient must be owned by the device owner")]
    InvalidRecipient,
    #[msg("Recipient token account mint mismatch")]
    InvalidMint,
    #[msg("Arithmetic overflow")]
    MathOverflow,
    #[msg("Invalid or missing Ed25519 verify instruction")]
    InvalidEd25519Instruction,
    #[msg("Ed25519 public key does not match registered device")]
    Ed25519PubkeyMismatch,
    #[msg("Ed25519 signed message does not match expected payload")]
    Ed25519MessageMismatch,
        #[msg("Invalid or malformed zk-SNARK proof")]
        InvalidZkProof,
    #[msg("Invalid hardware signature")]
    InvalidSignature,
    #[msg("Reward rate must be greater than zero")]
    InvalidRewardRate,
    #[msg("Speed vector exceeds physical threshold")]
    SpeedThresholdExceeded,
    #[msg("Acceleration exceeds IMU / g-force cap")]
    AccelerationCapExceeded,
    #[msg("IMU delta exceeds continuity threshold")]
    ImuDeltaThresholdExceeded,
    #[msg("Spatial discontinuity — likely GPS spoofing")]
    SpatialContinuityFailed,
    #[msg("Replay or out-of-order sample detected")]
    ReplayDetected,
    #[msg("Sliding-window rate limit reached for this epoch")]
    RateLimitReached,
    #[msg("Stake amount below tier minimum")]
    InsufficientStake,
    #[msg("Unbonding cooldown is still active")]
    CooldownActive,
    #[msg("No unbonding request pending")]
    NoUnbondPending,
    #[msg("Stake vault has insufficient $FUSION")]
    InsufficientStakeVault,
    #[msg("Unknown or inactive hardware adapter")]
    UnknownAdapter,
    #[msg("Payload too large or truncated")]
    InvalidPayload,
    #[msg("Decoder failed to normalize hardware payload")]
    DecodeFailed,
    #[msg("Re-entrancy guard is locked")]
    Reentrancy,
    #[msg("Motion proof signature invalid or malformed")]
    InvalidMotionProof,
    #[msg("Trick proof rejected — failed motion validation")]
    ProofOfMotionFailed,
    #[msg("Badge already exists or cannot be awarded")]
    BadgeAwardFailed,
    #[msg("Device is blacklisted for repeated anomalies")]
    DeviceBlacklisted,
    #[msg("Sensor slashing executed")]
    SensorSlashed,
    #[msg("Sponsor claim failed or not eligible")]
    SponsorClaimFailed,
    #[msg("Batch payload too large")]
    BatchTooLarge,
    #[msg("Halving interval has not elapsed")]
    HalvingNotDue,
    #[msg("Protocol fee amount is zero")]
    ZeroFee,
}
