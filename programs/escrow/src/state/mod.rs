use anchor_lang::prelude::*;

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
