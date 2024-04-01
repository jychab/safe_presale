use crate::{error::CustomError, state::*};
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{transfer, Token, Transfer},
    token_interface::{Mint, TokenAccount},
};

#[derive(Accounts)]
pub struct WithdrawPoolLpToken<'info> {
    #[account(
        constraint = pool.authority == user_wallet.key(),
        constraint = pool.vesting_period_end.is_some() && pool.vesting_period_end.unwrap() < Clock::get()?.unix_timestamp @CustomError::VestingStillInProgress
    )]
    pub pool: Box<Account<'info, Pool>>,
    #[account(mut)]
    pub user_wallet: Signer<'info>,
    #[account(
		init_if_needed,
        payer = user_wallet,
        associated_token::mint = lp_mint,
        associated_token::authority = user_wallet,
	)]
    pub user_token_lp: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
		init_if_needed,
        payer = user_wallet,
        associated_token::mint = lp_mint,
        associated_token::authority = pool,
	)]
    pub pool_token_lp: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
		constraint = pool.lp_mint.is_some() && lp_mint.key() == pool.lp_mint.unwrap()
	)]
    pub lp_mint: Box<InterfaceAccount<'info, Mint>>,
    /// Program to create the position manager state account
    pub system_program: Program<'info, System>,
    /// Program to create mint account and mint tokens
    pub token_program: Program<'info, Token>,
    /// Program to create an ATA for receiving position NFT
    pub associated_token_program: Program<'info, AssociatedToken>,
}
pub fn handler<'info>(ctx: Context<WithdrawPoolLpToken<'info>>, amount: u64) -> Result<()> {
    let pool = &ctx.accounts.pool;
    let pool_identifier = pool.identifier.to_le_bytes();
    let pool_seed = &[
        POOL_PREFIX.as_bytes(),
        pool_identifier.as_ref(),
        &[pool.bump],
    ];
    let signer = &[&pool_seed[..]];
    transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.pool_token_lp.to_account_info(),
                to: ctx.accounts.user_token_lp.to_account_info(),
                authority: pool.to_account_info(),
            },
        )
        .with_signer(signer),
        amount,
    )?;
    Ok(())
}
