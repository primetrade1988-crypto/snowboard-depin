use crate::constants::*;
use crate::crypto::{build_telemetry_message, verify_ed25519_signature};
use crate::errors::SnowboardDepinError;
use crate::state::{Device, DeviceStatus, GlobalConfig, TelemetryRecord, TelemetrySample};
use crate::universal_decoder;
use anchor_lang::prelude::*;
use anchor_lang::solana_program::sysvar::instructions::ID as IX_SYSVAR_ID;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};

pub fn initialize(ctx: Context<Initialize>, reward_per_meter: u64) -> Result<()> {
    let clock = Clock::get()?;
    let config = &mut ctx.accounts.global_config;
    config.admin = ctx.accounts.admin.key();
    config.fusion_mint = ctx.accounts.fusion_mint.key();
    config.treasury_vault = ctx.accounts.treasury_vault.key();
    config.fee_vault = ctx.accounts.fee_vault.key();
    config.stake_pool = Pubkey::default();
    config.reward_per_meter = if reward_per_meter == 0 {
        DEFAULT_REWARD_PER_METER
    } else {
        reward_per_meter
    };
    config.reward_per_drop_cm = DEFAULT_REWARD_PER_DROP_CM;
    config.reward_per_airtime_ms = DEFAULT_REWARD_PER_AIRTIME_MS;
    config.max_payout_per_report = DEFAULT_MAX_PAYOUT;
    config.max_epoch_emission = DEFAULT_MAX_EPOCH_EMISSION;
    config.epoch_slots = DEFAULT_EPOCH_SLOTS;
    config.total_devices = 0;
    config.total_rewards_distributed = 0;
    config.genesis_slot = clock.slot;
    config.bump = ctx.bumps.global_config;
    config.treasury_bump = ctx.bumps.treasury_vault;
    config.fee_vault_bump = ctx.bumps.fee_vault;
    Ok(())
}

pub fn register_device(
    ctx: Context<RegisterDevice>,
    device_id: String,
    device_pubkey: [u8; 32],
) -> Result<()> {
    require!(
        !device_id.is_empty() && device_id.len() <= MAX_DEVICE_ID_LEN,
        SnowboardDepinError::InvalidDeviceId
    );

    let device = &mut ctx.accounts.device;
    device.owner = ctx.accounts.owner.key();
    device.device_pubkey = device_pubkey;
    device.device_id = device_id;
    device.status = DeviceStatus::Active;
    device.last_nonce = 0;
    device.last_timestamp = 0;
    device.processing_lock = 0;
    device.bump = ctx.bumps.device;

    let config = &mut ctx.accounts.global_config;
    config.total_devices = config
        .total_devices
        .checked_add(1)
        .ok_or(SnowboardDepinError::MathOverflow)?;
    Ok(())
}

