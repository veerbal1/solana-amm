use anchor_lang::prelude::*;
use anchor_spl::token::Mint;

declare_id!("BWYxEA3HTy5Kv7LCTWH68yN52a7aZMMPFCaUrWGmFtfK");


#[error_code]
pub enum AMMError {
    #[msg("Invalid Mint order")]
    InvalidMintOrder
}

#[program]
pub mod solana_amm {
    use super::*;

    pub fn initialize_pool(ctx: Context<InitializePool>) -> Result<()> {
        let pool_state = &mut ctx.accounts.pool_account;
        pool_state.lp_token_mint = ctx.accounts.lp_token_mint.key();
        pool_state.token_a_amount = 0;
        pool_state.token_a_mint = ctx.accounts.token_a_mint.key();
        pool_state.token_b_amount = 0;
        pool_state.token_b_mint = ctx.accounts.token_b_mint.key();
        pool_state.bump = ctx.bumps.pool_account;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitializePool<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(init, 
              payer = signer, 
              seeds = [b"pool", token_a_mint.key().as_ref(), token_b_mint.key().as_ref()], 
              bump, 
              space = PoolAccount::LEN,
              constraint  = token_a_mint.key() < token_b_mint.key() @ AMMError::InvalidMintOrder 
        )]
    pub pool_account: Account<'info, PoolAccount>,

    pub token_a_mint: Account<'info, Mint>,

    pub token_b_mint: Account<'info, Mint>,

    pub lp_token_mint: Account<'info, Mint>,

    pub system_program: Program<'info, System>,
}

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

    pub bump: u8
}

impl PoolAccount {
    pub const LEN: usize = 8 + PoolAccount::INIT_SPACE;
}
