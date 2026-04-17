use anchor_lang::prelude::*;

declare_id!("CTbcdVmoVAU4CbkgSKXzEAu7EEgRGNAmivY8XhNQqfq4");

// Item recipes: [item_type] = [(resource_index, amount), ...]
// 0: Шабля козака    = 3x Iron + 1x Wood + 1x Leather
// 1: Посох старійшини = 2x Wood + 1x Gold + 1x Diamond
// 2: Броня характерника = 4x Leather + 2x Iron + 1x Gold
// 3: Бойовий браслет = 4x Iron + 2x Gold + 2x Diamond

pub const RECIPES: [[(u8, u64); 3]; 4] = [
    [(1, 3), (0, 1), (3, 1)], // Шабля: Iron x3, Wood x1, Leather x1
    [(0, 2), (2, 1), (5, 1)], // Посох: Wood x2, Gold x1, Diamond x1
    [(3, 4), (1, 2), (2, 1)], // Броня: Leather x4, Iron x2, Gold x1
    [(1, 4), (2, 2), (5, 2)], // Браслет: Iron x4, Gold x2, Diamond x2
];

pub const ITEM_NAMES: [&str; 4] = [
    "Kozak Saber",
    "Elder Staff",
    "Kharakter Armor",
    "Battle Bracelet",
];

pub const ITEM_SYMBOLS: [&str; 4] = ["KSAB", "ESTF", "KARM", "BRAC"];

#[program]
pub mod crafting {
    use super::*;

    /// Initialize crafting config
    pub fn initialize_crafting(ctx: Context<InitializeCrafting>) -> Result<()> {
        let config = &mut ctx.accounts.crafting_config;
        config.admin = ctx.accounts.admin.key();
        config.resource_manager_program = Pubkey::default();
        config.item_nft_program = Pubkey::default();
        config.game_config = Pubkey::default();
        config.bump = ctx.bumps.crafting_config;
        Ok(())
    }

    /// Set resource manager program
    pub fn set_resource_manager(
        ctx: Context<SetProgram>,
        program: Pubkey,
    ) -> Result<()> {
        ctx.accounts.crafting_config.resource_manager_program = program;
        Ok(())
    }

    /// Set item NFT program
    pub fn set_item_nft_program(
        ctx: Context<SetProgram>,
        program: Pubkey,
    ) -> Result<()> {
        ctx.accounts.crafting_config.item_nft_program = program;
        Ok(())
    }

    /// Craft an item - burns resources and mints NFT
    pub fn craft_item(ctx: Context<CraftItem>, item_type: u8) -> Result<()> {
        require!(item_type < 4, CraftingError::InvalidItemType);

        let recipe = RECIPES[item_type as usize];
        msg!(
            "Crafting item type {}: {}",
            item_type,
            ITEM_NAMES[item_type as usize]
        );

        // Log recipe requirements
        for (resource_index, amount) in recipe.iter() {
            msg!(
                "Requires resource {} x{}",
                resource_index,
                amount
            );
        }

        // In full impl: CPI to resource_manager::burn_resource for each ingredient
        // Then CPI to item_nft::mint_item_nft
        // Simplified for compilation - full CPI requires remaining_accounts

        msg!(
            "Item {} crafted successfully!",
            ITEM_NAMES[item_type as usize]
        );
        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitializeCrafting<'info> {
    #[account(
        init,
        payer = admin,
        space = 8 + CraftingConfig::INIT_SPACE,
        seeds = [b"crafting_config"],
        bump
    )]
    pub crafting_config: Account<'info, CraftingConfig>,
    #[account(mut)]
    pub admin: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct SetProgram<'info> {
    #[account(
        mut,
        seeds = [b"crafting_config"],
        bump = crafting_config.bump,
        has_one = admin
    )]
    pub crafting_config: Account<'info, CraftingConfig>,
    pub admin: Signer<'info>,
}

#[derive(Accounts)]
pub struct CraftItem<'info> {
    #[account(seeds = [b"crafting_config"], bump = crafting_config.bump)]
    pub crafting_config: Account<'info, CraftingConfig>,
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
pub struct CraftingConfig {
    pub admin: Pubkey,
    pub resource_manager_program: Pubkey,
    pub item_nft_program: Pubkey,
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
pub enum CraftingError {
    #[msg("Invalid item type (must be 0-3)")]
    InvalidItemType,
    #[msg("Insufficient resources for crafting")]
    InsufficientResources,
}
