use crate::error::CustomError;
use crate::state::*;
use anchor_lang::prelude::*;

#[event_cpi]
#[derive(Accounts)]
pub struct CheckClaimCtx<'info> {
    #[account(
        mut,
        constraint = purchase_receipt.mint_elligible.is_none() || purchase_receipt.lp_elligible.is_none() @CustomError::ClaimedAlreadyChecked,
    )]
    pub purchase_receipt: Box<Account<'info, PurchaseReceipt>>,

    #[account(
        constraint = pool.key() == purchase_receipt.pool @CustomError::InvalidPool,
        constraint = !pool.allow_purchase @CustomError::PresaleIsStillOngoing
    )]
    pub pool: Box<Account<'info, Pool>>,

    #[account(mut)]
    pub payer: Signer<'info>,
}

pub fn handler(ctx: Context<CheckClaimCtx>) -> Result<()> {
    let pool = &ctx.accounts.pool;
    let lp_mint_supply = pool.lp_mint_supply.unwrap();
    let liquidity_collected = pool.liquidity_collected;
    let vested_supply = pool.vested_supply;
    let purchase_receipt = &mut ctx.accounts.purchase_receipt;
    let lp_elligible = match purchase_receipt.amount.checked_mul(lp_mint_supply) {
        Some(result) => result
            .checked_div(liquidity_collected)
            .ok_or(CustomError::IntegerOverflow)?,
        None => return Err(error!(CustomError::IntegerOverflow)),
    };
    let mint_elligible = match purchase_receipt.amount.checked_mul(vested_supply) {
        Some(result) => result
            .checked_div(liquidity_collected)
            .ok_or(CustomError::IntegerOverflow)?,
        None => return Err(error!(CustomError::IntegerOverflow)),
    };
    purchase_receipt.lp_elligible = Some(lp_elligible);
    purchase_receipt.mint_elligible = Some(mint_elligible);

    emit_cpi!(CheckClaimEvent {
        payer: ctx.accounts.payer.key(),
        pool: pool.key(),
        original_mint: purchase_receipt.original_mint.key(),
        mint_elligible: mint_elligible,
        lp_elligibile: lp_elligible,
    });

    Ok(())
}
