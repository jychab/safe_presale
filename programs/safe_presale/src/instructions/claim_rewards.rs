use crate::error::CustomError;
use crate::state::*;
use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::Mint;
use anchor_spl::token::Token;
use anchor_spl::token::TokenAccount;
use anchor_spl::token::{self};
use std::cmp::min;

#[derive(Accounts)]
pub struct ClaimRewardsCtx<'info> {
    #[account(
        constraint = purchase_receipt.original_mint == original_mint.key()@ CustomError::InvalidStakeEntry
    )]
    purchase_receipt: Box<Account<'info, PurchaseReceipt>>,

    #[account(
        constraint = pool.key() == purchase_receipt.pool @CustomError::InvalidStakePool,
        constraint = pool.is_closed @CustomError::PresaleIsStillOngoing
    )]
    pool: Box<Account<'info, Pool>>,

    original_mint: Box<Account<'info, Mint>>,

    #[account(
        constraint = payer_original_mint_ata.amount > 0,
        constraint = payer_original_mint_ata.mint == original_mint.key(),
        constraint = payer_original_mint_ata.owner == payer.key(),
    )]
    payer_original_mint_ata:  Box<Account<'info, TokenAccount>>,

    #[account(
        mut, 
        constraint = reward_mint.key() == pool.mint @ CustomError::InvalidRewardMint
    )]
    reward_mint: Box<Account<'info, Mint>>,

    #[account(
        init_if_needed,
        payer = payer,
        associated_token::mint = reward_mint,
        associated_token::authority = payer,
    )]
    payer_reward_mint_token_account: Box<Account<'info, TokenAccount>>,

    #[account(mut)]
    payer: Signer<'info>,

    associated_token_program: Program<'info, AssociatedToken>,
    token_program: Program<'info, Token>,
    system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<ClaimRewardsCtx>,
) -> Result<()> {
    let pool = &mut ctx.accounts.pool;
    let purchase_receipt = &mut ctx.accounts.purchase_receipt;

    let pool_identifier = pool.identifier.to_le_bytes();
    let pool_seed = &[
        POOL_PREFIX.as_bytes(),
        pool_identifier.as_ref(),
        &[pool.bump],
    ];
    let signer = &[&pool_seed[..]];

    if let Some(last_claimed_at) = purchase_receipt.last_claimed_at {
        if last_claimed_at > pool.vesting_period {
            return Err(error!(CustomError::MaxRewardSecondsClaimed));
        }
    }
    let vesting_period_end = purchase_receipt.vesting_started_at.checked_add(pool.vesting_period).unwrap();
    let claim_duration = min(Clock::get().unwrap().unix_timestamp, vesting_period_end).checked_sub(purchase_receipt.last_claimed_at.unwrap_or(purchase_receipt.vesting_started_at)).unwrap();
    let allocated_liquidity_against_liquidity_pool = ctx.accounts.purchase_receipt.amount.checked_div(pool.liquidity_collected).unwrap();
    let token_allocation = allocated_liquidity_against_liquidity_pool.checked_mul(pool.supply_for_initial_liquidity).unwrap();


    if let Some(claimable_fraction) = claim_duration.checked_div(pool.vesting_period)  {
        let token_claimable = claimable_fraction.checked_mul(token_allocation.try_into().unwrap()).unwrap();
       //mint remaining token to pool
        let cpi_accounts = token::MintTo {
            mint: ctx.accounts.reward_mint.to_account_info(),
            to: ctx
                .accounts
                .payer_reward_mint_token_account
                .to_account_info(),
            authority: pool.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_context = CpiContext::new(cpi_program, cpi_accounts)
            .with_signer(signer);
        token::mint_to(
            cpi_context,
            token_claimable.try_into().unwrap()
        )?;
    };

    //update claims
    ctx.accounts.purchase_receipt.last_claimed_at =  Some(Clock::get().unwrap().unix_timestamp);

    Ok(())
}
