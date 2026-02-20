use anchor_lang::prelude::*;
mod instructions;

use instructions::*;
mod state;

declare_id!("8Ypi5qkweqhYCzaX7J8hqoPdzRJfupGgSoUcd7bZPePo");

#[program]
pub mod escrow {

    use super::*;

    pub fn make(ctx: Context<Make>, seed: u64, receive: u64, amount: u64) -> Result<()> {
        ctx.accounts.tranfer(amount);
        instructions::make::Make::initialize(ctx.accounts, seed, receive, &ctx.bumps);
        Ok(())
    }

    pub fn refund(ctx: Context<Refund>) -> Result<()> {
        instructions::refund::refund(ctx)
    }

    pub fn take(ctx: Context<Take>) -> Result<()> {
        instructions::take::take(ctx)
    }
}
