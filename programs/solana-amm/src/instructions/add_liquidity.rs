use crate::errors::*;
use crate::state::pool::PoolAccount;
use anchor_lang::prelude::*;
use anchor_spl::token::{self, MintTo, Transfer};
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount},
};

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

pub fn add_liquidity(ctx: Context<AddLiquidity>, amount_a: u64, amount_b: u64) -> Result<()> {
    // 1. Setup Accounts & Variables
    let pool_account = &mut ctx.accounts.pool_account;
    let lp_mint = &ctx.accounts.lp_token_mint;

    let amount_a_u128 = amount_a as u128;
    let amount_b_u128 = amount_b as u128;

    let liquidity_to_mint: u128;
    let amount_b_to_transfer: u64;

    // 2. The Math (Genesis vs Normal)
    if pool_account.token_a_amount == 0 {
        // --- GENESIS MODE ---
        msg!("Genesis Liquidity Deposit");

        // Formula: sqrt(x * y)
        liquidity_to_mint = (amount_a_u128.checked_mul(amount_b_u128).unwrap()).isqrt();

        // In Genesis, we take exactly what user provides
        amount_b_to_transfer = amount_b;

        pool_account.token_a_amount = amount_a_u128;
        pool_account.token_b_amount = amount_b_u128;
    } else {
        // --- NORMAL MODE ---
        msg!("Adding to existing Pool");

        let total_supply = lp_mint.supply as u128;
        let pool_a = pool_account.token_a_amount;
        let pool_b = pool_account.token_b_amount;

        // Formula: Required B = (Amount A * Pool B) / Pool A
        let amount_b_required = (amount_a_u128.checked_mul(pool_b).unwrap()) / pool_a;

        if amount_b_u128 < amount_b_required {
            return err!(AMMError::InsufficientFundsProvided);
        }

        // Formula: Shares = (Amount A * Total Supply) / Pool A
        liquidity_to_mint = (amount_a_u128.checked_mul(total_supply).unwrap()) / pool_a;

        // In Normal mode, we take the CALCULATED amount
        amount_b_to_transfer = amount_b_required as u64;

        pool_account.token_a_amount += amount_a_u128;
        pool_account.token_b_amount += amount_b_required;
    }

    // 3. Move the Real Money (CPI: User -> Vaults)

    // Transfer Token A
    let cpi_accounts_a = Transfer {
        from: ctx.accounts.user_a_token.to_account_info(),
        to: ctx.accounts.vault_a.to_account_info(),
        authority: ctx.accounts.signer.to_account_info(),
    };
    token::transfer(
        CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts_a),
        amount_a,
    )?;

    // Transfer Token B (The calculated amount!)
    let cpi_accounts_b = Transfer {
        from: ctx.accounts.user_b_token.to_account_info(),
        to: ctx.accounts.vault_b.to_account_info(),
        authority: ctx.accounts.signer.to_account_info(),
    };
    token::transfer(
        CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts_b),
        amount_b_to_transfer, // <--- Crucial usage
    )?;

    // 4. Print the Receipt (CPI: PDA Mints LP -> User)

    // Construct the seeds to sign for the PDA
    let token_a_key = pool_account.token_a_mint;
    let token_b_key = pool_account.token_b_mint;
    let bump = pool_account.bump;

    let signer_seeds: &[&[&[u8]]] =
        &[&[b"pool", token_a_key.as_ref(), token_b_key.as_ref(), &[bump]]];

    msg!("Minting {} LP Tokens", liquidity_to_mint);

    token::mint_to(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            MintTo {
                mint: ctx.accounts.lp_token_mint.to_account_info(),
                to: ctx.accounts.user_lp_token_account.to_account_info(),
                authority: ctx.accounts.pool_account.to_account_info(),
            },
            signer_seeds,
        ),
        liquidity_to_mint as u64,
    )?;

    Ok(())
}
