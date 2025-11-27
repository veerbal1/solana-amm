use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, token::{Mint, Token, TokenAccount}};

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

        msg!("Pool Initialized! Token A: {}, Token B: {}", pool_state.token_a_mint, pool_state.token_b_mint);
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

    #[account(init, payer = signer, seeds = [b"lp", pool_account.key().as_ref()], bump, mint::decimals = 6, mint::authority = pool_account)]
    pub lp_token_mint: Account<'info, Mint>,

    #[account(init, seeds=[b"vault", pool_account.key().as_ref(), token_a_mint.key().as_ref()], bump ,token::mint = token_a_mint, token::authority = pool_account, payer = signer)]
    pub vault_a: Account<'info, TokenAccount>,

    #[account(init, seeds=[b"vault", pool_account.key().as_ref(), token_b_mint.key().as_ref()], bump ,token::mint = token_b_mint, token::authority = pool_account, payer = signer)]
    pub vault_b: Account<'info, TokenAccount>,

    pub system_program: Program<'info, System>,

    pub token_program: Program<'info, Token>
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

#[derive(Accounts)]
pub struct AddLiquidity<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(mut, 
        seeds = [b"pool", pool_account.token_a_mint.as_ref(), pool_account.token_b_mint.as_ref()],
        bump = pool_account.bump
    )] // Needs mut because we change amounts
    pub pool_account: Account<'info, PoolAccount>,

    #[account(
        mut,
        seeds = [b"vault", pool_account.key().as_ref(), pool_account.token_a_mint.as_ref()],
        bump,
        token::mint = pool_account.token_a_mint,
        token::authority = pool_account
    )]
    pub vault_a: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [b"vault", pool_account.key().as_ref(), pool_account.token_b_mint.as_ref()],
        bump,
        token::mint = pool_account.token_b_mint,
        token::authority = pool_account
    )]
    pub vault_b: Account<'info, TokenAccount>,

    #[account(mut)]
    pub lp_token_mint: Account<'info, Mint>,

    // --- YOUR TURN: Add the 3 User Accounts here ---
    #[account(mut, token::mint = pool_account.token_a_mint, token::authority = signer)]
    pub user_a_token: Account<'info, TokenAccount>,
    // 2. user_token_b (Must match pool's token_b_mint)
    #[account(mut, token::mint = pool_account.token_b_mint, token::authority = signer)]
    pub user_b_token: Account<'info, TokenAccount>,
    // 3. user_lp_token_account (Must match pool's lp_token_mint)
    #[account(mut, token::mint = pool_account.lp_token_mint, token::authority = signer)]
    pub user_lp_token_account: Account<'info, TokenAccount>,
    // ...

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}