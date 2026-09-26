use anchor_lang::prelude::*;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

/// MIII Protocol
///
/// Core principle:
/// Physical Device
///     -> Device Signature
///     -> Protocol Validation
///     -> Immutable Motion Record
///
/// Tokenomics are intentionally kept outside this core layer.
/// The physical verification layer must remain useful independently
/// from token price/speculation.

#[program]
pub mod miii_protocol {
    use super::*;

    // ------------------------------------------------------------
    // DEVICE REGISTRATION
    // ------------------------------------------------------------

    pub fn register_device(
        ctx: Context<RegisterDevice>,
        device_pubkey: Pubkey,
    ) -> Result<()> {
        let registry = &mut ctx.accounts.device_registry;

        registry.owner = ctx.accounts.owner.key();
        registry.device_pubkey = device_pubkey;
        registry.is_active = true;
        registry.sequence = 0;
        registry.last_timestamp = 0;
        registry.bump = ctx.bumps.device_registry;

        emit!(DeviceRegistered {
            device: device_pubkey,
            owner: registry.owner,
        });

        Ok(())
    }

    // ------------------------------------------------------------
    // ENABLE / DISABLE DEVICE
    // ------------------------------------------------------------

    pub fn set_device_status(
        ctx: Context<SetDeviceStatus>,
        active: bool,
    ) -> Result<()> {
        let registry = &mut ctx.accounts.device_registry;

        registry.is_active = active;

        emit!(DeviceStatusChanged {
            device: registry.device_pubkey,
            active,
        });

        Ok(())
    }

    // ------------------------------------------------------------
    // TRANSFER DEVICE OWNERSHIP
    // ------------------------------------------------------------

    pub fn transfer_device(
        ctx: Context<TransferDevice>,
        new_owner: Pubkey,
    ) -> Result<()> {
        require!(
            new_owner != Pubkey::default(),
            CustomError::InvalidOwner
        );

        let registry = &mut ctx.accounts.device_registry;

        let previous_owner = registry.owner;

        registry.owner = new_owner;

        emit!(DeviceTransferred {
            device: registry.device_pubkey,
            previous_owner,
            new_owner,
        });

        Ok(())
    }

    // ------------------------------------------------------------
    // RECORD VERIFIED MOTION
    // ------------------------------------------------------------

    pub fn record_motion(
        ctx: Context<RecordMotion>,
        trick_type: u8,
        g_force: u32,
        timestamp: i64,
        sequence: u64,
    ) -> Result<()> {
        let registry = &mut ctx.accounts.device_registry;
        let event = &mut ctx.accounts.motion_event;

        let current_time = Clock::get()?.unix_timestamp;

        // --------------------------------------------------------
        // DEVICE STATUS
        // --------------------------------------------------------

        require!(
            registry.is_active,
            CustomError::DeviceInactive
        );

        // --------------------------------------------------------
        // OWNER CHECK
        // --------------------------------------------------------

        require!(
            registry.owner == ctx.accounts.user.key(),
            CustomError::UnauthorizedUser
        );

        // --------------------------------------------------------
        // DEVICE SIGNATURE CHECK
        //
        // physical_device MUST sign the Solana transaction.
        // Its public key must match the registered physical device.
        // --------------------------------------------------------

        require!(
            registry.device_pubkey == ctx.accounts.physical_device.key(),
            CustomError::InvalidDevice
        );

        // --------------------------------------------------------
        // TIMESTAMP VALIDATION
        //
        // Event cannot be too old or too far in the future.
        // --------------------------------------------------------

        let delta = current_time
            .checked_sub(timestamp)
            .ok_or(error!(CustomError::TimestampOverflow))?;

        require!(
            delta >= -MOTION_CLOCK_TOLERANCE
                && delta <= MOTION_CLOCK_TOLERANCE,
            CustomError::InvalidTimestamp
        );

        // --------------------------------------------------------
        // MONOTONIC SEQUENCE
        //
        // Prevents replaying the same signed event.
        // --------------------------------------------------------

        require!(
            sequence == registry.sequence,
            CustomError::InvalidSequence
        );

        // --------------------------------------------------------
        // MONOTONIC TIMESTAMP
        // --------------------------------------------------------

        if registry.last_timestamp != 0 {
            require!(
                timestamp > registry.last_timestamp,
                CustomError::TimestampNotIncreasing
            );
        }

        // --------------------------------------------------------
        // PHYSICAL PLAUSIBILITY
        //
        // g_force is stored as hundredths of G.
        //
        // 1500 == 15.00G
        // --------------------------------------------------------

        require!(
            g_force <= MAX_G_FORCE,
            CustomError::PhysicsLimitExceeded
        );

        // --------------------------------------------------------
        // TRICK VALIDATION
        //
        // 0..=MAX_TRICK_TYPE
        // --------------------------------------------------------

        require!(
            trick_type <= MAX_TRICK_TYPE,
            CustomError::InvalidTrickType
        );

        // --------------------------------------------------------
        // WRITE IMMUTABLE MOTION RECORD
        // --------------------------------------------------------

        event.device = registry.device_pubkey;
        event.user = ctx.accounts.user.key();
        event.trick_type = trick_type;
        event.g_force = g_force;
        event.timestamp = timestamp;
        event.sequence = sequence;
        event.bump = ctx.bumps.motion_event;

        // --------------------------------------------------------
        // UPDATE DEVICE STATE
        // --------------------------------------------------------

        registry.sequence = registry
            .sequence
            .checked_add(1)
            .ok_or(error!(CustomError::Overflow))?;

        registry.last_timestamp = timestamp;

        emit!(MotionRecorded {
            device: registry.device_pubkey,
            user: ctx.accounts.user.key(),
            trick_type,
            g_force,
            timestamp,
            sequence,
        });

        Ok(())
    }
}


