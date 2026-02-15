use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token_interface::{approve, Approve};
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};
declare_id!("8Ypi5qkweqhYCzaX7J8hqoPdzRJfupGgSoUcd7bZPePo");

#[program]
pub mod escrow {
    use super::*;

    #[instruction(discriminator = 0)]
    pub fn make(ctx: Context<Make>, seed: u64, receive: u64, amount: u64) -> Result<()> {
        approve(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Approve {
                    to: ctx.accounts.mint_b.to_account_info(),
                    delegate: ctx.accounts.mint_a.to_account_info(),
                    authority: ctx.accounts.signer.to_account_info(),
                },
            ),
            amount,
        )?;
        Ok(())
    }
    #[instruction(discriminator = 1)]
    pub fn take(ctx: Context<Take>) -> Result<()> {
        Ok(())
    }
    #[instruction(discriminator = 2)]
    pub fn refund(ctx: Context<Refund>) -> Result<()> {
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Make<'i> {
    // #[account(
    // init,
    // payer = user)]
    // pub user: Account<'info>,
    #[account(mut)]
    pub signer: Signer<'i>,

    // #[account(
    //     seeds = [b"state", user.key().as_ref()],
    //     bump = vault_state.state_bump,
    // )]
    // pub escrow_state: Account<'i, EscrowState>,
    // pub seed: u64,
    #[account(
        init_if_needed,
        payer = signer,
        associated_token::mint = mint_a,
        associated_token::authority = signer,
        associated_token::token_program = token_program,
    )]
    pub mint_ata: InterfaceAccount<'i, TokenAccount>,

    pub token_program: Interface<'i, TokenInterface>,
    pub associated_token_program: Program<'i, AssociatedToken>,

    pub mint_a: InterfaceAccount<'i, Mint>,
    pub mint_b: InterfaceAccount<'i, Mint>,
    // pub recieve: u64,

    // #[account(
    //     mut,
    //     seeds = [b"vault", vault_state.key().as_ref()],
    //     bump = vault_state.vault_bump,
    // )]
    #[account(
        init,
        payer = signer,
        seeds = [b"state", signer.key().as_ref()], 
        bump,
        space = VaultState::DISCRIMINATOR.len() + VaultState::INIT_SPACE,
    )]
    pub vault_state: Account<'i, VaultState>,

    #[account(
        mut,
        seeds = [b"vault", vault_state.key().as_ref()], 
        bump = vault_state.vault_bump,
    )]
    pub vault: SystemAccount<'i>,
    // pub make_bump: u8,
    pub system_program: Program<'i, System>,
}

#[derive(Accounts)]
pub struct Take {}
#[derive(Accounts)]
pub struct Refund {}

#[derive(InitSpace)]
#[account]
pub struct VaultState {
    pub vault_bump: u8,
    pub state_bump: u8,
}
