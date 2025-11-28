use anchor_lang::prelude::*;

// THE ROBOT'S MEMORY
#[account]
#[derive(InitSpace)]
pub struct PoolAccount {
    // Which tokens are in the pool?
    pub token_a_mint: Pubkey,
    pub token_b_mint: Pubkey,

    // How much of each token stored safely as huge numbers
    pub token_a_amount: u128,
    pub token_b_amount: u128,

    // The address of the receipt token the pool will mint
    pub lp_token_mint: Pubkey,

    pub bump: u8,
}

impl PoolAccount {
    pub const LEN: usize = 8 + PoolAccount::INIT_SPACE;
}
