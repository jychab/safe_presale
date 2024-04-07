use crate::error::CustomError;
use crate::state::*;
use crate::utils::Calculator;
use crate::utils::U128;
use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, token_interface::{Mint, MintTo, mint_to, TokenAccount, TokenInterface}};

#[event_cpi]
#[derive(Accounts)]
pub struct ClaimRewardsCtx<'info> {
    #[account(
        mut,
        constraint = purchase_receipt.mint_elligible.is_some() @CustomError::CheckClaimFirstBeforeClaiming,
        constraint = purchase_receipt.original_mint == nft.key()@ CustomError::MintNotAllowed
    )]
    pub purchase_receipt: Box<Account<'info, PurchaseReceipt>>,

    #[account(
        constraint = pool.key() == purchase_receipt.pool @CustomError::InvalidPool,
        constraint = !pool.allow_purchase @CustomError::PresaleIsStillOngoing
    )]
    pub pool: Box<Account<'info, Pool>>,

    #[account(
        constraint = nft_owner_nft_token_account.amount == 1,
        constraint = nft_owner_nft_token_account.mint == nft.key(),
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
        constraint = nft.supply == 1 @CustomError::NftIsNotNonFungible
    )]
    pub nft: Box<InterfaceAccount<'info, Mint>>,

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
    let pool = &ctx.accounts.pool;
    let vesting_started_at = pool.vesting_started_at.unwrap();
    let vesting_period_end = pool.vesting_period_end.unwrap();
    let vesting_period = pool.vesting_period;
    let purchase_receipt = &mut ctx.accounts.purchase_receipt;
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

    let pool_identifier = pool.identifier.to_le_bytes();
    let pool_seed = &[
            POOL_PREFIX.as_bytes(),
            pool_identifier.as_ref(),
            &[pool.bump],
        ];
    let signer = &[&pool_seed[..]];

    mint_to(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(), 
            MintTo {
                mint: ctx.accounts.reward_mint.to_account_info(),
                to: ctx
                    .accounts
                    .nft_owner_reward_mint_token_account
                    .to_account_info(),
                authority: pool.to_account_info(),
            }
        ).with_signer(signer),
        mint_claimable
    )?;   

    emit_cpi!(ClaimRewardsEvent {
        payer: ctx.accounts.payer.key(),
        pool: ctx.accounts.pool.key(),
        mint_claimed: mint_claimable,
        last_claimed_at: purchase_receipt.last_claimed_at.unwrap(),
        original_mint: ctx.accounts.nft.key(),
        original_mint_owner: ctx.accounts.nft_owner.key(),
    });
        
    Ok(())
}
