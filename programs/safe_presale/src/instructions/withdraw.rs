use crate::{error::CustomError, state::*};
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{close_account, transfer, CloseAccount, Token, Transfer},
    token_interface::{Mint, TokenAccount},
};

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(
        mut,
        close = user_wallet,
        constraint = purchase_receipt.original_mint == nft_owner_nft_token_account.mint @ CustomError::MintNotAllowed,
        constraint = purchase_receipt.pool == pool.key() @CustomError::InvalidPool
    )]
    pub purchase_receipt: Box<Account<'info, PurchaseReceipt>>,
    #[account(
        constraint = nft_owner_nft_token_account.amount == 1,
        constraint = nft_owner_nft_token_account.owner == user_wallet.key(),
    )]
    pub nft_owner_nft_token_account: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        constraint = pool.vesting_started_at.is_none() @CustomError::TokenHasLaunched,
        constraint = pool.presale_time_limit < Clock::get()?.unix_timestamp @CustomError::UnauthorizedAtCurrentTime
    )]
    pub pool: Box<Account<'info, Pool>>,
    #[account(mut)]
    pub user_wallet: Signer<'info>,
    #[account(
		init_if_needed,
        payer = user_wallet,
        associated_token::mint = wsol,
        associated_token::authority = user_wallet,
	)]
    pub user_token_wsol: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
		init_if_needed,
        payer = user_wallet,
        associated_token::mint = wsol,
        associated_token::authority = pool,
	)]
    pub pool_token_wsol: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
		address = public_keys::wsol::id()
	)]
    pub wsol: Box<InterfaceAccount<'info, Mint>>,
    /// Program to create the position manager state account
    pub system_program: Program<'info, System>,
    /// Program to create mint account and mint tokens
    pub token_program: Program<'info, Token>,
    /// Program to create an ATA for receiving position NFT
    pub associated_token_program: Program<'info, AssociatedToken>,
}
pub fn handler<'info>(ctx: Context<Withdraw<'info>>) -> Result<()> {
    let pool = &ctx.accounts.pool;
    let purchase_receipt = &mut ctx.accounts.purchase_receipt;
    let pool_identifier = pool.identifier.to_le_bytes();
    let pool_seed = &[
        POOL_PREFIX.as_bytes(),
        pool_identifier.as_ref(),
        &[pool.bump],
    ];
    let signer = &[&pool_seed[..]];

    transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.pool_token_wsol.to_account_info(),
                to: ctx.accounts.user_token_wsol.to_account_info(),
                authority: pool.to_account_info(),
            },
        )
        .with_signer(signer),
        purchase_receipt.amount,
    )?;

    msg!("Unwrapping any remaining WSol on payer account");
    close_account(CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        CloseAccount {
            account: ctx.accounts.user_token_wsol.to_account_info(),
            destination: ctx.accounts.user_wallet.to_account_info(),
            authority: ctx.accounts.user_wallet.to_account_info(),
        },
    ))?;

    emit!(WithdrawEvent {
        payer: ctx.accounts.user_wallet.key(),
        pool: pool.key(),
        original_mint: purchase_receipt.original_mint,
        amount_wsol_withdrawn: purchase_receipt.amount,
        wsol_mint: ctx.accounts.wsol.key(),
    });

    Ok(())
}
