use anchor_lang::prelude::*;
use anchor_spl::token_interface::{
    transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked,
};

use crate::state::EscrowState;

use anchor_spl::associated_token::AssociatedToken;

// #[instruction(discriminator = 0)]
// pub fn make(ctx: Context<Make>, seed: u64, receive: u64, amount: u64) -> Result<()> {
//     // saving the escrow pda basically the data
//     // which will be used throught the other instructions and this
//     ctx.accounts.escrow_state.seed = seed;
//     ctx.accounts.escrow_state.maker = ctx.accounts.signer.key();
//     ctx.accounts.escrow_state.receive = receive;
//     ctx.accounts.escrow_state.mint_a = ctx.accounts.mint_a.key();
//     ctx.accounts.escrow_state.mint_b = ctx.accounts.mint_b.key();
//     ctx.accounts.escrow_state.escrow_state_bump = ctx.bumps.escrow_state;

//     Ok(())
// }

#[derive(Accounts)]
#[instruction(seed: u64)]
pub struct Make<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        mint::token_program = token_program
    )]
    pub mint_a: InterfaceAccount<'info, Mint>,

    #[account(
        mint::token_program = token_program
    )]
    pub mint_b: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        associated_token::mint = mint_a,
        associated_token::authority = signer,
        associated_token::token_program = token_program,
    )]
    pub mint_ata: InterfaceAccount<'info, TokenAccount>,

    #[account(
        init,
        payer = signer,
        seeds = [b"escrow", signer.key().as_ref(), seed.to_le_bytes().as_ref()], 
        bump,
        space = EscrowState::DISCRIMINATOR.len() + EscrowState::INIT_SPACE,
    )]
    pub escrow_state: Account<'info, EscrowState>,

    #[account(
        init_if_needed,
        payer = signer,
        associated_token::mint = mint_a,
        associated_token::authority = escrow_state,
        associated_token::token_program = token_program,
    )]
    pub vault: InterfaceAccount<'info, TokenAccount>,

    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl<'info> Make<'info> {
    pub fn initialize(&mut self, seed: u64, receive: u64, bump: &MakeBumps) -> Result<()> {
        // saving the escrow pda basically the data
        // which will be used throught the other instructions and this
        self.escrow_state.seed = seed;
        self.escrow_state.maker = self.signer.key();
        self.escrow_state.receive = receive;
        self.escrow_state.mint_a = self.mint_a.key();
        self.escrow_state.mint_b = self.mint_b.key();
        self.escrow_state.escrow_state_bump = bump.escrow_state;
        Ok(())
    }
    pub fn tranfer(&self, amount: u64) {
        // tranfer struct
        let transfer_accounts = TransferChecked {
            mint: self.mint_a.to_account_info(),
            from: self.mint_ata.to_account_info(),
            to: self.vault.to_account_info(),
            authority: self.signer.to_account_info(),
        };

        // transfer from user wallet that's why uing new
        let tranfer_cpi_ctx =
            CpiContext::new(self.token_program.to_account_info(), transfer_accounts);

        // transfer function
        transfer_checked(tranfer_cpi_ctx, amount, self.mint_a.decimals);
    }
}
