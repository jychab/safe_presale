use std::str::FromStr;

use crate::error::CustomError;
use crate::state::*;
use crate::utils::U128;
use anchor_lang::prelude::*;
use anchor_lang::solana_program;
use anchor_spl::associated_token;
use anchor_spl::associated_token::Create;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked},
};

#[event_cpi]
#[derive(Accounts)]
pub struct LaunchTokenAmmCtx<'info> {
    #[account(mut,
        constraint = pool.presale_target == pool.liquidity_collected @CustomError::PresaleTargetNotMet,
        constraint = pool.vesting_started_at.is_none() @CustomError::TokenHasLaunched,
        constraint = pool.mint == amm_coin_mint.key(),
    )]
    pub pool: Box<Account<'info, Pool>>,
    /// Pays to mint the position
    #[account(mut,
        constraint = pool.authority == user_wallet.key() || (pool.delegate.is_some() && pool.delegate.unwrap() == user_wallet.key()),
    )]
    pub user_wallet: Signer<'info>,
    #[account(
        init_if_needed,
        payer = user_wallet,
        associated_token::mint = amm_coin_mint,
        associated_token::authority = user_wallet
    )]
    pub user_token_coin: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        init_if_needed,
        payer = user_wallet,
        associated_token::mint = amm_pc_mint,
        associated_token::authority = user_wallet,
    )]
    pub user_token_pc: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        mut,
        constraint = pool_token_coin.owner == pool.key(),
        constraint = pool_token_coin.mint == amm_coin_mint.key()
    )]
    pub pool_token_coin: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        mut,
        constraint = pool_token_pc.owner == pool.key(),
        constraint = pool_token_pc.mint == amm_pc_mint.key()
    )]
    pub pool_token_pc: Box<InterfaceAccount<'info, TokenAccount>>,
    /// Sysvar for token mint and ATA creation
    pub rent: Sysvar<'info, Rent>,
    /// Program to create the position manager state account
    pub system_program: Program<'info, System>,
    /// Program to create mint account and mint tokens
    pub token_program: Interface<'info, TokenInterface>,
    /// Program to create an ATA for receiving position NFT
    pub associated_token_program: Program<'info, AssociatedToken>,
    /// CHECK: Checked by cpi
    pub amm_coin_mint: Box<InterfaceAccount<'info, Mint>>,
    /// CHECK: Checked by cpi
    #[account(
        constraint = amm_pc_mint.key() == pool.quote_mint,
    )]
    pub amm_pc_mint: Box<InterfaceAccount<'info, Mint>>,
    /// CHECK: Checked by cpi
    #[account(
        address = Pubkey::from_str(RAYDIUM_CPMM_V4_MAINNET).unwrap()
    )]
    pub raydium_amm_program: AccountInfo<'info>,
}
pub fn handler<'a, 'b, 'c: 'info, 'info>(
    ctx: Context<'a, 'b, 'c, 'info, LaunchTokenAmmCtx<'info>>,
    open_time: u64,
) -> Result<()> {
    let current_time = Clock::get()?.unix_timestamp;
    let pool = &mut ctx.accounts.pool;
    let remaining_accounts = ctx.remaining_accounts.as_ref();
    let pool_token_pc = ctx.accounts.pool_token_pc.as_ref();
    let pool_token_coin = ctx.accounts.pool_token_coin.as_ref();
    let user_token_coin = ctx.accounts.user_token_coin.as_ref();
    let user_token_pc = ctx.accounts.user_token_pc.as_ref();
    let amm_lp_mint = remaining_accounts.get(0).unwrap().to_account_info();
    let user_token_lp = remaining_accounts.get(1).unwrap().to_account_info();
    let pool_token_lp = remaining_accounts.get(2).unwrap().to_account_info();
    let user_wallet = ctx.accounts.user_wallet.as_ref();
    let system_program = ctx.accounts.system_program.as_ref();
    let associated_token_program = ctx.accounts.associated_token_program.as_ref();
    let token_program = ctx.accounts.token_program.as_ref();
    // Launch Criteria
    // 1. Only allow launch after presale has ended
    // 2. Do not allow project to launch after the 7 day grace period
    // 3. Presale target must be met
    if current_time < pool.presale_time_limit {
        return Err(error!(CustomError::UnauthorizedAtCurrentTime));
    }
    if pool.presale_time_limit + GRACE_PERIOD < current_time {
        return Err(error!(CustomError::PoolHasExpired));
    }
    pool.vesting_started_at = Some(current_time);
    pool.lp_mint = Some(amm_lp_mint.key());

    let pool_seed = &[POOL_PREFIX.as_bytes(), pool.mint.as_ref(), &[pool.bump]];
    let signer = &[&pool_seed[..]];
    let amount_coin_in_pool = pool.liquidity_pool_supply;
    let amount_pc_in_pool = pool.liquidity_collected;

    transfer_amount(
        token_program.to_account_info(),
        pool_token_coin.to_account_info(),
        user_token_coin.to_account_info(),
        pool.to_account_info(),
        ctx.accounts.amm_coin_mint.to_account_info(),
        signer,
        amount_coin_in_pool,
        ctx.accounts.amm_coin_mint.decimals,
    )?;
    transfer_amount(
        token_program.to_account_info(),
        pool_token_pc.to_account_info(),
        user_token_pc.to_account_info(),
        pool.to_account_info(),
        ctx.accounts.amm_pc_mint.to_account_info(),
        signer,
        amount_pc_in_pool,
        ctx.accounts.amm_pc_mint.decimals,
    )?;

    let token_0_mint;
    let user_token_0_mint;
    let token_1_mint;
    let user_token_1_mint;
    let init_0_amount;
    let init_1_amount;
    if ctx.accounts.amm_coin_mint.key() < ctx.accounts.amm_pc_mint.key() {
        token_0_mint = ctx.accounts.amm_coin_mint.to_account_info();
        token_1_mint = ctx.accounts.amm_pc_mint.to_account_info();
        user_token_0_mint = user_token_coin.to_account_info();
        user_token_1_mint = user_token_pc.to_account_info();
        init_0_amount = amount_coin_in_pool;
        init_1_amount = amount_pc_in_pool;
    } else {
        token_0_mint = ctx.accounts.amm_pc_mint.to_account_info();
        token_1_mint = ctx.accounts.amm_coin_mint.to_account_info();
        user_token_0_mint = user_token_pc.to_account_info();
        user_token_1_mint = user_token_coin.to_account_info();
        init_0_amount = amount_pc_in_pool;
        init_1_amount = amount_coin_in_pool;
    };
    cpi_initialize(
        user_wallet.to_account_info(),
        remaining_accounts.get(3).unwrap().to_account_info(),
        remaining_accounts.get(4).unwrap().to_account_info(),
        remaining_accounts.get(5).unwrap().to_account_info(),
        token_0_mint,
        token_1_mint,
        amm_lp_mint.to_account_info(),
        user_token_0_mint.to_account_info(),
        user_token_1_mint.to_account_info(),
        user_token_lp.to_account_info(),
        remaining_accounts.get(6).unwrap().to_account_info(),
        remaining_accounts.get(7).unwrap().to_account_info(),
        remaining_accounts.get(8).unwrap().to_account_info(),
        remaining_accounts.get(9).unwrap().to_account_info(),
        token_program.to_account_info(),
        token_program.to_account_info(),
        token_program.to_account_info(),
        associated_token_program.to_account_info(),
        system_program.to_account_info(),
        ctx.accounts.rent.to_account_info(),
        ctx.accounts.raydium_amm_program.to_account_info(),
        init_0_amount,
        init_1_amount,
        open_time,
    )?;

    let liquidity = U128::from(init_0_amount)
        .checked_mul(init_1_amount.into())
        .unwrap()
        .integer_sqrt()
        .as_u64();
    let lock_lp_amount = 100;
    let user_lp_amount = liquidity.checked_sub(lock_lp_amount).unwrap();

    pool.lp_mint_supply = Some(user_lp_amount);
    transfer_lp_token(
        user_wallet.to_account_info(),
        associated_token_program.to_account_info(),
        pool.to_account_info(),
        amm_lp_mint.to_account_info(),
        system_program.to_account_info(),
        token_program.to_account_info(),
        user_token_lp.to_account_info(),
        pool_token_lp.to_account_info(),
        user_lp_amount,
        9,
    )?;

    pool.lp_mint_supply_for_creator = Some(
        U128::from(user_lp_amount)
            .checked_mul(pool.creator_fee_basis_points.try_into().unwrap())
            .and_then(|result| result.checked_div(U128::from(10000)))
            .and_then(|result| Some(result.as_u64()))
            .ok_or(CustomError::IntegerOverflow)?,
    );

    emit_cpi!(LaunchTokenAmmEvent {
        payer: user_wallet.key(),
        pool: pool.key(),
        amount_coin: amount_coin_in_pool,
        amount_pc: amount_pc_in_pool,
        amount_lp_received: user_lp_amount,
        lp_mint: pool.lp_mint.unwrap(),
        vesting_started_at: pool.vesting_started_at.unwrap(),
    });
    Ok(())
}

