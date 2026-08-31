use crate::constants::*;
use crate::errors::SnowboardDepinError;
use crate::state::{GlobalConfig, HardwareAdapter, PayloadFormat};
use anchor_lang::prelude::*;

pub fn register_adapter(
    ctx: Context<RegisterAdapter>,
    brand_id: u16,
    format: PayloadFormat,
    distance_scale: u32,
    speed_scale: u32,
    drop_scale: u32,
    airtime_scale: u32,
) -> Result<()> {
    require!(
        distance_scale > 0 && speed_scale > 0 && drop_scale > 0 && airtime_scale > 0,
        SnowboardDepinError::InvalidPayload
    );
    let adapter = &mut ctx.accounts.adapter;
    adapter.admin = ctx.accounts.admin.key();
    adapter.brand_id = brand_id;
    adapter.format = format;
    adapter.distance_scale = distance_scale;
    adapter.speed_scale = speed_scale;
    adapter.drop_scale = drop_scale;
    adapter.airtime_scale = airtime_scale;
    adapter.active = true;
    adapter.bump = ctx.bumps.adapter;
    Ok(())
}

#[derive(Accounts)]
#[instruction(brand_id: u16)]
pub struct RegisterAdapter<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(
        seeds = [CONFIG_SEED],
        bump = global_config.bump,
        has_one = admin,
    )]
    pub global_config: Account<'info, GlobalConfig>,
    #[account(
        init,
        payer = admin,
        space = 8 + HardwareAdapter::INIT_SPACE,
        seeds = [ADAPTER_SEED, &brand_id.to_le_bytes()],
        bump,
    )]
    pub adapter: Account<'info, HardwareAdapter>,
    pub system_program: Program<'info, System>,
}
