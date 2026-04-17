use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

declare_id!("28zTRgf8DT1Qyf1ADDirTK5c4tAEEKfdsBSct5mUJ4za");

#[program]
pub mod resource_manager {
    use super::*;

    /// Initialize the game configuration
    pub fn initialize_game(ctx: Context<InitializeGame>) -> Result<()> {
        let config = &mut ctx.accounts.game_config;
        config.admin = ctx.accounts.admin.key();
        config.bump = ctx.bumps.game_config;
        msg!("Game initialized by admin: {}", config.admin);
        Ok(())
    }

    /// Register search program
    pub fn set_search_program(ctx: Context<SetProgram>, search_program: Pubkey) -> Result<()> {
        ctx.accounts.game_config.search_program = search_program;
        Ok(())
    }

    /// Register crafting program
    pub fn set_crafting_program(ctx: Context<SetProgram>, crafting_program: Pubkey) -> Result<()> {
        ctx.accounts.game_config.crafting_program = crafting_program;
        Ok(())
    }

    /// Store resource mint address
    pub fn register_resource_mint(
        ctx: Context<RegisterResourceMint>,
        resource_index: u8,
    ) -> Result<()> {
        require!(resource_index < 6, GameError::InvalidResourceIndex);
        let config = &mut ctx.accounts.game_config;
        config.resource_mints[resource_index as usize] = ctx.accounts.resource_mint.key();
        msg!("Resource mint {} registered: {}", resource_index, ctx.accounts.resource_mint.key());
        Ok(())
    }

    /// Register player
    pub fn register_player(ctx: Context<RegisterPlayer>) -> Result<()> {
        let player = &mut ctx.accounts.player;
        player.owner = ctx.accounts.owner.key();
        player.last_search_timestamp = 0;
        player.bump = ctx.bumps.player;
        msg!("Player registered: {}", player.owner);
        Ok(())
    }

    /// Mint resource - only callable from search or crafting
    pub fn mint_resource(ctx: Context<MintResource>, amount: u64) -> Result<()> {
        let config = &ctx.accounts.game_config;
        let caller = ctx.accounts.caller_program.key();
        require!(
            caller == config.search_program || caller == config.crafting_program,
            GameError::UnauthorizedCaller
        );

        let seeds = &[b"game_config".as_ref(), &[config.bump]];
        let signer = &[&seeds[..]];

        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            anchor_spl::token::MintTo {
                mint: ctx.accounts.resource_mint.to_account_info(),
                to: ctx.accounts.player_token_account.to_account_info(),
                authority: ctx.accounts.game_config.to_account_info(),
            },
            signer,
        );
        anchor_spl::token::mint_to(cpi_ctx, amount)?;
        Ok(())
    }

    /// Burn resource - only callable from crafting
    pub fn burn_resource(ctx: Context<BurnResource>, amount: u64) -> Result<()> {
        let config = &ctx.accounts.game_config;
        require!(
            ctx.accounts.caller_program.key() == config.crafting_program,
            GameError::UnauthorizedCaller
        );

        let cpi_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            anchor_spl::token::Burn {
                mint: ctx.accounts.resource_mint.to_account_info(),
                from: ctx.accounts.player_token_account.to_account_info(),
                authority: ctx.accounts.owner.to_account_info(),
            },
        );
        anchor_spl::token::burn(cpi_ctx, amount)?;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitializeGame<'info> {
    #[account(
        init,
        payer = admin,
        space = 8 + GameConfig::INIT_SPACE,
        seeds = [b"game_config"],
        bump
    )]
    pub game_config: Account<'info, GameConfig>,
    #[account(mut)]
    pub admin: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct SetProgram<'info> {
    #[account(
        mut,
        seeds = [b"game_config"],
        bump = game_config.bump,
        has_one = admin
    )]
    pub game_config: Account<'info, GameConfig>,
    pub admin: Signer<'info>,
}

#[derive(Accounts)]
pub struct RegisterResourceMint<'info> {
    #[account(
        mut,
        seeds = [b"game_config"],
        bump = game_config.bump,
        has_one = admin
    )]
    pub game_config: Account<'info, GameConfig>,
    pub resource_mint: Account<'info, Mint>,
    #[account(mut)]
    pub admin: Signer<'info>,
}

#[derive(Accounts)]
pub struct RegisterPlayer<'info> {
    #[account(
        init,
        payer = owner,
        space = 8 + Player::INIT_SPACE,
        seeds = [b"player", owner.key().as_ref()],
        bump
    )]
    pub player: Account<'info, Player>,
    #[account(mut)]
    pub owner: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct MintResource<'info> {
    #[account(seeds = [b"game_config"], bump = game_config.bump)]
    pub game_config: Account<'info, GameConfig>,
    #[account(mut)]
    pub resource_mint: Account<'info, Mint>,
    #[account(mut)]
    pub player_token_account: Account<'info, TokenAccount>,
    /// CHECK: validated in instruction
    pub caller_program: AccountInfo<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct BurnResource<'info> {
    #[account(seeds = [b"game_config"], bump = game_config.bump)]
    pub game_config: Account<'info, GameConfig>,
    #[account(mut)]
    pub resource_mint: Account<'info, Mint>,
    #[account(mut)]
    pub player_token_account: Account<'info, TokenAccount>,
    /// CHECK: validated in instruction
    pub caller_program: AccountInfo<'info>,
    pub owner: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

#[account]
#[derive(InitSpace)]
pub struct GameConfig {
    pub admin: Pubkey,
    pub resource_mints: [Pubkey; 6],
    pub magic_token_mint: Pubkey,
    pub search_program: Pubkey,
    pub crafting_program: Pubkey,
    pub item_prices: [u64; 4],
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
pub enum GameError {
    #[msg("Invalid resource index")]
    InvalidResourceIndex,
    #[msg("Unauthorized caller program")]
    UnauthorizedCaller,
}