pub fn submit_telemetry(
    ctx: Context<SubmitTelemetry>,
    sample: TelemetrySample,
    ed25519_ix_index: u8,
) -> Result<()> {
    require!(
        ctx.accounts.device.status == DeviceStatus::Active,
        SnowboardDepinError::DeviceInactive
    );
    require!(
        ctx.accounts.device.processing_lock == 0,
        SnowboardDepinError::Reentrancy
    );

    apply_anti_fraud(&ctx.accounts.device, &sample)?;

    let message = build_telemetry_message(ctx.accounts.device.key().as_ref(), &sample);
    verify_ed25519_signature(
        &ctx.accounts.instructions_sysvar,
        ed25519_ix_index,
        &ctx.accounts.device.device_pubkey,
        &message,
    )?;

    let clock = Clock::get()?;
    let epoch_id = clock.slot / ctx.accounts.global_config.epoch_slots.max(1);
    rotate_epoch(&mut ctx.accounts.device, epoch_id);

    let mut reward = compute_reward(&ctx.accounts.global_config, &ctx.accounts.device, &sample)?;
    reward = reward.min(ctx.accounts.global_config.max_payout_per_report);

    let next_epoch_emitted = ctx
        .accounts
        .device
        .epoch_emitted
        .checked_add(reward)
        .ok_or(SnowboardDepinError::MathOverflow)?;
    require!(
        next_epoch_emitted <= ctx.accounts.global_config.max_epoch_emission,
        SnowboardDepinError::RateLimitReached
    );

    require!(
        ctx.accounts.treasury_vault.amount >= reward,
        SnowboardDepinError::InsufficientTreasury
    );

    // Effects before CPI (re-entrancy / double-spend guard).
    let device = &mut ctx.accounts.device;
    device.processing_lock = 1;
    device.last_nonce = sample.nonce;
    device.last_timestamp = sample.timestamp;
    device.last_lat_e7 = sample.lat_e7;
    device.last_lon_e7 = sample.lon_e7;
    device.last_speed_cm_s = sample.speed_cm_s;
    device.last_accel_milli_g = sample.accel_milli_g;
    device.epoch_emitted = next_epoch_emitted;
    let dist_m = (sample.distance_cm as u64) / 100;
    device.epoch_distance_m = device
        .epoch_distance_m
        .checked_add(dist_m)
        .ok_or(SnowboardDepinError::MathOverflow)?;
    device.total_distance_m = device
        .total_distance_m
        .checked_add(dist_m)
        .ok_or(SnowboardDepinError::MathOverflow)?;
    device.total_rewards = device
        .total_rewards
        .checked_add(reward)
        .ok_or(SnowboardDepinError::MathOverflow)?;

    let telemetry = &mut ctx.accounts.telemetry_record;
    telemetry.device = device.key();
    telemetry.nonce = sample.nonce;
    telemetry.timestamp = sample.timestamp;
    telemetry.distance_meters = dist_m as u32;
    telemetry.vertical_drop_cm = sample.vertical_drop_cm;
    telemetry.airtime_ms = sample.airtime_ms;
    telemetry.reward_amount = reward;
    telemetry.bump = ctx.bumps.telemetry_record;

    ctx.accounts.global_config.total_rewards_distributed = ctx
        .accounts
        .global_config
        .total_rewards_distributed
        .checked_add(reward)
        .ok_or(SnowboardDepinError::MathOverflow)?;

    if reward > 0 {
        let bump = ctx.accounts.global_config.bump;
        let seeds: &[&[u8]] = &[CONFIG_SEED, &[bump]];
        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.treasury_vault.to_account_info(),
                    to: ctx.accounts.recipient_fusion.to_account_info(),
                    authority: ctx.accounts.global_config.to_account_info(),
                },
                &[seeds],
            ),
            reward,
        )?;
    }

    ctx.accounts.device.processing_lock = 0;
    Ok(())
}

pub fn submit_decoded_telemetry(
    ctx: Context<SubmitDecodedTelemetry>,
    nonce: u64,
    payload: Vec<u8>,
    ed25519_ix_index: u8,
) -> Result<()> {
    require!(
        ctx.accounts.adapter.active,
        SnowboardDepinError::UnknownAdapter
    );
    let sample = universal_decoder::normalize(&ctx.accounts.adapter, &payload)?;
    require!(sample.nonce == nonce, SnowboardDepinError::InvalidNonce);

    let mut raw_msg = b"SNOWBOARD_DEPIN_RAW".to_vec();
    raw_msg.extend_from_slice(ctx.accounts.device.key().as_ref());
    raw_msg.extend_from_slice(&payload);
    verify_ed25519_signature(
        &ctx.accounts.instructions_sysvar,
        ed25519_ix_index,
        &ctx.accounts.device.device_pubkey,
        &raw_msg,
    )?;

    inner_settle(ctx, sample)
}

