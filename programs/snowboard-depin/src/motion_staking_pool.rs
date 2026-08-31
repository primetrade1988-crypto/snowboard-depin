use crate::constants::*;
use crate::errors::SnowboardDepinError;
use crate::state::{GlobalConfig, StakePool, StakePosition, StakeTier};
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Burn, Mint, Token, TokenAccount, Transfer};

pub fn initialize_staking(ctx: Context<InitializeStaking>) -> Result<()> {
    let clock = Clock::get()?;
    let pool = &mut ctx.accounts.stake_pool;
    pool.admin = ctx.accounts.admin.key();
    pool.fusion_mint = ctx.accounts.fusion_mint.key();
    pool.stake_vault = ctx.accounts.stake_vault.key();
    pool.total_staked = 0;
    pool.reward_index = 0;
    pool.emission_per_slot = DEFAULT_EMISSION_PER_SLOT;
    pool.last_update_slot = clock.slot;
    pool.last_halving_slot = clock.slot;
    pool.halving_interval_slots = DEFAULT_HALVING_INTERVAL;
    pool.unbond_slots = DEFAULT_UNBOND_SLOTS;
    pool.burn_bps = DEFAULT_BURN_BPS;
    pool.staker_share_bps = DEFAULT_STAKER_SHARE_BPS;
    pool.bump = ctx.bumps.stake_pool;
    pool.vault_bump = ctx.bumps.stake_vault;

    ctx.accounts.global_config.stake_pool = pool.key();
    Ok(())
}

pub fn accrue(pool: &mut StakePool) -> Result<()> {
    let slot = Clock::get()?.slot;
    if slot <= pool.last_update_slot || pool.total_staked == 0 {
        pool.last_update_slot = slot.max(pool.last_update_slot);
        return Ok(());
    }
    let dt = slot.saturating_sub(pool.last_update_slot);
    let minted = dt
        .checked_mul(pool.emission_per_slot)
        .ok_or(SnowboardDepinError::MathOverflow)?;
    let delta = (minted as u128)
        .checked_mul(INDEX_SCALE)
        .and_then(|v| v.checked_div(pool.total_staked as u128))
        .ok_or(SnowboardDepinError::MathOverflow)?;
    pool.reward_index = pool
        .reward_index
        .checked_add(delta)
        .ok_or(SnowboardDepinError::MathOverflow)?;
    pool.last_update_slot = slot;
    Ok(())
}

fn pending_rewards(pool: &StakePool, pos: &StakePosition) -> Result<u64> {
    let accrued = (pos.amount as u128)
        .checked_mul(pool.reward_index.saturating_sub(pos.reward_debt))
        .and_then(|v| v.checked_div(INDEX_SCALE))
        .ok_or(SnowboardDepinError::MathOverflow)?;
    let boosted = accrued
        .checked_mul(pos.tier.multiplier_bps() as u128)
        .and_then(|v| v.checked_div(BPS_DENOM as u128))
        .ok_or(SnowboardDepinError::MathOverflow)?;
    u64::try_from(boosted).map_err(|_| error!(SnowboardDepinError::MathOverflow))
}

pub fn stake(ctx: Context<Stake>, amount: u64) -> Result<()> {
    require!(amount > 0, SnowboardDepinError::InsufficientStake);
    accrue(&mut ctx.accounts.stake_pool)?;

    let pos = &mut ctx.accounts.stake_position;
    if pos.owner == Pubkey::default() {
        pos.owner = ctx.accounts.owner.key();
        pos.bump = ctx.bumps.stake_position;
    }
    if pos.amount > 0 {
        let pending = pending_rewards(&ctx.accounts.stake_pool, pos)?;
        if pending > 0 && ctx.accounts.treasury_vault.amount >= pending {
            let bump = ctx.accounts.global_config.bump;
            let seeds: &[&[u8]] = &[CONFIG_SEED, &[bump]];
            token::transfer(
                CpiContext::new_with_signer(
                    ctx.accounts.token_program.to_account_info(),
                    Transfer {
                        from: ctx.accounts.treasury_vault.to_account_info(),
                        to: ctx.accounts.user_fusion.to_account_info(),
                        authority: ctx.accounts.global_config.to_account_info(),
                    },
                    &[seeds],
                ),
                pending,
            )?;
        }
    }

    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.user_fusion.to_account_info(),
                to: ctx.accounts.stake_vault.to_account_info(),
                authority: ctx.accounts.owner.to_account_info(),
            },
        ),
        amount,
    )?;

    pos.amount = pos
        .amount
        .checked_add(amount)
        .ok_or(SnowboardDepinError::MathOverflow)?;
    pos.tier = StakeTier::from_amount(pos.amount);
    pos.reward_debt = ctx.accounts.stake_pool.reward_index;

    ctx.accounts.stake_pool.total_staked = ctx
        .accounts
        .stake_pool
        .total_staked
        .checked_add(amount)
        .ok_or(SnowboardDepinError::MathOverflow)?;
    Ok(())
}

