use anchor_lang::prelude::*;

declare_id!("EC7PmYuSvVtDghHQwdLe4eraFtAGGMyKQAuDetTGdZci");

pub const SEARCH_COOLDOWN: i64 = 60;
pub const RESOURCES_PER_SEARCH: u8 = 3;

#[program]
pub mod search {
    use super::*;

    /// Initialize search config
    pub fn initialize_search(ctx: Context<InitializeSearch>) -> Result<()> {
        let config = &mut ctx.accounts.search_config;
        config.resource_manager_program = ctx.accounts.resource_manager_program.key();
        config.game_config = ctx.accounts.game_config.key();
        config.bump = ctx.bumps.search_config;
        Ok(())
    }

    /// Search for resources - cooldown 60 seconds
    pub fn search_resources(ctx: Context<SearchResources>) -> Result<()> {
        let player = &mut ctx.accounts.player;
        let clock = Clock::get()?;
        let now = clock.unix_timestamp;

        // Check cooldown
        if player.last_search_timestamp != 0 {
            let elapsed = now - player.last_search_timestamp;
            require!(elapsed >= SEARCH_COOLDOWN, SearchError::CooldownNotExpired);
        }

        // Update timestamp
        player.last_search_timestamp = now;

        // Generate 3 random resources using on-chain randomness
        let slot = clock.slot;
        let seed = slot ^ (now as u64) ^ player.owner.to_bytes().iter().fold(0u64, |acc, &b| acc.wrapping_add(b as u64));

        for i in 0..RESOURCES_PER_SEARCH {
            let resource_index = ((seed.wrapping_mul(6364136223846793005u64).wrapping_add(i as u64)) % 6) as u8;
            msg!("Found resource index: {}", resource_index);
            // In real impl: CPI to resource_manager::mint_resource
            // Simplified for compilation
        }

        msg!("Search completed at timestamp: {}", now);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitializeSearch<'info> {
    #[account(
        init,
        payer = admin,
        space = 8 + SearchConfig::INIT_SPACE,
        seeds = [b"search_config"],
        bump
    )]
    pub search_config: Account<'info, SearchConfig>,
    /// CHECK: just storing the pubkey
    pub resource_manager_program: AccountInfo<'info>,
    /// CHECK: just storing the pubkey
    pub game_config: AccountInfo<'info>,
    #[account(mut)]
    pub admin: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct SearchResources<'info> {
    #[account(seeds = [b"search_config"], bump = search_config.bump)]
    pub search_config: Account<'info, SearchConfig>,
    #[account(
        mut,
        seeds = [b"player", owner.key().as_ref()],
        bump = player.bump,
        has_one = owner
    )]
    pub player: Account<'info, Player>,
    pub owner: Signer<'info>,
}

#[account]
#[derive(InitSpace)]
pub struct SearchConfig {
    pub resource_manager_program: Pubkey,
    pub game_config: Pubkey,
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct Player {
    pub owner: Pubkey,
    pub last_search_timestamp: i64,
    pub bump: u8,
}

#[error_code]
pub enum SearchError {
    #[msg("Search cooldown not expired yet (60 seconds)")]
    CooldownNotExpired,
}