// ================================================================
// CONSTANTS
// ================================================================

/// Maximum allowed difference between device timestamp and
/// Solana cluster time.
///
/// 60 seconds.
pub const MOTION_CLOCK_TOLERANCE: i64 = 60;

/// 15.00G represented in hundredths of G.
pub const MAX_G_FORCE: u32 = 1500;

/// Reserved trick types:
///
/// 0 = generic motion
/// 1..255 = application-defined trick identifiers
pub const MAX_TRICK_TYPE: u8 = 255;


// ================================================================
// ACCOUNTS
// ================================================================

#[derive(Accounts)]
#[instruction(device_pubkey: Pubkey)]
pub struct RegisterDevice<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        init,
        payer = owner,
        space = 8 + DeviceRegistry::INIT_SPACE,
        seeds = [
            b"device",
            device_pubkey.as_ref()
        ],
        bump
    )]
    pub device_registry: Account<'info, DeviceRegistry>,

    pub system_program: Program<'info, System>,
}


#[derive(Accounts)]
pub struct SetDeviceStatus<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        mut,
        seeds = [
            b"device",
            device_registry.device_pubkey.as_ref()
        ],
        bump = device_registry.bump,
        has_one = owner @ CustomError::UnauthorizedOwner
    )]
    pub device_registry: Account<'info, DeviceRegistry>,
}


#[derive(Accounts)]
pub struct TransferDevice<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        mut,
        seeds = [
            b"device",
            device_registry.device_pubkey.as_ref()
        ],
        bump = device_registry.bump,
        has_one = owner @ CustomError::UnauthorizedOwner
    )]
    pub device_registry: Account<'info, DeviceRegistry>,
}


#[derive(Accounts)]
pub struct RecordMotion<'info> {
    /// User must sign the transaction.
    #[account(mut)]
    pub user: Signer<'info>,

    /// Physical device must ALSO sign the transaction.
    ///
    /// This is the critical physical-device authentication layer.
    pub physical_device: Signer<'info>,

    #[account(
        mut,
        seeds = [
            b"device",
            device_registry.device_pubkey.as_ref()
        ],
        bump = device_registry.bump
    )]
    pub device_registry: Account<'info, DeviceRegistry>,

    /// One immutable account per device + sequence number.
    #[account(
        init,
        payer = user,
        space = 8 + MotionEvent::INIT_SPACE,
        seeds = [
            b"motion",
            device_registry.device_pubkey.as_ref(),
            &device_registry.sequence.to_le_bytes()
        ],
        bump
    )]
    pub motion_event: Account<'info, MotionEvent>,

    pub system_program: Program<'info, System>,
}


// ================================================================
// DEVICE REGISTRY
// ================================================================

#[account]
#[derive(InitSpace)]
pub struct DeviceRegistry {
    /// Current owner of the physical device.
    pub owner: Pubkey,

    /// Public key embedded/controlled by the physical device.
    pub device_pubkey: Pubkey,

    /// Whether the device is allowed to produce telemetry.
    pub is_active: bool,

    /// Next expected motion sequence.
    pub sequence: u64,

    /// Last accepted telemetry timestamp.
    pub last_timestamp: i64,

    /// PDA bump.
    pub bump: u8,
}


// ================================================================
// MOTION EVENT
// ================================================================

#[account]
#[derive(InitSpace)]
pub struct MotionEvent {
    /// Registered physical device.
    pub device: Pubkey,

    /// Human user associated with the device.
    pub user: Pubkey,

    /// Application-defined trick identifier.
    pub trick_type: u8,

    /// G-force in hundredths of G.
    ///
    /// Example:
    /// 845 = 8.45G
    pub g_force: u32,

    /// Device telemetry timestamp.
    pub timestamp: i64,

    /// Monotonic device event sequence.
    pub sequence: u64,

    /// PDA bump.
    pub bump: u8,
}


// ================================================================
// EVENTS
// ================================================================

#[event]
pub struct DeviceRegistered {
    pub device: Pubkey,
    pub owner: Pubkey,
}


#[event]
pub struct DeviceStatusChanged {
    pub device: Pubkey,
    pub active: bool,
}


#[event]
pub struct DeviceTransferred {
    pub device: Pubkey,
    pub previous_owner: Pubkey,
    pub new_owner: Pubkey,
}


#[event]
pub struct MotionRecorded {
    pub device: Pubkey,
    pub user: Pubkey,
    pub trick_type: u8,
    pub g_force: u32,
    pub timestamp: i64,
    pub sequence: u64,
}


// ================================================================
// ERRORS
// ================================================================

#[error_code]
pub enum CustomError {
    #[msg("Device is inactive.")]
    DeviceInactive,

    #[msg("Unauthorized user for this device.")]
    UnauthorizedUser,

    #[msg("Unauthorized device owner.")]
    UnauthorizedOwner,

    #[msg("Invalid physical device signature.")]
    InvalidDevice,

    #[msg("Invalid or out-of-sync timestamp.")]
    InvalidTimestamp,

    #[msg("Timestamp is not increasing.")]
    TimestampNotIncreasing,

    #[msg("Invalid motion sequence.")]
    InvalidSequence,

    #[msg("Physical limits exceeded.")]
    PhysicsLimitExceeded,

    #[msg("Invalid trick type.")]
    InvalidTrickType,

    #[msg("Invalid device owner.")]
    InvalidOwner,

    #[msg("Timestamp arithmetic overflow.")]
    TimestampOverflow,

    #[msg("Arithmetic overflow.")]
    Overflow,
}