pub fn request_unstake(ctx: Context<RequestUnstake>, amount: u64) -> Result<()> {
    accrue(&mut ctx.accounts.stake_pool)?;
    let pos = &mut ctx.accounts.stake_position;
    require!(amount > 0 && amount <= pos.amount, SnowboardDepinError::InsufficientStake);
    require!(pos.pending_unbond == 0, SnowboardDepinError::CooldownActive);

    let pending = pending_rewards(&ctx.accounts.stake_pool, pos)?;
    pos.amount = pos.amount.checked_sub(amount).ok_or(SnowboardDepinError::MathOverflow)?;
    pos.pending_unbond = amount;
    pos.unbond_complete_slot = Clock::get()?
        .slot
        .checked_add(ctx.accounts.stake_pool.unbond_slots)
        .ok_or(SnowboardDepinError::MathOverflow)?;
    pos.tier = StakeTier::from_amount(pos.amount);
    pos.reward_debt = ctx.accounts.stake_pool.reward_index;

    ctx.accounts.stake_pool.total_staked = ctx
        .accounts
        .stake_pool
        .total_staked
        .checked_sub(amount)
        .ok_or(SnowboardDepinError::MathOverflow)?;

    if pending > 0 && ctx.accounts.treasury_vault.amount >= pending {
        let bump = ctx.accounts.global_config.bump;
        let seeds: &[&[u8]] = &[CONFIG_SEED, &[bump]];
        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.treasury_vault.to_account_info(),
                    to: ctx.accounts.user_fusion.to_account_info(),
                    authority: ctx.accounts.global_config.to_account_info(),
                },
                &[seeds],
            ),
            pending,
        )?;
    }
    Ok(())
}

pub fn complete_unstake(ctx: Context<CompleteUnstake>) -> Result<()> {
    let pos = &mut ctx.accounts.stake_position;
    require!(pos.pending_unbond > 0, SnowboardDepinError::NoUnbondPending);
    require!(
        Clock::get()?.slot >= pos.unbond_complete_slot,
        SnowboardDepinError::CooldownActive
    );
    let amount = pos.pending_unbond;
    pos.pending_unbond = 0;
    pos.unbond_complete_slot = 0;

    require!(
        ctx.accounts.stake_vault.amount >= amount,
        SnowboardDepinError::InsufficientStakeVault
    );

    let bump = ctx.accounts.stake_pool.bump;
    let seeds: &[&[u8]] = &[STAKE_POOL_SEED, &[bump]];
    token::transfer(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.stake_vault.to_account_info(),
                to: ctx.accounts.user_fusion.to_account_info(),
                authority: ctx.accounts.stake_pool.to_account_info(),
            },
            &[seeds],
        ),
        amount,
    )?;
    Ok(())
}

pub fn apply_halving(ctx: Context<ApplyHalving>) -> Result<()> {
    accrue(&mut ctx.accounts.stake_pool)?;
    let pool = &mut ctx.accounts.stake_pool;
    let slot = Clock::get()?.slot;
    require!(
        slot >= pool
            .last_halving_slot
            .checked_add(pool.halving_interval_slots)
            .ok_or(SnowboardDepinError::MathOverflow)?,
        SnowboardDepinError::HalvingNotDue
    );
    pool.emission_per_slot = pool.emission_per_slot.max(1) / 2;
    pool.last_halving_slot = slot;
    Ok(())
}

/// Split marketplace/hardware fees: staker share stays in treasury index via
/// increasing reward_index; burn_bps is burned from fee_vault.
pub fn ingest_protocol_fees(ctx: Context<IngestProtocolFees>, amount: u64) -> Result<()> {
    require!(amount > 0, SnowboardDepinError::ZeroFee);
    require!(
        ctx.accounts.fee_vault.amount >= amount,
        SnowboardDepinError::InsufficientTreasury
    );
    accrue(&mut ctx.accounts.stake_pool)?;

    let burn_amt = amount
        .checked_mul(ctx.accounts.stake_pool.burn_bps as u64)
        .and_then(|v| v.checked_div(BPS_DENOM))
        .ok_or(SnowboardDepinError::MathOverflow)?;
    let staker_amt = amount.saturating_sub(burn_amt);

    let cfg_bump = ctx.accounts.global_config.bump;
    let cfg_seeds: &[&[u8]] = &[CONFIG_SEED, &[cfg_bump]];

    if burn_amt > 0 {
        token::burn(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Burn {
                    mint: ctx.accounts.fusion_mint.to_account_info(),
                    from: ctx.accounts.fee_vault.to_account_info(),
                    authority: ctx.accounts.global_config.to_account_info(),
                },
                &[cfg_seeds],
            ),
            burn_amt,
        )?;
    }

    if staker_amt > 0 {
        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.fee_vault.to_account_info(),
                    to: ctx.accounts.treasury_vault.to_account_info(),
                    authority: ctx.accounts.global_config.to_account_info(),
                },
                &[cfg_seeds],
            ),
            staker_amt,
        )?;
        if ctx.accounts.stake_pool.total_staked > 0 {
            let delta = (staker_amt as u128)
                .checked_mul(INDEX_SCALE)
                .and_then(|v| v.checked_div(ctx.accounts.stake_pool.total_staked as u128))
                .ok_or(SnowboardDepinError::MathOverflow)?;
            ctx.accounts.stake_pool.reward_index = ctx
                .accounts
                .stake_pool
                .reward_index
                .checked_add(delta)
                .ok_or(SnowboardDepinError::MathOverflow)?;
        }
    }
    Ok(())
}