fn inner_settle(ctx: Context<SubmitDecodedTelemetry>, sample: TelemetrySample) -> Result<()> {
    require!(
        ctx.accounts.device.status == DeviceStatus::Active,
        SnowboardDepinError::DeviceInactive
    );
    apply_anti_fraud(&ctx.accounts.device, &sample)?;
    let clock = Clock::get()?;
    let epoch_id = clock.slot / ctx.accounts.global_config.epoch_slots.max(1);
    rotate_epoch(&mut ctx.accounts.device, epoch_id);
    let mut reward = compute_reward(&ctx.accounts.global_config, &ctx.accounts.device, &sample)?;
    reward = reward.min(ctx.accounts.global_config.max_payout_per_report);
    require!(
        ctx.accounts
            .device
            .epoch_emitted
            .checked_add(reward)
            .ok_or(SnowboardDepinError::MathOverflow)?
            <= ctx.accounts.global_config.max_epoch_emission,
        SnowboardDepinError::RateLimitReached
    );
    require!(
        ctx.accounts.treasury_vault.amount >= reward,
        SnowboardDepinError::InsufficientTreasury
    );

    let device = &mut ctx.accounts.device;
    device.last_nonce = sample.nonce;
    device.last_timestamp = sample.timestamp;
    device.last_lat_e7 = sample.lat_e7;
    device.last_lon_e7 = sample.lon_e7;
    device.last_speed_cm_s = sample.speed_cm_s;
    device.epoch_emitted = device
        .epoch_emitted
        .checked_add(reward)
        .ok_or(SnowboardDepinError::MathOverflow)?;
    let dist_m = (sample.distance_cm as u64) / 100;
    device.total_distance_m = device
        .total_distance_m
        .checked_add(dist_m)
        .ok_or(SnowboardDepinError::MathOverflow)?;
    device.total_rewards = device
        .total_rewards
        .checked_add(reward)
        .ok_or(SnowboardDepinError::MathOverflow)?;

    let rec = &mut ctx.accounts.telemetry_record;
    rec.device = device.key();
    rec.nonce = sample.nonce;
    rec.timestamp = sample.timestamp;
    rec.distance_meters = dist_m as u32;
    rec.vertical_drop_cm = sample.vertical_drop_cm;
    rec.airtime_ms = sample.airtime_ms;
    rec.reward_amount = reward;
    rec.bump = ctx.bumps.telemetry_record;

    ctx.accounts.global_config.total_rewards_distributed = ctx
        .accounts
        .global_config
        .total_rewards_distributed
        .checked_add(reward)
        .ok_or(SnowboardDepinError::MathOverflow)?;

    if reward > 0 {
        let bump = ctx.accounts.global_config.bump;
        let seeds: &[&[u8]] = &[CONFIG_SEED, &[bump]];
        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.treasury_vault.to_account_info(),
                    to: ctx.accounts.recipient_fusion.to_account_info(),
                    authority: ctx.accounts.global_config.to_account_info(),
                },
                &[seeds],
            ),
            reward,
        )?;
    }
    Ok(())
}

pub fn update_reward_rate(ctx: Context<UpdateConfig>, reward_per_meter: u64) -> Result<()> {
    require!(reward_per_meter > 0, SnowboardDepinError::InvalidRewardRate);
    ctx.accounts.global_config.reward_per_meter = reward_per_meter;
    Ok(())
}

pub fn deactivate_device(ctx: Context<DeactivateDevice>) -> Result<()> {
    ctx.accounts.device.status = DeviceStatus::Inactive;
    Ok(())
}

pub fn apply_anti_fraud(device: &Device, sample: &TelemetrySample) -> Result<()> {
    require!(sample.nonce > device.last_nonce, SnowboardDepinError::ReplayDetected);
    require!(
        sample.distance_cm > 0 && sample.distance_cm <= MAX_DISTANCE_CM,
        SnowboardDepinError::InvalidDistance
    );
    require!(
        sample.speed_cm_s <= MAX_SPEED_CM_S,
        SnowboardDepinError::SpeedThresholdExceeded
    );
    require!(
        sample.accel_milli_g <= MAX_ACCEL_MILLI_G,
        SnowboardDepinError::AccelerationCapExceeded
    );
    require!(
        sample.imu_delta <= MAX_IMU_DELTA,
        SnowboardDepinError::ImuDeltaThresholdExceeded
    );

    let clock = Clock::get()?;
    require!(
        sample.timestamp <= clock.unix_timestamp + 30,
        SnowboardDepinError::InvalidTimestamp
    );

    if device.last_timestamp > 0 {
        require!(
            sample.timestamp > device.last_timestamp,
            SnowboardDepinError::ReplayDetected
        );
        let dt = sample
            .timestamp
            .checked_sub(device.last_timestamp)
            .ok_or(SnowboardDepinError::InvalidTimestamp)?;
        require!(dt <= MAX_SAMPLE_DT_SECS, SnowboardDepinError::InvalidTimestamp);

        let dlat = (sample.lat_e7 as i64) - (device.last_lat_e7 as i64);
        let dlon = (sample.lon_e7 as i64) - (device.last_lon_e7 as i64);
        // ~1.11 cm per 1e-7 deg of latitude; longitude scaled the same as conservative bound.
        let approx_cm = dlat
            .unsigned_abs()
            .saturating_add(dlon.unsigned_abs())
            .saturating_mul(111)
            / 100;
        let max_travel = (sample.speed_cm_s as u64)
            .saturating_add(device.last_speed_cm_s as u64)
            .saturating_mul(dt as u64)
            / 2
            + 5_000;
        require!(
            approx_cm <= max_travel,
            SnowboardDepinError::SpatialContinuityFailed
        );
    }
    Ok(())
}

