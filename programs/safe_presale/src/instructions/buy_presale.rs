use crate::{error::CustomError, state::*};
use anchor_lang::{
    prelude::*,
    system_program::{transfer, Transfer},
};
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{sync_native, Mint, SyncNative, TokenAccount, TokenInterface},
};
use mpl_token_metadata::accounts::Metadata;
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
        constraint = Clock::get()?.unix_timestamp < pool.presale_time_limit @CustomError::PresaleHasEnded,
        seeds = [POOL_PREFIX.as_bytes(), pool.mint.as_ref()],
        bump = pool.bump
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

    #[account(
        seeds = ["metadata".as_bytes(), mpl_token_metadata::ID.as_ref(), nft.key().as_ref()],
        bump,
        seeds::program = mpl_token_metadata::ID
    )]
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub nft_metadata: AccountInfo<'info>,

    #[account(
        seeds = [PURCHASE_AUTHORISATION_PREFIX.as_bytes(), pool.key().as_ref(), purchase_authorisation_record.collection_mint.as_ref()],
        bump = purchase_authorisation_record.bump
    )]
    pub purchase_authorisation_record: Option<Box<Account<'info, PurchaseAuthorisationRecord>>>,

    #[account(mut)]
    pub payer: Signer<'info>,

    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(
        mut,
        address = public_keys::fee_collector::id()
    )]
    pub fee_collector: AccountInfo<'info>,

    pub system_program: Program<'info, System>,

    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

pub fn handler(ctx: Context<BuyPresaleCtx>, amount: u64) -> Result<()> {
    require!(amount > 0, CustomError::NumberCannotBeZero);

    let purchase_receipt = &mut ctx.accounts.purchase_receipt;
    let pool = &mut ctx.accounts.pool;
    let mut allowed = true;
    if pool.requires_collection {
        if let Some(authorization_record) = &ctx.accounts.purchase_authorisation_record {
            if !ctx.accounts.nft_metadata.data_is_empty() {
                let mint_metadata_data = ctx
                    .accounts
                    .nft_metadata
                    .try_borrow_mut_data()
                    .expect("Failed to borrow data");
                if ctx.accounts.nft_metadata.to_account_info().owner.key() != mpl_token_metadata::ID
                {
                    return Err(error!(CustomError::InvalidMintMetadataOwner));
                }
                let original_mint_metadata =
                    Metadata::deserialize(&mut mint_metadata_data.as_ref())
                        .expect("Failed to deserialize metadata");
                if original_mint_metadata.mint != ctx.accounts.nft.key() {
                    return Err(error!(CustomError::InvalidMintMetadata));
                }

                if original_mint_metadata.collection.is_some() {
                    let collection = original_mint_metadata.collection.unwrap();
                    allowed = collection.verified
                        && collection.key == authorization_record.collection_mint;
                }
            }
        } else {
            return Err(error!(CustomError::PurchaseAuthorisationRecordMissing));
        }
    }
    require!(allowed, CustomError::UnauthorisedCollection);

    pool.liquidity_collected = pool
        .liquidity_collected
        .checked_add(amount)
        .ok_or(CustomError::IntegerOverflow)?;

    require!(
        pool.liquidity_collected <= pool.presale_target,
        CustomError::PresaleTargetExceeded
    );

    if !purchase_receipt.is_initialized {
        purchase_receipt.bump = ctx.bumps.purchase_receipt;
        purchase_receipt.pool = pool.key();
        purchase_receipt.original_mint = ctx.accounts.nft.key();
        purchase_receipt.amount = amount;
        purchase_receipt.lp_claimed = 0;
        purchase_receipt.mint_claimed = false;
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

    transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.payer.to_account_info(),
                to: ctx.accounts.fee_collector.to_account_info(),
            },
        ),
        amount.checked_div(100).unwrap(),
    )?;

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
