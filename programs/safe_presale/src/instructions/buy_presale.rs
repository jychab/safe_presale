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
        init,
        payer = payer,
        space = PURCHASE_RECEIPT_SIZE,
        seeds = [PURCHASE_RECEIPT_PREFIX.as_bytes(), pool.key().as_ref(), original_mint.key().as_ref()],
        bump,
    )]
    pub purchase_receipt: Box<Account<'info, PurchaseReceipt>>,

    #[account(mut)]
    pub pool: Box<Account<'info, Pool>>,

    #[account(
        init_if_needed,
        payer = payer,
        associated_token::mint = wsol_mint,
        associated_token::authority = pool
    )]
    pub pool_wsol_token_account: Box<Account<'info, TokenAccount>>,

    pub wsol_mint: Box<Account<'info, Mint>>,

    pub original_mint: Box<Account<'info, Mint>>,

    #[account(
        seeds = ["metadata".as_bytes(), mpl_token_metadata::ID.as_ref(), original_mint.key().as_ref()],
        bump,
        seeds::program = mpl_token_metadata::ID
    )]
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub original_mint_metadata: AccountInfo<'info>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub system_program: Program<'info, System>,

    pub token_program: Program<'info, Token>,

    pub associated_token_program: Program<'info, AssociatedToken>,
}

pub fn handler(ctx: Context<BuyPresaleCtx>, amount: u64) -> Result<()> {
    let purchase_receipt = &mut ctx.accounts.purchase_receipt;
    let pool = &mut ctx.accounts.pool;
    purchase_receipt.bump = ctx.bumps.purchase_receipt;
    purchase_receipt.pool = pool.key();
    purchase_receipt.original_mint = ctx.accounts.original_mint.key();
    purchase_receipt.amount = amount;
    purchase_receipt.vesting_started_at = Clock::get().unwrap().unix_timestamp;

    // check allowlist
    if pool.is_closed {
        return Err(error!(CustomError::MintNotAllowedInPool));
    } else if !pool.requires_collections.is_empty() {
        let mut allowed = false;

        if !ctx.accounts.original_mint_metadata.data_is_empty() {
            let mint_metadata_data = ctx
                .accounts
                .original_mint_metadata
                .try_borrow_mut_data()
                .expect("Failed to borrow data");
            if ctx
                .accounts
                .original_mint_metadata
                .to_account_info()
                .owner
                .key()
                != mpl_token_metadata::ID
            {
                return Err(error!(CustomError::InvalidMintMetadataOwner));
            }
            let original_mint_metadata = Metadata::deserialize(&mut mint_metadata_data.as_ref())
                .expect("Failed to deserialize metadata");
            if original_mint_metadata.mint != ctx.accounts.original_mint.key() {
                return Err(error!(CustomError::InvalidMintMetadata));
            }

            if !pool.requires_collections.is_empty() && original_mint_metadata.collection.is_some()
            {
                let collection = original_mint_metadata.collection.unwrap();
                if collection.verified && pool.requires_collections.contains(&collection.key) {
                    allowed = true
                }
            }
        }

        if !allowed {
            return Err(error!(CustomError::MintNotAllowedInPool));
        }
    }
    //add liquidity to the pool
    pool.liquidity_collected = pool.liquidity_collected.checked_add(amount).unwrap();

    //transfer lamports to WSOL ata account
    transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info().clone(),
            Transfer {
                from: ctx.accounts.payer.to_account_info().clone(),
                to: ctx
                    .accounts
                    .pool_wsol_token_account
                    .to_account_info()
                    .clone(),
            },
        ),
        amount,
    )?;

    //convert sol to wSOL
    sync_native(CpiContext::new(
        ctx.accounts.token_program.to_account_info().clone(),
        SyncNative {
            account: ctx
                .accounts
                .pool_wsol_token_account
                .to_account_info()
                .clone(),
        },
    ))?;

    Ok(())
}
