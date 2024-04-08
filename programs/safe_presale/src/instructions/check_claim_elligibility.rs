use crate::error::CustomError;
use crate::state::*;
use crate::utils::U128;
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
        constraint = pool.launched @CustomError::PresaleIsStillOngoing
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
    purchase_receipt.lp_elligible = Some(lp_elligible.as_u64());
    purchase_receipt.mint_elligible = Some(mint_elligible);

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
