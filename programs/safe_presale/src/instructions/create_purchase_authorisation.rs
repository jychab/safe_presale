use crate::error::CustomError;
use crate::state::*;
use anchor_lang::prelude::*;

#[event_cpi]
#[derive(Accounts)]
#[instruction(collection_mint:Pubkey)]
pub struct CreatePurchaseAuthorizationCtx<'info> {
    #[account(
        init,
        space = PURCHASE_AUTHORISATION_SIZE,
        payer = payer,
        seeds = [PURCHASE_AUTHORISATION_PREFIX.as_bytes(), pool.key().as_ref(), collection_mint.as_ref()],
        bump,
    )]
    pub purchase_authorisation_record: Box<Account<'info, PurchaseAuthorisationRecord>>,

    #[account(
        constraint = pool.vesting_started_at.is_none() @CustomError::TokenHasLaunched,
        constraint = pool.requires_collection,
        constraint = pool.authority == payer.key(),
        seeds = [POOL_PREFIX.as_bytes(), pool.mint.as_ref()],
        bump = pool.bump
    )]
    pub pool: Box<Account<'info, Pool>>,

    #[account(mut)]
    pub payer: Signer<'info>,

    /// CHECK: This is not dangerous because we don't read or write from this account
    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<CreatePurchaseAuthorizationCtx>,
    collection_mint: Pubkey,
) -> Result<()> {
    let purchase_authorisation_record = &mut ctx.accounts.purchase_authorisation_record;
    purchase_authorisation_record.bump = ctx.bumps.purchase_authorisation_record;
    purchase_authorisation_record.collection_mint = collection_mint;
    purchase_authorisation_record.pool = ctx.accounts.pool.key();

    emit_cpi!(CreatePurchaseAuthorisationEvent {
        payer: ctx.accounts.payer.key(),
        pool: ctx.accounts.pool.key(),
        collection_mint: collection_mint,
    });

    Ok(())
}
