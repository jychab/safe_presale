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
        constraint = purchase_receipt.original_mint == nft_owner_nft_token_account.mint @ CustomError::MintNotAllowed,
        constraint = purchase_receipt.pool == pool.key() @CustomError::InvalidPool
    )]
    pub purchase_receipt: Box<Account<'info, PurchaseReceipt>>,
    #[account(
        constraint = nft_owner_nft_token_account.amount == 1,
        constraint = nft_owner_nft_token_account.owner == user_wallet.key(),
    )]
    pub nft_owner_nft_token_account: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        constraint = pool.authority == pool_authority.key(),
        constraint = pool.vesting_period_end.is_some() && pool.vesting_period_end.unwrap() < Clock::get()?.unix_timestamp @CustomError::VestingStillInProgress
    )]
    pub pool: Box<Account<'info, Pool>>,
    /// CHECK: Not dangerous because we are not writing or reading data from this account
    pub pool_authority: AccountInfo<'info>,
    #[account(
        init_if_needed,
        payer = user_wallet,
        associated_token::mint = lp_mint,
        associated_token::authority = pool_authority,
    )]
    pub pool_authority_token_lp: Box<InterfaceAccount<'info, TokenAccount>>,
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
pub fn handler<'info>(ctx: Context<WithdrawPoolLpToken<'info>>) -> Result<()> {
    let pool = &ctx.accounts.pool;
    let purchase_receipt = &ctx.accounts.purchase_receipt;
    let pool_identifier = pool.identifier.to_le_bytes();
    let pool_seed = &[
        POOL_PREFIX.as_bytes(),
        pool_identifier.as_ref(),
        &[pool.bump],
    ];
    let signer = &[&pool_seed[..]];

    let lp_elligible = match purchase_receipt.amount.checked_mul(pool.lp_mint_supply) {
        Some(result) => result
            .checked_div(pool.liquidity_collected)
            .ok_or(CustomError::IntegerOverflow)?,
        None => return Err(error!(CustomError::IntegerOverflow)),
    };

    let creator_fees = lp_elligible
        .checked_mul(pool.creator_fee_basis_points.try_into().unwrap())
        .ok_or(CustomError::IntegerOverflow)?
        .checked_div(10000)
        .ok_or(CustomError::IntegerOverflow)?;

    let amount_after_creator_fees = lp_elligible
        .checked_sub(creator_fees)
        .ok_or(CustomError::IntegerOverflow)?;

    transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.pool_token_lp.to_account_info(),
                to: ctx.accounts.pool_authority_token_lp.to_account_info(),
                authority: pool.to_account_info(),
            },
        )
        .with_signer(signer),
        creator_fees,
    )?;

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
        amount_after_creator_fees,
    )?;
    Ok(())
}
