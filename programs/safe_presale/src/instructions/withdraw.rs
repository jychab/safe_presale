use crate::{error::CustomError, state::*};
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked},
};

#[event_cpi]
#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(
        mut,
        close = payer,
        constraint = purchase_receipt.original_mint == nft_owner_nft_token_account.mint @ CustomError::MintNotAllowed,
        constraint = purchase_receipt.pool == pool.key() @CustomError::InvalidPool,
    )]
    pub purchase_receipt: Box<Account<'info, PurchaseReceipt>>,
    #[account(
        constraint = nft_owner_nft_token_account.amount == 1,
        constraint = nft_owner_nft_token_account.owner == nft_owner.key(),
    )]
    pub nft_owner_nft_token_account: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        constraint = pool.vesting_started_at.is_none() @CustomError::TokenHasLaunched,
    )]
    pub pool: Box<Account<'info, Pool>>,
    #[account(
		init_if_needed,
        payer = payer,
        associated_token::mint = quote_mint,
        associated_token::authority = nft_owner,
	)]
    pub nft_owner_quote_mint_token_account: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
		init_if_needed,
        payer = payer,
        associated_token::mint = quote_mint,
        associated_token::authority = pool,
	)]
    pub pool_quote_mint_token_account: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
		constraint = quote_mint.key() == pool.quote_mint,
	)]
    pub quote_mint: Box<InterfaceAccount<'info, Mint>>,
    ///CHECK: Contraint is checked by other accounts
    #[account(mut)]
    pub nft_owner: AccountInfo<'info>,
    #[account(
        seeds = ["metadata".as_bytes(), mpl_token_metadata::ID.as_ref(), purchase_receipt.original_mint.as_ref()],
        bump,
        seeds::program = mpl_token_metadata::ID
    )]
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub nft_metadata: AccountInfo<'info>,
    #[account(mut)]
    pub payer: Signer<'info>,
    /// Program to create the position manager state account
    pub system_program: Program<'info, System>,
    /// Program to create mint account and mint tokens
    pub token_program: Interface<'info, TokenInterface>,
    /// Program to create an ATA for receiving position NFT
    pub associated_token_program: Program<'info, AssociatedToken>,
}
pub fn handler<'info>(ctx: Context<Withdraw<'info>>) -> Result<()> {
    let pool = &ctx.accounts.pool;
    let purchase_receipt = &mut ctx.accounts.purchase_receipt;
    // Withdrawal criteria
    // 1. Only allow withdrawal after presale has ended.
    // 2. If presale target amount is not met, withdrawal is allowed immediately.
    // 3. If presale target amount is met, only allow withdrawal after the creator failed to launch the project after 7 days.
    let current_time = Clock::get()?.unix_timestamp;
    if current_time < pool.presale_time_limit + GRACE_PERIOD {
        if current_time < pool.presale_time_limit {
            return Err(error!(CustomError::UnauthorizedAtCurrentTime));
        }
        if pool.presale_target == pool.liquidity_collected {
            return Err(error!(CustomError::WaitingForCreatorToLaunch));
        }
    }
    // Delegated withdrawal criteria
    // 1. Only allow delegated withdrawal if the nfts are frozen to the owner's wallet.
    let allowed = ctx.accounts.nft_owner.key() == ctx.accounts.payer.key();
    // if !ctx.accounts.nft_metadata.data_is_empty() {
    //     let mint_metadata_data = ctx
    //         .accounts
    //         .nft_metadata
    //         .try_borrow_mut_data()
    //         .expect("Failed to borrow data");
    //     if ctx.accounts.nft_metadata.to_account_info().owner.key() != mpl_token_metadata::ID {
    //         return Err(error!(CustomError::InvalidMintMetadataOwner));
    //     }
    //     let original_mint_metadata = Metadata::deserialize(&mut mint_metadata_data.as_ref())
    //         .expect("Failed to deserialize metadata");
    //     if original_mint_metadata.mint != purchase_receipt.original_mint {
    //         return Err(error!(CustomError::InvalidMintMetadata));
    //     }

    //     if original_mint_metadata.collection.is_some() {
    //         let collection = original_mint_metadata.collection.unwrap();
    //         if collection.verified
    //             && &collection.key == &collection::id()
    //             && ctx.accounts.nft_owner_nft_token_account.is_frozen()
    //         {
    //             allowed = true;
    //         }
    //     }
    // }

    if !allowed {
        return Err(error!(CustomError::InvalidSigner));
    }

    let pool_seed = &[POOL_PREFIX.as_bytes(), pool.mint.as_ref(), &[pool.bump]];
    let signer = &[&pool_seed[..]];

    transfer_checked(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            TransferChecked {
                mint: ctx.accounts.quote_mint.to_account_info(),
                from: ctx.accounts.pool_quote_mint_token_account.to_account_info(),
                to: ctx
                    .accounts
                    .nft_owner_quote_mint_token_account
                    .to_account_info(),
                authority: pool.to_account_info(),
            },
        )
        .with_signer(signer),
        purchase_receipt.amount,
        ctx.accounts.quote_mint.decimals,
    )?;

    emit_cpi!(WithdrawEvent {
        payer: ctx.accounts.payer.key(),
        pool: pool.key(),
        original_mint: purchase_receipt.original_mint,
        amount_withdrawn: purchase_receipt.amount,
        original_mint_owner: ctx.accounts.nft_owner.key(),
    });

    Ok(())
}