pub fn compute_reward(
    config: &GlobalConfig,
    device: &Device,
    sample: &TelemetrySample,
) -> Result<u64> {
    let dist_m = (sample.distance_cm as u64) / 100;
    let distance_part = dist_m
        .checked_mul(config.reward_per_meter)
        .ok_or(SnowboardDepinError::MathOverflow)?;
    let drop_part = (sample.vertical_drop_cm as u64)
        .checked_mul(config.reward_per_drop_cm)
        .ok_or(SnowboardDepinError::MathOverflow)?;
    let air_part = (sample.airtime_ms as u64)
        .checked_mul(config.reward_per_airtime_ms)
        .ok_or(SnowboardDepinError::MathOverflow)?;
    let raw = distance_part
        .checked_add(drop_part)
        .and_then(|v| v.checked_add(air_part))
        .ok_or(SnowboardDepinError::MathOverflow)?;

    // Diminishing returns vs epoch distance: raw * K / (K + epoch_distance_m)
    let denom = DIMINISH_K
        .checked_add(device.epoch_distance_m)
        .ok_or(SnowboardDepinError::MathOverflow)?;
    raw.checked_mul(DIMINISH_K)
        .and_then(|v| v.checked_div(denom))
        .ok_or(error!(SnowboardDepinError::MathOverflow))
}

fn rotate_epoch(device: &mut Device, epoch_id: u64) {
    if device.epoch_id != epoch_id {
        device.epoch_id = epoch_id;
        device.epoch_emitted = 0;
        device.epoch_distance_m = 0;
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    pub fusion_mint: Account<'info, Mint>,
    #[account(
        init,
        payer = admin,
        space = 8 + GlobalConfig::INIT_SPACE,
        seeds = [CONFIG_SEED],
        bump,
    )]
    pub global_config: Account<'info, GlobalConfig>,
    #[account(
        init,
        payer = admin,
        seeds = [TREASURY_SEED, global_config.key().as_ref()],
        bump,
        token::mint = fusion_mint,
        token::authority = global_config,
    )]
    pub treasury_vault: Account<'info, TokenAccount>,
    #[account(
        init,
        payer = admin,
        seeds = [FEE_VAULT_SEED, global_config.key().as_ref()],
        bump,
        token::mint = fusion_mint,
        token::authority = global_config,
    )]
    pub fee_vault: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
#[instruction(device_id: String)]
pub struct RegisterDevice<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(mut, seeds = [CONFIG_SEED], bump = global_config.bump)]
    pub global_config: Account<'info, GlobalConfig>,
    #[account(
        init,
        payer = owner,
        space = 8 + Device::INIT_SPACE,
        seeds = [DEVICE_SEED, owner.key().as_ref(), device_id.as_bytes()],
        bump,
    )]
    pub device: Account<'info, Device>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(sample: TelemetrySample)]
