use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token_interface;
use anchor_spl::token_interface::{
    close_account, transfer_checked, CloseAccount, Mint, TokenAccount, TokenInterface,
    TransferChecked,
};

declare_id!("8Ypi5qkweqhYCzaX7J8hqoPdzRJfupGgSoUcd7bZPePo");

#[program]
pub mod escrow {

    use super::*;

    #[instruction(discriminator = 0)]
    pub fn make(ctx: Context<Make>, seed: u64, receive: u64, amount: u64) -> Result<()> {
        ctx.accounts.escrow_state.seed = seed;
        ctx.accounts.escrow_state.maker = ctx.accounts.signer.key();
        ctx.accounts.escrow_state.receive = receive;
        ctx.accounts.escrow_state.mint_a = ctx.accounts.mint_a.key();
        ctx.accounts.escrow_state.mint_b = ctx.accounts.mint_b.key();

        ctx.accounts.escrow_state.escrow_state_bump = ctx.bumps.escrow_state;

        let decimals = ctx.accounts.mint_a.decimals;

        let cpi_accounts = TransferChecked {
            mint: ctx.accounts.mint_a.to_account_info(),
            from: ctx.accounts.mint_ata.to_account_info(),
            to: ctx.accounts.vault.to_account_info(),
            authority: ctx.accounts.signer.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_context = CpiContext::new(cpi_program, cpi_accounts);
        token_interface::transfer_checked(cpi_context, amount, decimals)?;

        Ok(())
    }
    #[instruction(discriminator = 1)]
    pub fn take(ctx: Context<Take>) -> Result<()> {
        Ok(())
    }
    #[instruction(discriminator = 2)]
    pub fn refund(ctx: Context<Refund>) -> Result<()> {
        let signer_seeds: &[&[&[u8]]] = &[&[
            b"escrow",
            ctx.accounts.signer.to_account_info().key.as_ref(),
            &ctx.accounts.escrow_state.seed.to_le_bytes(),
            &[ctx.accounts.escrow_state.escrow_state_bump],
        ]];

        let decimals = ctx.accounts.mint_a.decimals;
        let transfer_accounts = TransferChecked {
            mint: ctx.accounts.mint_a.to_account_info(),
            from: ctx.accounts.vault.to_account_info(),
            to: ctx.accounts.mint_ata.to_account_info(),
            authority: ctx.accounts.escrow_state.to_account_info(),
        };

        // let amount = ctx.accounts.escrow_state.receive;
        // let cpi_program = ctx.accounts.token_program.to_account_info();
        // let cpi_context = CpiContext::new(cpi_program, cpi_accounts);
        // token_interface::transfer_checked(cpi_context, amount, decimals)?;

        let tranfer_cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            transfer_accounts,
            signer_seeds,
        );

        transfer_checked(
            tranfer_cpi_ctx,
            ctx.accounts.vault.amount,
            ctx.accounts.mint_a.decimals,
        )?;

        let close_accounts = CloseAccount {
            account: ctx.accounts.vault.to_account_info(),
            destination: ctx.accounts.signer.to_account_info(),
            authority: ctx.accounts.escrow_state.to_account_info(),
        };

        let close_cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            close_accounts,
            signer_seeds,
        );

        close_account(close_cpi_ctx);
        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(seed: u64)]
pub struct Make<'i> {
    #[account(mut)]
    pub signer: Signer<'i>,

    #[account(
        mint::token_program = token_program
    )]
    pub mint_a: InterfaceAccount<'i, Mint>,

    #[account(
        mint::token_program = token_program
    )]
    pub mint_b: InterfaceAccount<'i, Mint>,

    #[account(
        mut,
        associated_token::mint = mint_a,
        associated_token::authority = signer,
        associated_token::token_program = token_program,
    )]
    pub mint_ata: InterfaceAccount<'i, TokenAccount>,

    #[account(
        init,
        payer = signer,
        seeds = [b"escrow", signer.key().as_ref(), seed.to_le_bytes().as_ref()], 
        bump,
        space = EscrowState::DISCRIMINATOR.len() + EscrowState::INIT_SPACE,
    )]
    pub escrow_state: Account<'i, EscrowState>,

    #[account(
        init_if_needed,
        payer = signer,
        associated_token::mint = mint_a,
        associated_token::authority = escrow_state,
        associated_token::token_program = token_program,
    )]
    pub vault: InterfaceAccount<'i, TokenAccount>,

    pub token_program: Interface<'i, TokenInterface>,
    pub associated_token_program: Program<'i, AssociatedToken>,
    pub system_program: Program<'i, System>,
}

#[derive(Accounts)]
pub struct Take {}
#[derive(Accounts)]
pub struct Refund<'r> {
    #[account(mut)]
    pub signer: Signer<'r>,

    pub mint_a: InterfaceAccount<'r, Mint>,

    #[account(
        mut,
        associated_token::mint = mint_a,
        associated_token::authority = signer,
        associated_token::token_program = token_program,
    )]
    pub mint_ata: InterfaceAccount<'r, TokenAccount>,

    #[account(
        mut,
        seeds = [b"escrow", signer.key().as_ref(), &escrow_state.seed.to_le_bytes() ], 
        has_one = mint_a,
        // has_one = signer,
        bump = escrow_state.escrow_state_bump,
        close = signer,
    )]
    pub escrow_state: Account<'r, EscrowState>,

    #[account(
        mut,
        associated_token::mint = mint_a,
        associated_token::authority = escrow_state,
        associated_token::token_program = token_program,
    )]
    pub vault: InterfaceAccount<'r, TokenAccount>,
    pub token_program: Interface<'r, TokenInterface>,
    pub associated_token_program: Program<'r, AssociatedToken>,
    pub system_program: Program<'r, System>,
}

#[derive(InitSpace)]
#[account]
pub struct EscrowState {
    pub escrow_state_bump: u8,
    pub receive: u64,
    pub maker: Pubkey,
    pub seed: u64,
    pub mint_a: Pubkey,
    pub mint_b: Pubkey,
}
