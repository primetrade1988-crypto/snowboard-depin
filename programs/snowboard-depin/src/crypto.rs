use anchor_lang::prelude::*;
use anchor_lang::solana_program::ed25519_program;
use anchor_lang::solana_program::sysvar::instructions::load_instruction_at_checked;

use crate::errors::SnowboardDepinError;
use crate::state::TelemetrySample;

pub const TELEM_PREFIX: &[u8] = b"SNOWBOARD_DEPIN_TELEM";
pub const MOTION_PREFIX: &[u8] = b"SNOWBOARD_DEPIN_MOTION";

pub fn build_telemetry_message(device: &[u8], sample: &TelemetrySample) -> Vec<u8> {
    let mut msg = TELEM_PREFIX.to_vec();
    msg.extend_from_slice(device);
    msg.extend_from_slice(&sample.nonce.to_le_bytes());
    msg.extend_from_slice(&sample.timestamp.to_le_bytes());
    msg.extend_from_slice(&sample.lat_e7.to_le_bytes());
    msg.extend_from_slice(&sample.lon_e7.to_le_bytes());
    msg.extend_from_slice(&sample.altitude_cm.to_le_bytes());
    msg.extend_from_slice(&sample.speed_cm_s.to_le_bytes());
    msg.extend_from_slice(&sample.accel_milli_g.to_le_bytes());
    msg.extend_from_slice(&sample.imu_delta.to_le_bytes());
    msg.extend_from_slice(&sample.distance_cm.to_le_bytes());
    msg.extend_from_slice(&sample.vertical_drop_cm.to_le_bytes());
    msg.extend_from_slice(&sample.airtime_ms.to_le_bytes());
    msg
}

pub fn build_motion_message(device: &[u8], proof: &crate::state::MotionProof) -> Vec<u8> {
    let mut msg = MOTION_PREFIX.to_vec();
    msg.extend_from_slice(device);
    msg.extend_from_slice(&proof.nonce.to_le_bytes());
    msg.extend_from_slice(&proof.timestamp.to_le_bytes());
    msg.extend_from_slice(&proof.trick_id.to_le_bytes());
    msg.extend_from_slice(&proof.airtime_ms.to_le_bytes());
    msg.extend_from_slice(&proof.rotation_deg.to_le_bytes());
    msg.push(proof.confidence);
    msg
}

/// Introspect Ed25519Program at `sig_ix_index` and match pubkey + canonical message.
pub fn verify_ed25519_signature(
    ix_sysvar: &AccountInfo,
    sig_ix_index: u8,
    expected_pubkey: &[u8; 32],
    expected_message: &[u8],
) -> Result<()> {
    let ix = load_instruction_at_checked(sig_ix_index as usize, ix_sysvar)
        .map_err(|_| error!(SnowboardDepinError::InvalidEd25519Instruction))?;

    require_keys_eq!(
        ix.program_id,
        ed25519_program::ID,
        SnowboardDepinError::InvalidSignature
    );
    require!(
        ix.data.len() >= 16,
        SnowboardDepinError::InvalidEd25519Instruction
    );

    let num_sigs = ix.data[0];
    require!(num_sigs == 1, SnowboardDepinError::InvalidEd25519Instruction);

    let pubkey_offset = u16::from_le_bytes([ix.data[6], ix.data[7]]) as usize;
    let msg_offset = u16::from_le_bytes([ix.data[10], ix.data[11]]) as usize;
    let msg_size = u16::from_le_bytes([ix.data[12], ix.data[13]]) as usize;

    require!(
        ix.data.len() >= pubkey_offset.saturating_add(32),
        SnowboardDepinError::InvalidEd25519Instruction
    );
    require!(
        ix.data.len() >= msg_offset.saturating_add(msg_size),
        SnowboardDepinError::InvalidEd25519Instruction
    );

    let pubkey = &ix.data[pubkey_offset..pubkey_offset + 32];
    require!(
        pubkey == expected_pubkey.as_slice(),
        SnowboardDepinError::Ed25519PubkeyMismatch
    );

    let message = &ix.data[msg_offset..msg_offset + msg_size];
    require!(
        message == expected_message,
        SnowboardDepinError::Ed25519MessageMismatch
    );

    Ok(())
}
