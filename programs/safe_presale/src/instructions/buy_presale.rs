use crate::{error::CustomError, state::*};
use anchor_lang::{
    prelude::*,
    system_program::{transfer, Transfer},
};
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{sync_native, Mint, SyncNative, TokenAccount, TokenInterface},
};
#[event_cpi]
#[derive(Accounts)]
pub struct BuyPresaleCtx<'info> {
    #[account(
        init_if_needed,
        payer = payer,
        space = PURCHASE_RECEIPT_SIZE,
        seeds = [PURCHASE_RECEIPT_PREFIX.as_bytes(), pool.key().as_ref(), nft.key().as_ref()],
        bump,
    )]
    pub purchase_receipt: Box<Account<'info, PurchaseReceipt>>,

    #[account(
        mut,
        constraint = !pool.launched @CustomError::TokenHasLaunched,
        constraint = Clock::get()?.unix_timestamp < pool.presale_time_limit @CustomError::PresaleHasEnded
    )]
    pub pool: Box<Account<'info, Pool>>,
    #[account(
        init_if_needed,
        payer = payer,
        associated_token::mint = wsol_mint,
        associated_token::authority = pool
    )]
    pub pool_wsol_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        constraint = nft_owner_nft_token_account.amount == 1,
        constraint = nft_owner_nft_token_account.mint == nft.key(),
        constraint = nft_owner_nft_token_account.owner == payer.key() @CustomError::InvalidSigner
    )]
    pub nft_owner_nft_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        address = public_keys::wsol::id()
    )]
    pub wsol_mint: Box<InterfaceAccount<'info, Mint>>,
    #[account(
        constraint = nft.supply == 1 @CustomError::NftIsNotNonFungible
    )]
    pub nft: Box<InterfaceAccount<'info, Mint>>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub system_program: Program<'info, System>,

    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

pub fn handler(ctx: Context<BuyPresaleCtx>, amount: u64) -> Result<()> {
    if amount == 0 {
        return Err(error!(CustomError::NumberCannotBeZero));
    }
    let purchase_receipt = &mut ctx.accounts.purchase_receipt;
    let pool = &mut ctx.accounts.pool;
    if !purchase_receipt.is_initialized {
        purchase_receipt.bump = ctx.bumps.purchase_receipt;
        purchase_receipt.pool = pool.key();
        purchase_receipt.original_mint = ctx.accounts.nft.key();
        purchase_receipt.amount = amount;
        purchase_receipt.mint_claimed = 0;
        purchase_receipt.is_initialized = true;
    } else {
        purchase_receipt.amount = purchase_receipt
            .amount
            .checked_add(amount)
            .ok_or(CustomError::IntegerOverflow)?;
    }
    if pool.max_amount_per_purchase.is_some()
        && purchase_receipt.amount > pool.max_amount_per_purchase.unwrap()
    {
        return Err(error!(CustomError::AmountPurchaseExceeded));
    }

    pool.liquidity_collected = pool
        .liquidity_collected
        .checked_add(amount)
        .ok_or(CustomError::IntegerOverflow)?;

    transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.payer.to_account_info(),
                to: ctx.accounts.pool_wsol_token_account.to_account_info(),
            },
        ),
        amount,
    )?;
    sync_native(CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        SyncNative {
            account: ctx.accounts.pool_wsol_token_account.to_account_info(),
        },
    ))?;

    emit_cpi!(PurchasedPresaleEvent {
        payer: ctx.accounts.payer.key(),
        amount: amount,
        pool: pool.key(),
        original_mint: ctx.accounts.nft.key(),
    });

    Ok(())
}