#[derive(Accounts)]
pub struct InitializeStaking<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    pub fusion_mint: Account<'info, Mint>,
    #[account(
        mut,
        seeds = [CONFIG_SEED],
        bump = global_config.bump,
        has_one = admin,
        has_one = fusion_mint,
    )]
    pub global_config: Account<'info, GlobalConfig>,
    #[account(
        init,
        payer = admin,
        space = 8 + StakePool::INIT_SPACE,
        seeds = [STAKE_POOL_SEED],
        bump,
    )]
    pub stake_pool: Account<'info, StakePool>,
    #[account(
        init,
        payer = admin,
        seeds = [STAKE_VAULT_SEED, stake_pool.key().as_ref()],
        bump,
        token::mint = fusion_mint,
        token::authority = stake_pool,
    )]
    pub stake_vault: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct Stake<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(
        seeds = [CONFIG_SEED],
        bump = global_config.bump,
        has_one = fusion_mint,
        has_one = treasury_vault,
    )]
    pub global_config: Account<'info, GlobalConfig>,
    pub fusion_mint: Account<'info, Mint>,
    #[account(mut, constraint = treasury_vault.key() == global_config.treasury_vault)]
    pub treasury_vault: Account<'info, TokenAccount>,
    #[account(
        mut,
        seeds = [STAKE_POOL_SEED],
        bump = stake_pool.bump,
        has_one = stake_vault,
        has_one = fusion_mint,
    )]
    pub stake_pool: Account<'info, StakePool>,
    #[account(mut)]
    pub stake_vault: Account<'info, TokenAccount>,
    #[account(
        init_if_needed,
        payer = owner,
        space = 8 + StakePosition::INIT_SPACE,
        seeds = [STAKE_POS_SEED, owner.key().as_ref()],
        bump,
    )]
    pub stake_position: Account<'info, StakePosition>,
    #[account(
        mut,
        constraint = user_fusion.owner == owner.key() @ SnowboardDepinError::InvalidRecipient,
        constraint = user_fusion.mint == fusion_mint.key() @ SnowboardDepinError::InvalidMint,
    )]
    pub user_fusion: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct RequestUnstake<'info> {
    pub owner: Signer<'info>,
    #[account(
        seeds = [CONFIG_SEED],
        bump = global_config.bump,
        has_one = treasury_vault,
    )]
    pub global_config: Account<'info, GlobalConfig>,
    #[account(mut)]
    pub treasury_vault: Account<'info, TokenAccount>,
    #[account(mut, seeds = [STAKE_POOL_SEED], bump = stake_pool.bump)]
    pub stake_pool: Account<'info, StakePool>,
    #[account(
        mut,
        seeds = [STAKE_POS_SEED, owner.key().as_ref()],
        bump = stake_position.bump,
        has_one = owner,
    )]
    pub stake_position: Account<'info, StakePosition>,
    #[account(
        mut,
        constraint = user_fusion.owner == owner.key(),
        constraint = user_fusion.mint == global_config.fusion_mint,
    )]
    pub user_fusion: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct CompleteUnstake<'info> {
    pub owner: Signer<'info>,
    #[account(
        mut,
        seeds = [STAKE_POOL_SEED],
        bump = stake_pool.bump,
        has_one = stake_vault,
    )]
    pub stake_pool: Account<'info, StakePool>,
    #[account(mut)]
    pub stake_vault: Account<'info, TokenAccount>,
    #[account(
        mut,
        seeds = [STAKE_POS_SEED, owner.key().as_ref()],
        bump = stake_position.bump,
        has_one = owner,
    )]
    pub stake_position: Account<'info, StakePosition>,
    #[account(mut, constraint = user_fusion.owner == owner.key())]
    pub user_fusion: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct ApplyHalving<'info> {
    pub admin: Signer<'info>,
    #[account(
        mut,
        seeds = [STAKE_POOL_SEED],
        bump = stake_pool.bump,
        has_one = admin,
    )]
    pub stake_pool: Account<'info, StakePool>,
}

#[derive(Accounts)]
pub struct IngestProtocolFees<'info> {
    pub admin: Signer<'info>,
    #[account(
        seeds = [CONFIG_SEED],
        bump = global_config.bump,
        has_one = admin,
        has_one = fusion_mint,
        has_one = treasury_vault,
        has_one = fee_vault,
    )]
    pub global_config: Account<'info, GlobalConfig>,
    #[account(mut)]
    pub fusion_mint: Account<'info, Mint>,
    #[account(mut)]
    pub treasury_vault: Account<'info, TokenAccount>,
    #[account(mut)]
    pub fee_vault: Account<'info, TokenAccount>,
    #[account(mut, seeds = [STAKE_POOL_SEED], bump = stake_pool.bump)]
    pub stake_pool: Account<'info, StakePool>,
    pub token_program: Program<'info, Token>,
}
