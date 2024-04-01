use crate::{error::CustomError, state::*};
use anchor_lang::{
    prelude::*,
    system_program::{transfer, Transfer},
};
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{sync_native, Mint, SyncNative, Token, TokenAccount},
};
use mpl_token_metadata::{self, accounts::Metadata};

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
        constraint = pool.allow_purchase @CustomError::PresaleHasEnded,
    )]
    pub pool: Box<Account<'info, Pool>>,

    #[account(
        init_if_needed,
        payer = payer,
        associated_token::mint = wsol_mint,
        associated_token::authority = pool
    )]
    pub pool_wsol_token_account: Box<Account<'info, TokenAccount>>,

    #[account(
        address = public_keys::wsol::id()
    )]
    pub wsol_mint: Box<Account<'info, Mint>>,

    #[account(
        constraint = nft.supply == 1 @CustomError::NftIsNotNonFungible
    )]
    pub nft: Box<Account<'info, Mint>>,

    #[account(
        seeds = ["metadata".as_bytes(), mpl_token_metadata::ID.as_ref(), nft.key().as_ref()],
        bump,
        seeds::program = mpl_token_metadata::ID
    )]
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub nft_metadata: AccountInfo<'info>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub system_program: Program<'info, System>,

    pub token_program: Program<'info, Token>,

    pub associated_token_program: Program<'info, AssociatedToken>,
}

pub fn handler(ctx: Context<BuyPresaleCtx>, amount: u64) -> Result<()> {
    msg!("Initializing purchase receipt");
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

    msg!("Checking allowlist");
    if !pool.requires_collections.is_empty() {
        let mut allowed = false;

        if !ctx.accounts.nft_metadata.data_is_empty() {
            let mint_metadata_data = ctx
                .accounts
                .nft_metadata
                .try_borrow_mut_data()
                .expect("Failed to borrow data");
            if ctx.accounts.nft_metadata.to_account_info().owner.key() != mpl_token_metadata::ID {
                return Err(error!(CustomError::InvalidMintMetadataOwner));
            }
            let nft_metadata = Metadata::deserialize(&mut mint_metadata_data.as_ref())
                .expect("Failed to deserialize metadata");
            if nft_metadata.mint != ctx.accounts.nft.key() {
                return Err(error!(CustomError::InvalidMintMetadata));
            }

            if !pool.requires_collections.is_empty() && nft_metadata.collection.is_some() {
                let collection = nft_metadata.collection.unwrap();
                if collection.verified && pool.requires_collections.contains(&collection.key) {
                    allowed = true
                }
            }
        }

        if !allowed {
            return Err(error!(CustomError::MintNotAllowedInPool));
        }
    }
    msg!("Adding liquidity to the pool");
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

    Ok(())
}
