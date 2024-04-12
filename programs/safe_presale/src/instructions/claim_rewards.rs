use crate::error::CustomError;
use crate::state::*;
use crate::utils::Calculator;
use crate::utils::U128;
use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, token_interface::{Mint, transfer_checked, TransferChecked, TokenAccount, TokenInterface}};
use mpl_token_metadata::accounts::Metadata;
use state::public_keys::collection;

#[event_cpi]
#[derive(Accounts)]
pub struct ClaimRewardsCtx<'info> {
    #[account(
        mut,
        constraint = purchase_receipt.mint_elligible.is_some() @CustomError::CheckClaimFirstBeforeClaiming,
        constraint = purchase_receipt.original_mint == nft_owner_nft_token_account.mint @ CustomError::MintNotAllowed,
        seeds = [PURCHASE_RECEIPT_PREFIX.as_bytes(), purchase_receipt.pool.as_ref(), purchase_receipt.original_mint.as_ref()],
        bump = purchase_receipt.bump,
    )]
    pub purchase_receipt: Box<Account<'info, PurchaseReceipt>>,

    #[account(
        mut,
        constraint = purchase_receipt_mint_token_account.owner == purchase_receipt.key(),
        constraint = purchase_receipt_mint_token_account.mint == reward_mint.key(),
    )]
    pub purchase_receipt_mint_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        constraint = pool.key() == purchase_receipt.pool @CustomError::InvalidPool,
        constraint = pool.launched @CustomError::PresaleIsStillOngoing,
        seeds = [POOL_PREFIX.as_bytes(), pool.mint.as_ref()],
        bump = pool.bump
    )]
    pub pool: Box<Account<'info, Pool>>,

    #[account(
        constraint = nft_owner_nft_token_account.amount == 1,
        constraint = nft_owner_nft_token_account.owner == nft_owner.key()
    )]
    pub nft_owner_nft_token_account:  Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        init_if_needed,
        payer = payer,
        associated_token::mint = reward_mint,
        associated_token::authority = nft_owner,
    )]
    pub nft_owner_reward_mint_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

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
        mut, 
        constraint = reward_mint.key() == pool.mint @ CustomError::InvalidRewardMint
    )]
    pub reward_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<ClaimRewardsCtx>,
) -> Result<()> {
    let purchase_receipt = &mut ctx.accounts.purchase_receipt;
    // Delegated Claim Criteria
    // 1. Only allow delegated claiming if the nfts are frozen to the owner's wallet.
    let mut allowed = ctx.accounts.nft_owner.key() == ctx.accounts.payer.key();
    if !ctx.accounts.nft_metadata.data_is_empty() {
        let mint_metadata_data = ctx
            .accounts
            .nft_metadata
            .try_borrow_mut_data()
            .expect("Failed to borrow data");
        if ctx.accounts.nft_metadata.to_account_info().owner.key() != mpl_token_metadata::ID {
            return Err(error!(CustomError::InvalidMintMetadataOwner));
        }
        let original_mint_metadata = Metadata::deserialize(&mut mint_metadata_data.as_ref())
            .expect("Failed to deserialize metadata");
        if original_mint_metadata.mint != purchase_receipt.original_mint {
            return Err(error!(CustomError::InvalidMintMetadata));
        }

        if original_mint_metadata.collection.is_some() {
            let collection = original_mint_metadata.collection.unwrap();
            if collection.verified && &collection.key == &collection::id() && ctx.accounts.nft_owner_nft_token_account.is_frozen(){ // only allow staked nfts to do delegated claiming
                allowed = true;
            }
        }
    }
    if !allowed {
        return Err(error!(CustomError::InvalidSigner));
    }

    let pool = &ctx.accounts.pool;
    let vesting_started_at = pool.vesting_started_at.unwrap();
    let vesting_period_end = pool.vesting_period_end.unwrap();
    let vesting_period = pool.vesting_period;
    let mint_elligible_to_claim = purchase_receipt.mint_elligible.unwrap();

    let current_time = Clock::get()?.unix_timestamp;
    let mint_claimable ;
    if current_time >= vesting_period_end {
        if mint_elligible_to_claim == purchase_receipt.mint_claimed {
            return Err(error!(CustomError::MaximumAmountClaimed))
        }
        mint_claimable = mint_elligible_to_claim.checked_sub(purchase_receipt.mint_claimed).ok_or(CustomError::IntegerOverflow)?;
    }else{
        // all using unix_timestamp
        let last_claimed_at = purchase_receipt.last_claimed_at.unwrap_or(vesting_started_at);
        
        let duration_since_last_claimed = Calculator::to_u64_from_i64(current_time.checked_sub(last_claimed_at).ok_or(CustomError::IntegerOverflow)?)?;
        mint_claimable = U128::from(duration_since_last_claimed)
            .checked_mul(U128::from(mint_elligible_to_claim))
            .and_then(|result| result.checked_div(vesting_period.try_into().unwrap()))
            .and_then(|result| Some(result.as_u64()))
            .ok_or(CustomError::IntegerOverflow)?;
    }


    //update mint_claimed
    purchase_receipt.mint_claimed = purchase_receipt.mint_claimed.checked_add(mint_claimable).ok_or(CustomError::IntegerOverflow)?;
    //update last_claimed_at
    purchase_receipt.last_claimed_at =  Some(current_time);

    let purchase_seed = &[
            PURCHASE_RECEIPT_PREFIX.as_bytes(), 
            purchase_receipt.pool.as_ref(), 
            purchase_receipt.original_mint.as_ref(),
            &[purchase_receipt.bump],
        ];
    let signer = &[&purchase_seed[..]];

    transfer_checked(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            TransferChecked {
                mint: ctx.accounts.reward_mint.to_account_info(),
                from: ctx.accounts.purchase_receipt_mint_token_account.to_account_info(),
                to: ctx.accounts.nft_owner_reward_mint_token_account.to_account_info(),
                authority: purchase_receipt.to_account_info(),
            },
        )
        .with_signer(signer),
        mint_claimable,
        ctx.accounts.reward_mint.decimals,
    )?;

    emit_cpi!(ClaimRewardsEvent {
        payer: ctx.accounts.payer.key(),
        pool: ctx.accounts.pool.key(),
        mint_claimed: mint_claimable,
        last_claimed_at: purchase_receipt.last_claimed_at.unwrap(),
        original_mint: purchase_receipt.original_mint,
        original_mint_owner: ctx.accounts.nft_owner.key(),
    });
        
    Ok(())
}
