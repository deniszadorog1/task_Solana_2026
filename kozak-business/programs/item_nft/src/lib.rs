use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

declare_id!("34pqnbRPyWF8okanpnkGjKPEgy9A16sqfcTRHYnXTi8W");

#[program]
pub mod item_nft {
    use super::*;

    pub fn initialize_item_config(ctx: Context<InitializeItemConfig>) -> Result<()> {
        let config = &mut ctx.accounts.item_config;
        config.admin = ctx.accounts.admin.key();
        config.marketplace_program = Pubkey::default();
        config.crafting_program = Pubkey::default();
        config.bump = ctx.bumps.item_config;
        Ok(())
    }

    pub fn set_marketplace_program(ctx: Context<SetProgram>, marketplace_program: Pubkey) -> Result<()> {
        ctx.accounts.item_config.marketplace_program = marketplace_program;
        Ok(())
    }

    pub fn set_crafting_program(ctx: Context<SetProgram>, crafting_program: Pubkey) -> Result<()> {
        ctx.accounts.item_config.crafting_program = crafting_program;
        Ok(())
    }

    pub fn mint_item_nft(ctx: Context<MintItemNft>, item_type: u8) -> Result<()> {
        require!(item_type < 4, ItemNftError::InvalidItemType);
        let config = &ctx.accounts.item_config;
        require!(
            ctx.accounts.caller_program.key() == config.crafting_program,
            ItemNftError::UnauthorizedCaller
        );
        let metadata = &mut ctx.accounts.item_metadata;
        metadata.item_type = item_type;
        metadata.owner = ctx.accounts.recipient.key();
        metadata.mint = ctx.accounts.item_mint.key();
        metadata.bump = ctx.bumps.item_metadata;

        msg!("Minted NFT item type {}", item_type);
        Ok(())
    }

    pub fn burn_item_nft(ctx: Context<BurnItemNft>) -> Result<()> {
        let config = &ctx.accounts.item_config;
        require!(
            ctx.accounts.caller_program.key() == config.marketplace_program,
            ItemNftError::UnauthorizedCaller
        );
        let cpi_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            anchor_spl::token::Burn {
                mint: ctx.accounts.item_mint.to_account_info(),
                from: ctx.accounts.owner_token_account.to_account_info(),
                authority: ctx.accounts.owner.to_account_info(),
            },
        );
        anchor_spl::token::burn(cpi_ctx, 1)?;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitializeItemConfig<'info> {
    #[account(init, payer = admin, space = 8 + ItemConfig::INIT_SPACE, seeds = [b"item_config"], bump)]
    pub item_config: Account<'info, ItemConfig>,
    #[account(mut)]
    pub admin: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct SetProgram<'info> {
    #[account(mut, seeds = [b"item_config"], bump = item_config.bump, has_one = admin)]
    pub item_config: Account<'info, ItemConfig>,
    pub admin: Signer<'info>,
}

#[derive(Accounts)]
pub struct MintItemNft<'info> {
    #[account(seeds = [b"item_config"], bump = item_config.bump)]
    pub item_config: Account<'info, ItemConfig>,
    /// CHECK: new mint keypair passed by crafting
    pub item_mint: AccountInfo<'info>,
    #[account(
        init,
        payer = recipient,
        space = 8 + ItemMetadata::INIT_SPACE,
        seeds = [b"item_metadata", item_mint.key().as_ref()],
        bump
    )]
    pub item_metadata: Account<'info, ItemMetadata>,
    #[account(mut)]
    pub recipient: Signer<'info>,
    /// CHECK: validated in instruction
    pub caller_program: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct BurnItemNft<'info> {
    #[account(seeds = [b"item_config"], bump = item_config.bump)]
    pub item_config: Account<'info, ItemConfig>,
    #[account(mut)]
    pub item_mint: Account<'info, Mint>,
    #[account(mut)]
    pub owner_token_account: Account<'info, TokenAccount>,
    /// CHECK: validated in instruction
    pub caller_program: AccountInfo<'info>,
    pub owner: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

#[account]
#[derive(InitSpace)]
pub struct ItemConfig {
    pub admin: Pubkey,
    pub marketplace_program: Pubkey,
    pub crafting_program: Pubkey,
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct ItemMetadata {
    pub item_type: u8,
    pub owner: Pubkey,
    pub mint: Pubkey,
    pub bump: u8,
}

#[error_code]
pub enum ItemNftError {
    #[msg("Invalid item type")]
    InvalidItemType,
    #[msg("Unauthorized caller")]
    UnauthorizedCaller,
}
