```rust
use anchor_lang::prelude::*;

use anchor_lang::solana_program::{
    ed25519_program,
    sysvar::instructions as sysvar_instructions,
};

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

/// MIII Protocol
///
/// Core principle:
///
/// Physical Device
///     -> Device Private Key
///     -> Ed25519 Signed Motion Payload
///     -> Solana Ed25519 Precompile
///     -> MIII Protocol Validation
///     -> Immutable Motion Record
///
/// Tokenomics are intentionally kept outside this core layer.
/// The physical verification layer remains useful independently
/// from token price/speculation.

// ================================================================
// PROGRAM
// ================================================================

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
        require!(
            device_pubkey != Pubkey::default(),
            CustomError::InvalidDevice
        );

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
    //
    // Transaction layout MUST be:
    //
    // [0] Ed25519 precompile instruction
    // [1] MIII::record_motion
    //
    // The Ed25519 instruction signs the exact motion payload.
    // MIII verifies that the Ed25519 instruction:
    //
    // 1. Uses the Solana Ed25519 precompile.
    // 2. References the registered device public key.
    // 3. References the exact motion payload expected by MIII.
    //
    // The Ed25519 precompile itself performs the cryptographic
    // signature verification before MIII executes.
    //
    // The device DOES NOT need to sign the Solana transaction.
    // It only signs the motion data.
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
        // SEQUENCE CHECK
        //
        // The event must use exactly the next expected sequence.
        // This prevents replaying an already accepted motion.
        // --------------------------------------------------------

        require!(
            sequence == registry.sequence,
            CustomError::InvalidSequence
        );

        // --------------------------------------------------------
        // TIMESTAMP VALIDATION
        //
        // Event cannot be more than 60 seconds old or
        // 60 seconds in the future relative to Solana time.
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
        // 845  == 8.45G
        // 1500 == 15.00G
        // --------------------------------------------------------

        require!(
            g_force <= MAX_G_FORCE,
            CustomError::PhysicsLimitExceeded
        );

        // --------------------------------------------------------
        // TRICK VALIDATION
        // --------------------------------------------------------

        require!(
            trick_type <= MAX_TRICK_TYPE,
            CustomError::InvalidTrickType
        );

        // --------------------------------------------------------
        // BUILD EXACT SIGNED MESSAGE
        //
        // The physical device signs this exact byte sequence:
        //
        // DOMAIN
        // DEVICE PUBKEY
        // SEQUENCE
        // TIMESTAMP
        // TRICK TYPE
        // G-FORCE
        //
        // Including the device public key and domain prevents
        // accidental cross-context signature reuse.
        // --------------------------------------------------------

        let message = build_motion_message(
            &registry.device_pubkey,
            sequence,
            timestamp,
            trick_type,
            g_force,
        );

        // --------------------------------------------------------
        // VERIFY ED25519 PRECOMPILE INSTRUCTION
        //
        // We intentionally inspect the instruction immediately
        // before record_motion using relative instruction lookup.
        //
        // This avoids hard-coding an absolute transaction index.
        // --------------------------------------------------------

        verify_ed25519_signature(
            &ctx.accounts.instructions_sysvar.to_account_info(),
            &registry.device_pubkey,
            &message,
        )?;

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

        // --------------------------------------------------------
        // EVENT
        // --------------------------------------------------------

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

/// Domain separator / protocol version for signed motion data.
pub const MOTION_DOMAIN: &[u8] = b"MIII_MOTION_V1";

/// Maximum allowed difference between device timestamp and
/// Solana cluster time.
///
/// 60 seconds.
pub const MOTION_CLOCK_TOLERANCE: i64 = 60;

/// 15.00G represented in hundredths of G.
pub const MAX_G_FORCE: u32 = 1500;

/// Application-defined trick identifiers.
///
/// 0   = generic motion
/// 1..255 = application-defined trick identifiers
pub const MAX_TRICK_TYPE: u8 = 255;


// ================================================================
// SIGNED MOTION MESSAGE
// ================================================================
//
// Exact format:
//
// [MOTION_DOMAIN]
// [device_pubkey: 32 bytes]
// [sequence: 8 bytes LE]
// [timestamp: 8 bytes LE]
// [trick_type: 1 byte]
// [g_force: 4 bytes LE]
//
// The same serialization MUST be implemented by the device
// firmware / SDK that creates the Ed25519 signature.
// ================================================================

fn build_motion_message(
    device_pubkey: &Pubkey,
    sequence: u64,
    timestamp: i64,
    trick_type: u8,
    g_force: u32,
) -> Vec<u8> {
    let mut message = Vec::with_capacity(
        MOTION_DOMAIN.len()
            + 32
            + 8
            + 8
            + 1
            + 4,
    );

    message.extend_from_slice(MOTION_DOMAIN);
    message.extend_from_slice(device_pubkey.as_ref());
    message.extend_from_slice(&sequence.to_le_bytes());
    message.extend_from_slice(&timestamp.to_le_bytes());
    message.push(trick_type);
    message.extend_from_slice(&g_force.to_le_bytes());

    message
}


// ================================================================
// ED25519 VERIFICATION
// ================================================================
//
// The Solana Ed25519 precompile validates the actual signature.
//
// This function validates that the previous top-level instruction:
//
// 1. Is the Ed25519 precompile.
// 2. Uses the registered device public key.
// 3. References the exact expected message.
//
// Ed25519 precompiles cannot be called through CPI.
// Instead, MIII inspects the top-level Ed25519 instruction
// through the Instructions Sysvar.
// ================================================================

fn verify_ed25519_signature(
    instructions_sysvar: &AccountInfo,
    expected_pubkey: &Pubkey,
    expected_message: &[u8],
) -> Result<()> {
    // ------------------------------------------------------------
    // Get the instruction immediately before record_motion.
    // ------------------------------------------------------------

    let ed25519_instruction =
        sysvar_instructions::get_instruction_relative(
            -1,
            instructions_sysvar,
        )
        .map_err(|_| error!(CustomError::MissingEd25519Instruction))?;

    // ------------------------------------------------------------
    // Verify program ID.
    // ------------------------------------------------------------

    require!(
        ed25519_instruction.program_id == ed25519_program::id(),
        CustomError::InvalidEd25519Instruction
    );

    let data = ed25519_instruction.data;

    // ------------------------------------------------------------
    // Minimum size:
    //
    // 1 byte  = signature count
    // 1 byte  = padding
    // 14 bytes = Ed25519SignatureOffsets
    //
    // Total = 16 bytes.
    // ------------------------------------------------------------

    require!(
        data.len() >= 16,
        CustomError::InvalidEd25519Instruction
    );

    // ------------------------------------------------------------
    // Exactly one signature.
    // ------------------------------------------------------------

    let num_signatures = data[0];

    require!(
        num_signatures == 1,
        CustomError::InvalidEd25519Instruction
    );

    // Padding must be zero.
    require!(
        data[1] == 0,
        CustomError::InvalidEd25519Instruction
    );

    // ------------------------------------------------------------
    // Parse Ed25519SignatureOffsets.
    // ------------------------------------------------------------

    let signature_offset =
        read_u16_le(&data, 2)?;

    let signature_instruction_index =
        read_u16_le(&data, 4)?;

    let public_key_offset =
        read_u16_le(&data, 6)?;

    let public_key_instruction_index =
        read_u16_le(&data, 8)?;

    let message_data_offset =
        read_u16_le(&data, 10)?;

    let message_data_size =
        read_u16_le(&data, 12)?;

    let message_instruction_index =
        read_u16_le(&data, 14)?;

    // ------------------------------------------------------------
    // We require all referenced data to live inside THIS
    // Ed25519 instruction.
    //
    // 0xFFFF means "current instruction" for the Ed25519
    // precompile.
    // ------------------------------------------------------------

    const CURRENT_INSTRUCTION: u16 = u16::MAX;

    require!(
        signature_instruction_index == CURRENT_INSTRUCTION,
        CustomError::InvalidEd25519Instruction
    );

    require!(
        public_key_instruction_index == CURRENT_INSTRUCTION,
        CustomError::InvalidEd25519Instruction
    );

    require!(
        message_instruction_index == CURRENT_INSTRUCTION,
        CustomError::InvalidEd25519Instruction
    );

    // ------------------------------------------------------------
    // Signature must be a valid 64-byte region.
    //
    // We don't manually verify the signature here.
    // The Ed25519 precompile already performed that verification
    // before this instruction executes.
    // ------------------------------------------------------------

    let signature_end = signature_offset
        .checked_add(64)
        .ok_or(error!(CustomError::InvalidEd25519Instruction))?;

    require!(
        signature_end <= data.len(),
        CustomError::InvalidEd25519Instruction
    );

    // ------------------------------------------------------------
    // Extract public key.
    // ------------------------------------------------------------

    let public_key_end = public_key_offset
        .checked_add(32)
        .ok_or(error!(CustomError::InvalidEd25519Instruction))?;

    require!(
        public_key_end <= data.len(),
        CustomError::InvalidEd25519Instruction
    );

    let public_key_bytes =
        &data[public_key_offset..public_key_end];

    require!(
        public_key_bytes == expected_pubkey.as_ref(),
        CustomError::InvalidDeviceSignature
    );

    // ------------------------------------------------------------
    // Extract signed message.
    // ------------------------------------------------------------

    let message_end = message_data_offset
        .checked_add(message_data_size as usize)
        .ok_or(error!(CustomError::InvalidEd25519Instruction))?;

    require!(
        message_end <= data.len(),
        CustomError::InvalidEd25519Instruction
    );

    let signed_message =
        &data[message_data_offset..message_end];

    // ------------------------------------------------------------
    // Exact byte-for-byte message match.
    // ------------------------------------------------------------

    require!(
        signed_message == expected_message,
        CustomError::InvalidSignedMessage
    );

    Ok(())
}


// ================================================================
// SAFE LITTLE-ENDIAN READER
// ================================================================

fn read_u16_le(data: &[u8], offset: usize) -> Result<u16> {
    let end = offset
        .checked_add(2)
        .ok_or(error!(CustomError::InvalidEd25519Instruction))?;

    require!(
        end <= data.len(),
        CustomError::InvalidEd25519Instruction
    );

    Ok(u16::from_le_bytes([
        data[offset],
        data[offset + 1],
    ]))
}


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
    /// User / relayer who pays for and submits the transaction.
    ///
    /// The physical device does NOT need to be online
    /// and does NOT need to sign the Solana transaction.
    #[account(mut)]
    pub user: Signer<'info>,

    /// Instructions Sysvar containing all top-level instructions
    /// in the current transaction.
    ///
    /// Used to inspect the Ed25519 precompile instruction
    /// immediately preceding record_motion.
    /// CHECK: Address is constrained to the Instructions Sysvar.
    #[account(address = sysvar_instructions::ID)]
    pub instructions_sysvar: UncheckedAccount<'info>,

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

    /// Ed25519 public key controlled by the physical device.
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

    #[msg("Invalid physical device.")]
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

    #[msg("Ed25519 instruction is missing.")]
    MissingEd25519Instruction,

    #[msg("Invalid Ed25519 instruction format.")]
    InvalidEd25519Instruction,

    #[msg("Ed25519 public key does not match the registered device.")]
    InvalidDeviceSignature,

    #[msg("Signed motion message does not match the submitted motion.")]
    InvalidSignedMessage,
}
```