pub struct SubmitTelemetry<'info> {
    #[account(
        mut,
        seeds = [CONFIG_SEED],
        bump = global_config.bump,
        has_one = fusion_mint,
        has_one = treasury_vault,
    )]
    pub global_config: Account<'info, GlobalConfig>,
    pub fusion_mint: Account<'info, Mint>,
    #[account(
        mut,
        seeds = [TREASURY_SEED, global_config.key().as_ref()],
        bump = global_config.treasury_bump,
    )]
    pub treasury_vault: Account<'info, TokenAccount>,
    #[account(
        mut,
        seeds = [DEVICE_SEED, device.owner.as_ref(), device.device_id.as_bytes()],
        bump = device.bump,
        constraint = device.status == DeviceStatus::Active @ SnowboardDepinError::UnauthorizedDevice,
    )]
    pub device: Account<'info, Device>,
    #[account(
        init,
        payer = payer,
        space = 8 + TelemetryRecord::INIT_SPACE,
        seeds = [TELEMETRY_SEED, device.key().as_ref(), &sample.nonce.to_le_bytes()],
        bump,
    )]
    pub telemetry_record: Account<'info, TelemetryRecord>,
    #[account(
        mut,
        constraint = recipient_fusion.owner == device.owner @ SnowboardDepinError::InvalidRecipient,
        constraint = recipient_fusion.mint == fusion_mint.key() @ SnowboardDepinError::InvalidMint,
    )]
    pub recipient_fusion: Account<'info, TokenAccount>,
    #[account(mut)]
    pub payer: Signer<'info>,
    /// CHECK: Ed25519 introspection sysvar.
    #[account(address = IX_SYSVAR_ID)]
    pub instructions_sysvar: AccountInfo<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(nonce: u64)]
pub struct SubmitDecodedTelemetry<'info> {
    #[account(
        mut,
        seeds = [CONFIG_SEED],
        bump = global_config.bump,
        has_one = fusion_mint,
        has_one = treasury_vault,
    )]
    pub global_config: Account<'info, GlobalConfig>,
    pub fusion_mint: Account<'info, Mint>,
    #[account(
        mut,
        seeds = [TREASURY_SEED, global_config.key().as_ref()],
        bump = global_config.treasury_bump,
    )]
    pub treasury_vault: Account<'info, TokenAccount>,
    #[account(
        mut,
        seeds = [DEVICE_SEED, device.owner.as_ref(), device.device_id.as_bytes()],
        bump = device.bump,
        constraint = device.status == DeviceStatus::Active @ SnowboardDepinError::UnauthorizedDevice,
    )]
    pub device: Account<'info, Device>,
    #[account(
        seeds = [ADAPTER_SEED, &adapter.brand_id.to_le_bytes()],
        bump = adapter.bump,
    )]
    pub adapter: Account<'info, crate::state::HardwareAdapter>,
    #[account(
        init,
        payer = payer,
        space = 8 + TelemetryRecord::INIT_SPACE,
        seeds = [TELEMETRY_SEED, device.key().as_ref(), &nonce.to_le_bytes()],
        bump,
    )]
    pub telemetry_record: Account<'info, TelemetryRecord>,
    #[account(
        mut,
        constraint = recipient_fusion.owner == device.owner @ SnowboardDepinError::InvalidRecipient,
        constraint = recipient_fusion.mint == fusion_mint.key() @ SnowboardDepinError::InvalidMint,
    )]
    pub recipient_fusion: Account<'info, TokenAccount>,
    #[account(mut)]
    pub payer: Signer<'info>,
    /// CHECK: Ed25519 introspection sysvar.
    #[account(address = IX_SYSVAR_ID)]
    pub instructions_sysvar: AccountInfo<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdateConfig<'info> {
    pub admin: Signer<'info>,
    #[account(
        mut,
        seeds = [CONFIG_SEED],
        bump = global_config.bump,
        has_one = admin,
    )]
    pub global_config: Account<'info, GlobalConfig>,
}

#[derive(Accounts)]
pub struct DeactivateDevice<'info> {
    pub owner: Signer<'info>,
    #[account(
        mut,
        seeds = [DEVICE_SEED, device.owner.as_ref(), device.device_id.as_bytes()],
        bump = device.bump,
        has_one = owner,
    )]
    pub device: Account<'info, Device>,
}
