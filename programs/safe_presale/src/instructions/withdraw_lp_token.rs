use crate::{error::CustomError, state::*, utils::U128};
use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, token_interface::{Mint, TransferChecked, transfer_checked, TokenAccount, TokenInterface}};

#[event_cpi]
#[derive(Accounts)]
pub struct WithdrawPoolLpToken<'info> {
    #[account(
        mut,
        close = user_wallet,
        constraint = purchase_receipt.lp_elligible.is_some() @CustomError::CheckClaimFirstBeforeClaiming,
        constraint = purchase_receipt.original_mint == nft_owner_nft_token_account.mint @ CustomError::MintNotAllowed,
        constraint = purchase_receipt.pool == pool.key() @CustomError::InvalidPool,
        seeds = [PURCHASE_RECEIPT_PREFIX.as_bytes(), purchase_receipt.pool.as_ref(), purchase_receipt.original_mint.as_ref()],
        bump = purchase_receipt.bump,
    )]
    pub purchase_receipt: Box<Account<'info, PurchaseReceipt>>,
    #[account(
        constraint = nft_owner_nft_token_account.amount == 1,
        constraint = nft_owner_nft_token_account.owner == user_wallet.key() @CustomError::InvalidSigner,
    )]
    pub nft_owner_nft_token_account: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        constraint = pool.vesting_period_end.is_some() && pool.vesting_period_end.unwrap() < Clock::get()?.unix_timestamp @CustomError::UnauthorizedAtCurrentTime,
        seeds = [POOL_PREFIX.as_bytes(), pool.mint.as_ref()],
        bump = pool.bump
    )]
    pub pool: Box<Account<'info, Pool>>,
    #[account(
        mut, 
        constraint = pool_authority_lp_token_account.owner == pool.authority,
        constraint = pool_authority_lp_token_account.mint == lp_mint.key()
    )]
    pub pool_authority_lp_token_account: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(mut)]
    pub user_wallet: Signer<'info>,
    #[account(
		init_if_needed,
        payer = user_wallet,
        associated_token::mint = lp_mint,
        associated_token::authority = user_wallet,
	)]
    pub user_lp_token_account: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
		mut,
        constraint = purchase_receipt_lp_token_account.owner == purchase_receipt.key(),
        constraint = purchase_receipt_lp_token_account.mint == lp_mint.key(),
	)]
    pub purchase_receipt_lp_token_account: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
		constraint = pool.lp_mint.is_some() && lp_mint.key() == pool.lp_mint.unwrap()
	)]
    pub lp_mint: Box<InterfaceAccount<'info, Mint>>,
    /// Program to create the position manager state account
    pub system_program: Program<'info, System>,
    /// Program to create mint account and mint tokens
    pub token_program: Interface<'info, TokenInterface>,
    /// Program to create an ATA for receiving position NFT
    pub associated_token_program: Program<'info, AssociatedToken>,
}
pub fn handler<'info>(ctx: Context<WithdrawPoolLpToken<'info>>) -> Result<()> {
    let pool = &ctx.accounts.pool;
    let purchase_receipt = &mut ctx.accounts.purchase_receipt;
    let lp_elligible = purchase_receipt.lp_elligible.unwrap();

    let creator_fees = U128::from(lp_elligible)
        .checked_mul(pool.creator_fee_basis_points.try_into().unwrap())
        .and_then(|result| result.checked_div(U128::from(10000)))
        .and_then(|result| Some(result.as_u64()))
        .ok_or(CustomError::IntegerOverflow)?;

    let amount_after_creator_fees = lp_elligible
        .checked_sub(creator_fees)
        .ok_or(CustomError::IntegerOverflow)?;

    let purchase_seed = &[
            PURCHASE_RECEIPT_PREFIX.as_bytes(), 
            purchase_receipt.pool.as_ref(), 
            purchase_receipt.original_mint.as_ref(),
            &[purchase_receipt.bump],
        ];
    let signer = &[&purchase_seed[..]];

    transfer_checked(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            TransferChecked {
                mint: ctx.accounts.lp_mint.to_account_info(),
                from: ctx.accounts.purchase_receipt_lp_token_account.to_account_info(),
                to: ctx.accounts.pool_authority_lp_token_account.to_account_info(),
                authority: pool.to_account_info(),
            },
        )
        .with_signer(signer),
        creator_fees,
        ctx.accounts.lp_mint.decimals,
    )?;

    transfer_checked(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            TransferChecked {
                mint: ctx.accounts.lp_mint.to_account_info(),
                from: ctx.accounts.purchase_receipt_lp_token_account.to_account_info(),
                to: ctx.accounts.user_lp_token_account.to_account_info(),
                authority: pool.to_account_info(),
            },
        )
        .with_signer(signer),
        amount_after_creator_fees,
        ctx.accounts.lp_mint.decimals
    )?;

    emit_cpi!(WithdrawLpTokenEvent {
        payer: ctx.accounts.user_wallet.key(),
        pool: pool.key(),
        original_mint: purchase_receipt.original_mint,
        amount_lp_withdrawn: amount_after_creator_fees,
        lp_mint: pool.lp_mint.unwrap(),
    });

    Ok(())
}
