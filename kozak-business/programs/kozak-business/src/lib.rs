use anchor_lang::prelude::*;

declare_id!("Cf6GFUyNA8ZiiUgPN1T98DzzVwCgB3TrJEA4k4p2Atq3");

#[program]
pub mod kozak_business {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
