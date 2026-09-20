use anchor_lang::prelude::*;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod miii_protocol {
    use super::*;

    // Регистрация нового устройства в реестре
    pub fn register_device(ctx: Context<RegisterDevice>, device_pubkey: Pubkey) -> Result<()> {
        let registry = &mut ctx.accounts.device_registry;
        registry.authority = ctx.accounts.authority.key();
        registry.device_pubkey = device_pubkey;
        registry.is_active = true;
        registry.bump = ctx.bumps.device_registry;
        Ok(())
    }

    // Запись движения с проверкой подписи устройства и анти-чит логикой
    pub fn record_motion(
        ctx: Context<VerifyAndRecordMotion>,
        _trick_type: u8,
        g_force: u32,
        timestamp: i64,
    ) -> Result<()> {
        let progress = &mut ctx.accounts.user_progress;
        let current_time = Clock::get()?.unix_timestamp;

        // --- АНТИ-ЧИТ ЛОГИКА ---
        // 1. Проверка таймстампа (допуск синхронизации в 60 секунд)
        require!(timestamp <= current_time && current_time - timestamp < 60, CustomError::InvalidTimestamp);

        // 2. Физические лимиты (например, перегрузка до 15.00G в сотых долях)
        require!(g_force <= 1500, CustomError::PhysicsLimitExceeded);

        // 3. Защита от спама (мин. 1 секунда между действиями)
        if progress.last_timestamp > 0 {
            require!(timestamp - progress.last_timestamp >= 1, CustomError::ActionTooFrequent);
        }

        // --- ОНЧЕЙН-ФИКСАЦИЯ ---
        progress.total_tricks = progress.total_tricks.checked_add(1).ok_or(error!(CustomError::Overflow))?;
        progress.last_timestamp = timestamp;

        msg!("Motion verified successfully for device: {}", ctx.accounts.physical_device.key());
        Ok(())
    }
}

#[derive(Accounts)]
pub struct RegisterDevice<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        init,
        payer = authority,
        space = 8 + DeviceRegistry::INIT_SPACE,
        seeds = [b"device", authority.key().as_ref()],
        bump
    )]
    pub device_registry: Account<'info, DeviceRegistry>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct VerifyAndRecordMotion<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        constraint = device_registry.device_pubkey == physical_device.key(),
        constraint = device_registry.is_active == true
    )]
    pub device_registry: Account<'info, DeviceRegistry>,

    /// CHECK: Публичный ключ физического устройства как дополнительный подписывающий элемент
    pub physical_device: Signer<'info>,

    #[account(
        init_if_needed,
        payer = user,
        space = 8 + UserProgress::INIT_SPACE,
        seeds = [b"progress", user.key().as_ref()],
        bump
    )]
    pub user_progress: Account<'info, UserProgress>,

    pub system_program: Program<'info, System>,
}

#[account]
#[derive(InitSpace)]
pub struct DeviceRegistry {
    pub authority: Pubkey,
    pub device_pubkey: Pubkey,
    pub is_active: bool,
    pub bump: u8,
}

impl DeviceRegistry {
    pub const INIT_SPACE: usize = 32 + 32 + 1 + 1; // authority + device_pubkey + is_active + bump
}

#[account]
#[derive(InitSpace)]
pub struct UserProgress {
    pub total_tricks: u32,
    pub last_timestamp: i64,
}

impl UserProgress {
    pub const INIT_SPACE: usize = 4 + 8; // total_tricks + last_timestamp
}

#[error_code]
pub enum CustomError {
    #[msg("Invalid or out-of-sync timestamp.")]
    InvalidTimestamp,
    #[msg("Physical limits exceeded — possible cheat detected.")]
    PhysicsLimitExceeded,
    #[msg("Actions are performed too frequently.")]
    ActionTooFrequent,
    #[msg("Overflow occurred.")]
    Overflow,
}