#[inline(never)]
fn transfer_lp_token<'info>(
    user_wallet: AccountInfo<'info>,
    associated_token_program: AccountInfo<'info>,
    pool: AccountInfo<'info>,
    mint: AccountInfo<'info>,
    system_program: AccountInfo<'info>,
    token_program: AccountInfo<'info>,
    user_token_lp: AccountInfo<'info>,
    pool_token_lp: AccountInfo<'info>,
    amount: u64,
    decimal: u8,
) -> Result<()> {
    associated_token::create(CpiContext::new(
        associated_token_program.to_account_info(),
        Create {
            payer: user_wallet.to_account_info(),
            associated_token: pool_token_lp.to_account_info(),
            authority: pool.to_account_info(),
            mint: mint.to_account_info(),
            system_program: system_program.to_account_info(),
            token_program: token_program.to_account_info(),
        },
    ))?;
    transfer_checked(
        CpiContext::new(
            token_program.to_account_info(),
            TransferChecked {
                mint: mint.to_account_info(),
                from: user_token_lp.to_account_info(),
                to: pool_token_lp.to_account_info(),
                authority: user_wallet.to_account_info(),
            },
        ),
        amount,
        decimal,
    )?;
    Ok(())
}

#[inline(never)]
fn cpi_initialize<'a, 'b, 'c: 'info, 'info>(
    user_wallet: AccountInfo<'info>,
    amm_config: AccountInfo<'info>,
    amm_authority: AccountInfo<'info>,
    amm_pool: AccountInfo<'info>,
    token_0_mint: AccountInfo<'info>,
    token_1_mint: AccountInfo<'info>,
    amm_lp_mint: AccountInfo<'info>,
    user_token_0_mint: AccountInfo<'info>,
    user_token_1_mint: AccountInfo<'info>,
    user_token_lp: AccountInfo<'info>,
    token_0_vault: AccountInfo<'info>,
    token_1_vault: AccountInfo<'info>,
    create_fee_destination: AccountInfo<'info>,
    observation_state: AccountInfo<'info>,
    token_program: AccountInfo<'info>,
    token_0_program: AccountInfo<'info>,
    token_1_program: AccountInfo<'info>,
    associated_token_program: AccountInfo<'info>,
    system_program: AccountInfo<'info>,
    rent: AccountInfo<'info>,
    amm_program: AccountInfo<'info>,
    init_0_amount: u64,
    init_1_amount: u64,
    open_time: u64,
) -> Result<()> {
    let accounts = vec![
        AccountMeta::new(user_wallet.key(), true),
        AccountMeta::new_readonly(amm_config.key(), false),
        AccountMeta::new_readonly(amm_authority.key(), false),
        AccountMeta::new(amm_pool.key(), false),
        AccountMeta::new_readonly(token_0_mint.key(), false),
        AccountMeta::new_readonly(token_1_mint.key(), false),
        AccountMeta::new(amm_lp_mint.key(), false),
        AccountMeta::new(user_token_0_mint.key(), false),
        AccountMeta::new(user_token_1_mint.key(), false),
        AccountMeta::new(user_token_lp.key(), false),
        AccountMeta::new(token_0_vault.key(), false),
        AccountMeta::new(token_1_vault.key(), false),
        AccountMeta::new(create_fee_destination.key(), false),
        AccountMeta::new(observation_state.key(), false),
        AccountMeta::new_readonly(token_program.key(), false),
        AccountMeta::new_readonly(token_0_program.key(), false),
        AccountMeta::new_readonly(token_1_program.key(), false),
        AccountMeta::new_readonly(associated_token_program.key(), false),
        AccountMeta::new_readonly(system_program.key(), false),
        AccountMeta::new_readonly(rent.key(), false),
    ];
    let mut bytes_data = vec![];
    bytes_data.extend([175, 175, 109, 31, 13, 152, 155, 237]);
    bytes_data.extend(init_0_amount.to_le_bytes());
    bytes_data.extend(init_1_amount.to_le_bytes());
    bytes_data.extend(open_time.to_le_bytes());

    let account_infos: Vec<AccountInfo> = vec![
        user_wallet,
        amm_config,
        amm_authority,
        amm_pool,
        token_0_mint,
        token_1_mint,
        amm_lp_mint,
        user_token_0_mint,
        user_token_1_mint,
        user_token_lp,
        token_0_vault,
        token_1_vault,
        create_fee_destination,
        observation_state,
        token_program,
        token_0_program,
        token_1_program,
        associated_token_program,
        system_program,
        rent,
    ];

    let _invoke = solana_program::program::invoke(
        &solana_program::instruction::Instruction {
            program_id: amm_program.key(),
            accounts,
            data: bytes_data,
        },
        &account_infos[..],
    );
    Ok(())
}

#[inline(never)]
fn transfer_amount<'info>(
    token_program: AccountInfo<'info>,
    from: AccountInfo<'info>,
    to: AccountInfo<'info>,
    authority: AccountInfo<'info>,
    mint: AccountInfo<'info>,
    signer: &[&[&[u8]]; 1],
    amount_max: u64,
    decimal: u8,
) -> Result<()> {
    transfer_checked(
        CpiContext::new(
            token_program,
            TransferChecked {
                mint,
                from,
                to,
                authority,
            },
        )
        .with_signer(signer),
        amount_max,
        decimal,
    )?;
    Ok(())
}
