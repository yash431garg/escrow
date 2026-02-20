use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token_interface::{
     close_account, CloseAccount, Mint, TokenAccount, TokenInterface, TransferChecked,
};
use anchor_spl::token_interface;


use crate::state::EscrowState;

#[instruction(discriminator = 1)]
pub fn take(ctx: Context<Take>) -> Result<()> {
       // Transfer 1: Taker pays maker with mint_b tokens
    let transfer_taker_to_maker = TransferChecked {
        mint: ctx.accounts.mint_b.to_account_info(),
        from: ctx.accounts.mint_ata_b.to_account_info(),
        to: ctx.accounts.signer_ata_b.to_account_info(),
        authority: ctx.accounts.taker.to_account_info(),
    };

    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_context = CpiContext::new(cpi_program, transfer_taker_to_maker);
    token_interface::transfer_checked(
        cpi_context,
        ctx.accounts.escrow_state.receive,
        ctx.accounts.mint_b.decimals,
    )?;

    // Signer seeds for escrow_state PDA
    let signer_seeds: &[&[&[u8]]] = &[&[
        b"escrow",
        ctx.accounts.maker.to_account_info().key.as_ref(),
        &ctx.accounts.escrow_state.seed.to_le_bytes(),
        &[ctx.accounts.escrow_state.escrow_state_bump],
    ]];

    // Transfer 2: Vault pays taker with mint_a tokens
    let transfer_vault_to_taker = TransferChecked {
        mint: ctx.accounts.mint_a.to_account_info(),
        from: ctx.accounts.vault.to_account_info(),
        to: ctx.accounts.mint_ata_a.to_account_info(),
        authority: ctx.accounts.escrow_state.to_account_info(), // Fixed: escrow_state is the authority
    };

    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_context = CpiContext::new_with_signer(cpi_program, transfer_vault_to_taker, signer_seeds);
    token_interface::transfer_checked(
        cpi_context, 
        ctx.accounts.vault.amount, 
        ctx.accounts.mint_a.decimals
    )?;

    // Close vault account
    let close_accounts = CloseAccount {
        account: ctx.accounts.vault.to_account_info(),
        destination: ctx.accounts.maker.to_account_info(),
        authority: ctx.accounts.escrow_state.to_account_info(),
    };

    let close_cpi_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        close_accounts,
        signer_seeds,
    );

    close_account(close_cpi_ctx)?;

    Ok(())
}

#[derive(Accounts)]
pub struct Take<'t> {
    #[account(mut)]
    pub taker: Signer<'t>,

    /// CHECK: This is the maker from escrow_state
    #[account(mut)]
    pub maker: AccountInfo<'t>,

    pub mint_a: InterfaceAccount<'t, Mint>,

    #[account(
        mint::token_program = token_program
    )]
    pub mint_b: InterfaceAccount<'t, Mint>,

    #[account(
        init_if_needed,
        payer = taker, 
        associated_token::mint = mint_a,
        associated_token::authority = taker,
        associated_token::token_program = token_program,
    )]
    pub mint_ata_a: InterfaceAccount<'t, TokenAccount>,

    #[account(
        init_if_needed,
        payer = taker, 
        associated_token::mint = mint_b,
        associated_token::authority = taker,
        associated_token::token_program = token_program,
    )]
    pub mint_ata_b: InterfaceAccount<'t, TokenAccount>,

    #[account(
        init_if_needed,
        payer = taker, 
        associated_token::mint = mint_b,
        associated_token::authority = maker,
        associated_token::token_program = token_program,
    )]
    pub signer_ata_b: InterfaceAccount<'t, TokenAccount>,

    #[account(
        mut,
        seeds = [b"escrow", escrow_state.maker.key().as_ref(), &escrow_state.seed.to_le_bytes()], 
        has_one = maker,
        bump,
        close = maker,
    )]
    pub escrow_state: Account<'t, EscrowState>,

    #[account(
        mut,
        associated_token::mint = mint_a,
        associated_token::authority = escrow_state,
        associated_token::token_program = token_program,
    )]
    pub vault: InterfaceAccount<'t, TokenAccount>,

    pub token_program: Interface<'t, TokenInterface>,
    pub associated_token_program: Program<'t, AssociatedToken>,
    pub system_program: Program<'t, System>,
}
