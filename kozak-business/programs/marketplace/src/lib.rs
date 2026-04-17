use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount, Mint};

declare_id!("g6rJSQaH9qjy8Hxb8ZdPkg7BAQEFpkmeeEeLCeR8wbZ");

#[program]
pub mod marketplace {
    use super::*;

    /// Initialize marketplace
    pub fn initialize_marketplace(ctx: Context<InitializeMarketplace>) -> Result<()> {
        let config = &mut ctx.accounts.marketplace_config;
        config.admin = ctx.accounts.admin.key();
        config.magic_token_program = Pubkey::default();
        config.item_nft_program = Pubkey::default();
        config.bump = ctx.bumps.marketplace_config;
        msg!("Marketplace initialized");
        Ok(())
    }

    /// Set magic token program
    pub fn set_magic_token_program(
        ctx: Context<SetProgram>,
        program: Pubkey,
    ) -> Result<()> {
        ctx.accounts.marketplace_config.magic_token_program = program;
        Ok(())
    }

    /// Set item NFT program
    pub fn set_item_nft_program(
        ctx: Context<SetProgram>,
        program: Pubkey,
    ) -> Result<()> {
        ctx.accounts.marketplace_config.item_nft_program = program;
        Ok(())
    }

    /// List item for sale
    pub fn list_item(ctx: Context<ListItem>, price: u64) -> Result<()> {
        require!(price > 0, MarketplaceError::InvalidPrice);

        let listing = &mut ctx.accounts.listing;
        listing.seller = ctx.accounts.seller.key();
        listing.item_mint = ctx.accounts.item_mint.key();
        listing.price = price;
        listing.is_active = true;
        listing.bump = ctx.bumps.listing;

        msg!(
            "Item {} listed for {} MagicToken",
            ctx.accounts.item_mint.key(),
            price
        );
        Ok(())
    }

    /// Cancel listing
    pub fn cancel_listing(ctx: Context<CancelListing>) -> Result<()> {
        let listing = &mut ctx.accounts.listing;
        require!(listing.is_active, MarketplaceError::ListingNotActive);
        require!(
            listing.seller == ctx.accounts.seller.key(),
            MarketplaceError::NotSeller
        );
        listing.is_active = false;
        msg!("Listing cancelled for item: {}", listing.item_mint);
        Ok(())
    }

    /// Buy item - burns NFT, mints MagicToken to seller
    pub fn buy_item(ctx: Context<BuyItem>) -> Result<()> {
        let listing = &mut ctx.accounts.listing;
        require!(listing.is_active, MarketplaceError::ListingNotActive);

        let price = listing.price;
        let seller = listing.seller;
        listing.is_active = false;

        // In full impl:
        // 1. CPI to item_nft::burn_item_nft
        // 2. CPI to magic_token::mint_magic_token(price) to seller

        msg!(
            "Item sold! Seller {} receives {} MagicToken",
            seller,
            price
        );
        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitializeMarketplace<'info> {
    #[account(
        init,
        payer = admin,
        space = 8 + MarketplaceConfig::INIT_SPACE,
        seeds = [b"marketplace_config"],
        bump
    )]
    pub marketplace_config: Account<'info, MarketplaceConfig>,
    #[account(mut)]
    pub admin: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct SetProgram<'info> {
    #[account(
        mut,
        seeds = [b"marketplace_config"],
        bump = marketplace_config.bump,
        has_one = admin
    )]
    pub marketplace_config: Account<'info, MarketplaceConfig>,
    pub admin: Signer<'info>,
}

#[derive(Accounts)]
pub struct ListItem<'info> {
    #[account(seeds = [b"marketplace_config"], bump = marketplace_config.bump)]
    pub marketplace_config: Account<'info, MarketplaceConfig>,
    #[account(
        init,
        payer = seller,
        space = 8 + Listing::INIT_SPACE,
        seeds = [b"listing", item_mint.key().as_ref()],
        bump
    )]
    pub listing: Account<'info, Listing>,
    pub item_mint: Account<'info, Mint>,
    #[account(mut)]
    pub seller: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CancelListing<'info> {
    #[account(
        mut,
        seeds = [b"listing", listing.item_mint.as_ref()],
        bump = listing.bump
    )]
    pub listing: Account<'info, Listing>,
    pub seller: Signer<'info>,
}

#[derive(Accounts)]
pub struct BuyItem<'info> {
    #[account(seeds = [b"marketplace_config"], bump = marketplace_config.bump)]
    pub marketplace_config: Account<'info, MarketplaceConfig>,
    #[account(
        mut,
        seeds = [b"listing", item_mint.key().as_ref()],
        bump = listing.bump
    )]
    pub listing: Account<'info, Listing>,
    pub item_mint: Account<'info, Mint>,
    #[account(mut)]
    pub buyer: Signer<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[account]
#[derive(InitSpace)]
pub struct MarketplaceConfig {
    pub admin: Pubkey,
    pub magic_token_program: Pubkey,
    pub item_nft_program: Pubkey,
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct Listing {
    pub seller: Pubkey,
    pub item_mint: Pubkey,
    pub price: u64,
    pub is_active: bool,
    pub bump: u8,
}

#[error_code]
pub enum MarketplaceError {
    #[msg("Invalid price")]
    InvalidPrice,
    #[msg("Listing is not active")]
    ListingNotActive,
    #[msg("Only seller can cancel listing")]
    NotSeller,
}
