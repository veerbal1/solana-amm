use crate::errors::*;
use crate::state::pool::PoolAccount;
use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

pub fn initialize_pool(ctx: Context<InitializePool>) -> Result<()> {
    let pool_state = &mut ctx.accounts.pool_account;
    pool_state.lp_token_mint = ctx.accounts.lp_token_mint.key();
    pool_state.token_a_amount = 0;
    pool_state.token_a_mint = ctx.accounts.token_a_mint.key();
    pool_state.token_b_amount = 0;
    pool_state.token_b_mint = ctx.accounts.token_b_mint.key();
    pool_state.bump = ctx.bumps.pool_account;

    msg!(
        "Pool Initialized! Token A: {}, Token B: {}",
        pool_state.token_a_mint,
        pool_state.token_b_mint
    );
    Ok(())
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

    pub token_program: Program<'info, Token>,
}
