use crate::error::CustomError;
use crate::state::*;
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked},
};

#[event_cpi]
#[derive(Accounts)]
pub struct ClaimRewardCtx<'info> {
    #[account(
        mut,
        constraint = !purchase_receipt.mint_claimed, 
        constraint = purchase_receipt.mint_elligible.is_some() @CustomError::CheckClaimFirstBeforeClaiming,
        constraint = purchase_receipt.original_mint == nft_owner_nft_token_account.mint @ CustomError::MintNotAllowed,
    )]
    pub purchase_receipt: Box<Account<'info, PurchaseReceipt>>,

    #[account(
        mut,
        constraint = purchase_receipt_reward_token_account.owner == purchase_receipt.key(),
        constraint = purchase_receipt_reward_token_account.mint == pool.mint,
    )]
    pub purchase_receipt_reward_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        constraint = pool.key() == purchase_receipt.pool @CustomError::InvalidPool,
    )]
    pub pool: Box<Account<'info, Pool>>,

    #[account(
        constraint = nft_owner_nft_token_account.amount == 1,
        constraint = nft_owner_nft_token_account.owner == nft_owner.key()
    )]
    pub nft_owner_nft_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        init_if_needed,
        payer = payer,
        associated_token::mint = reward_mint,
        associated_token::authority = nft_owner,
    )]
    pub nft_owner_reward_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    ///CHECK: Contraint is checked by other accounts
    pub nft_owner: AccountInfo<'info>,

    #[account(
        seeds = ["metadata".as_bytes(), mpl_token_metadata::ID.as_ref(), purchase_receipt.original_mint.as_ref()],
        bump,
        seeds::program = mpl_token_metadata::ID
    )]
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub nft_metadata: AccountInfo<'info>,

    #[account(
        constraint =  pool.mint == reward_mint.key() @ CustomError::InvalidRewardMint
    )]
    pub reward_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<ClaimRewardCtx>) -> Result<()> {
    let purchase_receipt = &mut ctx.accounts.purchase_receipt;
    // Delegated Claim Criteria
    // 1. Only allow delegated claiming if the nfts are frozen to the owner's wallet.
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
    //         if collection.verified && &collection.key == &collection::id() && ctx.accounts.nft_owner_nft_token_account.is_frozen(){ // only allow staked nfts to do delegated claiming
    //             allowed = true;
    //         }
    //     }
    // }
    if !allowed {
        return Err(error!(CustomError::InvalidSigner));
    }
    let mint_elligible = purchase_receipt.mint_elligible.unwrap();
    purchase_receipt.mint_claimed = true;
    let purchase_seed = &[
        PURCHASE_RECEIPT_PREFIX.as_bytes(),
        purchase_receipt.pool.as_ref(),
        purchase_receipt.original_mint.as_ref(),
        &[purchase_receipt.bump],
    ];
    let signer = &[&purchase_seed[..]];

    //transfer to nft owner
    transfer_checked(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            TransferChecked {
                mint: ctx.accounts.reward_mint.to_account_info(),
                from: ctx
                    .accounts
                    .purchase_receipt_reward_token_account
                    .to_account_info(),
                to: ctx.accounts.nft_owner_reward_token_account.to_account_info(),
                authority: purchase_receipt.to_account_info(),
            },
        )
        .with_signer(signer),
        mint_elligible,
        ctx.accounts.reward_mint.decimals,
    )?;

    emit_cpi!(ClaimRewardEvent {
        payer: ctx.accounts.payer.key(),
        pool: purchase_receipt.pool,
        mint_elligible: mint_elligible,
        original_mint_owner: ctx.accounts.nft_owner.key(),
        original_mint: purchase_receipt.original_mint,
    });

    Ok(())
}
