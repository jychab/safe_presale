use crate::error::CustomError;
use crate::state::*;
use crate::utils::U128;
use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token_interface::{TokenAccount, TokenInterface, Mint, transfer_checked, TransferChecked};

#[event_cpi]
#[derive(Accounts)]
pub struct CheckClaimCtx<'info> {
    #[account(
        mut,
        constraint = purchase_receipt.mint_elligible.is_none() || purchase_receipt.lp_elligible.is_none() @CustomError::ClaimedAlreadyChecked,
        seeds = [PURCHASE_RECEIPT_PREFIX.as_bytes(), purchase_receipt.pool.as_ref(), purchase_receipt.original_mint.as_ref()],
        bump = purchase_receipt.bump,
    )]
    pub purchase_receipt: Box<Account<'info, PurchaseReceipt>>,

    #[account(
        init_if_needed,
        payer = payer,  
        associated_token::mint = lp_mint,
        associated_token::authority = purchase_receipt,
    )]
    pub purchase_receipt_lp_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        init_if_needed,
        payer = payer,  
        associated_token::mint = reward_mint,
        associated_token::authority = purchase_receipt,
    )]
    pub purchase_receipt_mint_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        mut,
        constraint = pool_lp_token_account.owner == pool.key(),
        constraint = pool_lp_token_account.mint == pool.lp_mint.unwrap(),
    )]
    pub pool_lp_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        mut,
        constraint = pool_mint_token_account.owner == pool.key(),
        constraint = pool_mint_token_account.mint == pool.mint,
    )]
    pub pool_mint_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        constraint = reward_mint.key() == pool.mint @CustomError::InvalidRewardMint,
    )]
    pub reward_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        constraint = lp_mint.key() == pool.lp_mint.unwrap() @CustomError::InvalidLpMint,
    )]
    pub lp_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        constraint = pool.key() == purchase_receipt.pool @CustomError::InvalidPool,
        constraint = pool.launched @CustomError::PresaleIsStillOngoing,
        seeds = [POOL_PREFIX.as_bytes(), pool.mint.as_ref()],
        bump = pool.bump
    )]
    pub pool: Box<Account<'info, Pool>>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<CheckClaimCtx>) -> Result<()> {
    let pool = &ctx.accounts.pool;
    let lp_mint_supply = pool.lp_mint_supply.unwrap();
    let liquidity_collected = pool.liquidity_collected;
    let vested_supply = pool.vested_supply;
    let purchase_receipt = &mut ctx.accounts.purchase_receipt;
    //after creator fees
    let lp_elligible =
        match U128::from(purchase_receipt.amount).checked_mul(U128::from(lp_mint_supply)) {
            Some(result) => result
                .checked_div(U128::from(liquidity_collected))
                .ok_or(CustomError::IntegerOverflow)?,
            None => return Err(error!(CustomError::IntegerOverflow)),
        };
    let creator_fees = lp_elligible
        .checked_mul(pool.creator_fee_basis_points.try_into().unwrap())
        .and_then(|result| result.checked_div(10000.try_into().unwrap()))
        .ok_or(CustomError::IntegerOverflow)?;

    let amount_after_creator_fees = lp_elligible
        .checked_sub(creator_fees)
        .and_then(|result| Some(result.as_u64()))
        .ok_or(CustomError::IntegerOverflow)?;

    let mint_elligible =
        match U128::from(purchase_receipt.amount).checked_mul(U128::from(vested_supply)) {
            Some(result) => result
                .checked_div(U128::from(liquidity_collected))
                .and_then(|result| Some(result.as_u64()))
                .ok_or(CustomError::IntegerOverflow)?,
            None => return Err(error!(CustomError::IntegerOverflow)),
        };
    purchase_receipt.mint_elligible = Some(mint_elligible);
    purchase_receipt.lp_elligible = Some(lp_elligible.as_u64());

    let pool_seed = &[
        POOL_PREFIX.as_bytes(),
        pool.mint.as_ref(),
        &[pool.bump],
    ];
    let signer = &[&pool_seed[..]];

    transfer_checked(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            TransferChecked {
                mint: ctx.accounts.reward_mint.to_account_info(),
                from: ctx.accounts.pool_mint_token_account.to_account_info(),
                to: ctx.accounts.purchase_receipt_mint_token_account.to_account_info(),
                authority: pool.to_account_info(),
            },
        )
        .with_signer(signer),
        mint_elligible,
        ctx.accounts.reward_mint.decimals,
    )?;

    transfer_checked(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            TransferChecked {
                mint: ctx.accounts.lp_mint.to_account_info(),
                from: ctx.accounts.pool_lp_token_account.to_account_info(),
                to: ctx.accounts.purchase_receipt_lp_token_account.to_account_info(),
                authority: pool.to_account_info(),
            },
        )
        .with_signer(signer),
        lp_elligible.as_u64(),
        ctx.accounts.lp_mint.decimals,
    )?;

    emit_cpi!(CheckClaimEvent {
        payer: ctx.accounts.payer.key(),
        pool: pool.key(),
        original_mint: purchase_receipt.original_mint.key(),
        mint_elligible: purchase_receipt.mint_elligible.unwrap(),
        lp_elligible: purchase_receipt.lp_elligible.unwrap(),
        lp_elligible_after_fees: amount_after_creator_fees,
    });

    Ok(())
}
