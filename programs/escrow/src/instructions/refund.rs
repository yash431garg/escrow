use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token_interface::{
    close_account, transfer_checked, CloseAccount, Mint, TokenAccount, TokenInterface,
    TransferChecked,
};

use crate::state::EscrowState;

#[instruction(discriminator = 2)]
pub fn refund(ctx: Context<Refund>) -> Result<()> {
    // escrow state
    let signer_seeds: &[&[&[u8]]] = &[&[
        b"escrow",
        ctx.accounts.signer.to_account_info().key.as_ref(),
        &ctx.accounts.escrow_state.seed.to_le_bytes(),
        &[ctx.accounts.escrow_state.escrow_state_bump],
    ]];

    // transfer struct
    let transfer_accounts = TransferChecked {
        mint: ctx.accounts.mint_a.to_account_info(),
        from: ctx.accounts.vault.to_account_info(),
        to: ctx.accounts.mint_ata.to_account_info(),
        authority: ctx.accounts.escrow_state.to_account_info(),
    };

    // transfer from pda that's why uing new_with_signer
    let tranfer_cpi_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        transfer_accounts,
        signer_seeds,
    );

    // transfer function
    transfer_checked(
        tranfer_cpi_ctx,
        ctx.accounts.vault.amount,
        ctx.accounts.mint_a.decimals,
    )?;

    //close struct
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

    //close account function
    close_account(close_cpi_ctx);
    Ok(())
}

#[derive(Accounts)]
pub struct Refund<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    pub mint_a: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        associated_token::mint = mint_a,
        associated_token::authority = signer,
        associated_token::token_program = token_program,
    )]
    pub mint_ata: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [b"escrow", signer.key().as_ref(), &escrow_state.seed.to_le_bytes() ], 
        has_one = mint_a,
        // has_one = signer,
        bump = escrow_state.escrow_state_bump,
        close = signer,
    )]
    pub escrow_state: Account<'info, EscrowState>,

    #[account(
        mut,
        associated_token::mint = mint_a,
        associated_token::authority = escrow_state,
        associated_token::token_program = token_program,
    )]
    pub vault: InterfaceAccount<'info, TokenAccount>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}
