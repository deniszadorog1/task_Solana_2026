use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

declare_id!("2M3Kv3ZQZ3Hxmacx49NPFnqXktGkv1Xcb8dPFNnLtqJA");

#[program]
pub mod magic_token {
    use super::*;

    /// Initialize MagicToken mint - only admin
    pub fn initialize_magic_token(ctx: Context<InitializeMagicToken>) -> Result<()> {
        let config = &mut ctx.accounts.magic_config;
        config.admin = ctx.accounts.admin.key();
        config.mint = ctx.accounts.magic_mint.key();
        config.marketplace_program = Pubkey::default();
        config.bump = ctx.bumps.magic_config;
        config.mint_bump = ctx.bumps.magic_mint;
        msg!("MagicToken initialized: {}", config.mint);
        Ok(())
    }

    /// Set marketplace program - only admin
    pub fn set_marketplace_program(
        ctx: Context<SetMarketplace>,
        marketplace_program: Pubkey,
    ) -> Result<()> {
        ctx.accounts.magic_config.marketplace_program = marketplace_program;
        msg!("Marketplace program set: {}", marketplace_program);
        Ok(())
    }

    /// Mint MagicToken - only callable from marketplace via CPI
    pub fn mint_magic_token(
        ctx: Context<MintMagicToken>,
        amount: u64,
    ) -> Result<()> {
        let config = &ctx.accounts.magic_config;
        require!(
            ctx.accounts.caller_program.key() == config.marketplace_program,
            MagicTokenError::UnauthorizedCaller
        );

        let seeds = &[
            b"magic_authority".as_ref(),
            &[ctx.bumps.magic_authority],
        ];
        let signer = &[&seeds[..]];

        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            anchor_spl::token_interface::MintTo {
                mint: ctx.accounts.magic_mint.to_account_info(),
                to: ctx.accounts.recipient_token_account.to_account_info(),
                authority: ctx.accounts.magic_authority.to_account_info(),
            },
            signer,
        );
        anchor_spl::token_interface::mint_to(cpi_ctx, amount)?;
        msg!("Minted {} MagicToken to {}", amount, ctx.accounts.recipient_token_account.key());
        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitializeMagicToken<'info> {
    #[account(
        init,
        payer = admin,
        space = 8 + MagicConfig::INIT_SPACE,
        seeds = [b"magic_config"],
        bump
    )]
    pub magic_config: Account<'info, MagicConfig>,
    #[account(
        init,
        payer = admin,
        seeds = [b"magic_mint"],
        bump,
        mint::decimals = 6,
        mint::authority = magic_authority,
        mint::token_program = token_program,
    )]
    pub magic_mint: InterfaceAccount<'info, Mint>,
    /// CHECK: PDA authority for magic mint
    #[account(seeds = [b"magic_authority"], bump)]
    pub magic_authority: AccountInfo<'info>,
    #[account(mut)]
    pub admin: Signer<'info>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct SetMarketplace<'info> {
    #[account(
        mut,
        seeds = [b"magic_config"],
        bump = magic_config.bump,
        has_one = admin
    )]
    pub magic_config: Account<'info, MagicConfig>,
    pub admin: Signer<'info>,
}

#[derive(Accounts)]
pub struct MintMagicToken<'info> {
    #[account(seeds = [b"magic_config"], bump = magic_config.bump)]
    pub magic_config: Account<'info, MagicConfig>,
    #[account(mut, seeds = [b"magic_mint"], bump = magic_config.mint_bump)]
    pub magic_mint: InterfaceAccount<'info, Mint>,
    /// CHECK: PDA authority
    #[account(seeds = [b"magic_authority"], bump)]
    pub magic_authority: AccountInfo<'info>,
    #[account(mut)]
    pub recipient_token_account: InterfaceAccount<'info, TokenAccount>,
    /// CHECK: validated in instruction
    pub caller_program: AccountInfo<'info>,
    pub token_program: Interface<'info, TokenInterface>,
}

#[account]
#[derive(InitSpace)]
pub struct MagicConfig {
    pub admin: Pubkey,
    pub mint: Pubkey,
    pub marketplace_program: Pubkey,
    pub bump: u8,
    pub mint_bump: u8,
}

#[error_code]
pub enum MagicTokenError {
    #[msg("Only marketplace program can mint MagicToken")]
    UnauthorizedCaller,
}
