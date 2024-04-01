use crate::error::CustomError;
use crate::state::*;
use crate::utils::Calculator;
use crate::utils::U128;
use anchor_lang::prelude::*;
use anchor_lang::solana_program;
use anchor_spl::associated_token;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::associated_token::Create;
use anchor_spl::token::transfer;
use anchor_spl::token::Token;
use anchor_spl::token::Transfer;
use anchor_spl::token_interface::Mint;
use anchor_spl::token_interface::TokenAccount;

#[derive(Accounts)]
pub struct LaunchTokenAmmCtx<'info> {
    #[account(mut,
        constraint = pool.mint == amm_coin_mint.key(),
        constraint = pool.authority == user_wallet.key(),
        constraint = pool.vesting_started_at.is_none() @CustomError::TokenHasLaunched
    )]
    pub pool: Box<Account<'info, Pool>>,
    /// Pays to mint the position
    #[account(mut)]
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
    /// CHECK: To be created by cpi
    #[account(mut)]
    pub user_token_lp: UncheckedAccount<'info>,
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
    /// CHECK: To be created later after lp mint has been created by cpi
    #[account(mut)]
    pub pool_token_lp: UncheckedAccount<'info>,
    /// Sysvar for token mint and ATA creation
    pub rent: Sysvar<'info, Rent>,
    /// Program to create the position manager state account
    pub system_program: Program<'info, System>,
    /// Program to create mint account and mint tokens
    pub token_program: Program<'info, Token>,
    /// Program to create an ATA for receiving position NFT
    pub associated_token_program: Program<'info, AssociatedToken>,
    /// CHECK: Checked by cpi
    pub amm_coin_mint: Box<InterfaceAccount<'info, Mint>>,
    /// CHECK: Checked by cpi
    #[account(
        constraint = amm_pc_mint.key() == public_keys::wsol::id()
    )]
    pub amm_pc_mint: Box<InterfaceAccount<'info, Mint>>,
    /// CHECK: To be created by cpi
    #[account(mut)]
    pub amm_lp_mint: UncheckedAccount<'info>,
    /// CHECK: Checked by cpi
    #[account(address = public_keys::amm_v4_devnet::id())]
    pub raydium_amm_program: AccountInfo<'info>,
}
pub fn handler<'a, 'b, 'c: 'info, 'info>(
    ctx: Context<'a, 'b, 'c, 'info, LaunchTokenAmmCtx<'info>>,
    nonce: u8,
    open_time: u64,
) -> Result<()> {
    let current_time = Clock::get()?.unix_timestamp;
    let pool = &mut ctx.accounts.pool;
    pool.allow_purchase = false;
    pool.vesting_started_at = Some(current_time);
    pool.vesting_period_end = Some(
        current_time
            .checked_add(pool.vesting_period.try_into().unwrap())
            .ok_or(CustomError::IntegerOverflow)?,
    );
    pool.lp_mint = Some(ctx.accounts.amm_lp_mint.key());

    let pool_identifier = pool.identifier.to_le_bytes();
    let pool_seed = &[
        POOL_PREFIX.as_bytes(),
        pool_identifier.as_ref(),
        &[pool.bump],
    ];
    let signer = &[&pool_seed[..]];

    let pool_token_coin = ctx.accounts.pool_token_coin.as_ref();
    let pool_token_pc = ctx.accounts.pool_token_pc.as_ref();
    let user_token_coin = ctx.accounts.user_token_coin.as_ref();
    let user_token_pc = ctx.accounts.user_token_pc.as_ref();
    let user_token_lp = ctx.accounts.user_token_lp.as_ref();

    let amm_lp_mint = ctx.accounts.amm_lp_mint.as_ref();
    let user_wallet = ctx.accounts.user_wallet.as_ref();

    let system_program = ctx.accounts.system_program.as_ref();
    let associated_token_program = ctx.accounts.associated_token_program.as_ref();
    let token_program = ctx.accounts.token_program.as_ref();
    let remaining_accounts = ctx.remaining_accounts.as_ref();
    let amount_coin_in_pool = pool.total_supply.checked_sub(pool.vested_supply).unwrap();
    let amount_pc_in_pool = pool.liquidity_collected;

    msg!(
        "Transfering {} mint from pool to payer",
        amount_coin_in_pool
    );
    transfer_amount(
        token_program.to_account_info(),
        pool_token_coin.to_account_info(),
        user_token_coin.to_account_info(),
        pool.to_account_info(),
        signer,
        amount_coin_in_pool,
    )?;
    msg!(
        "Transfering {} lamports from pool to payer",
        amount_pc_in_pool
    );
    transfer_amount(
        token_program.to_account_info(),
        pool_token_pc.to_account_info(),
        user_token_pc.to_account_info(),
        pool.to_account_info(),
        signer,
        amount_pc_in_pool,
    )?;
    msg!("Launching Amm");
    cpi_initialize2(
        token_program.to_account_info(),
        associated_token_program.to_account_info(),
        system_program.to_account_info(),
        ctx.accounts.rent.to_account_info(),
        ctx.accounts.raydium_amm_program.to_account_info(),
        remaining_accounts.get(0).unwrap().to_account_info(),
        remaining_accounts.get(1).unwrap().to_account_info(),
        remaining_accounts.get(2).unwrap().to_account_info(),
        amm_lp_mint.to_account_info(),
        ctx.accounts.amm_coin_mint.to_account_info(),
        ctx.accounts.amm_pc_mint.to_account_info(),
        remaining_accounts.get(3).unwrap().to_account_info(),
        remaining_accounts.get(4).unwrap().to_account_info(),
        remaining_accounts.get(5).unwrap().to_account_info(),
        remaining_accounts.get(6).unwrap().to_account_info(),
        remaining_accounts.get(7).unwrap().to_account_info(),
        remaining_accounts.get(8).unwrap().to_account_info(),
        remaining_accounts.get(9).unwrap().to_account_info(),
        user_wallet.to_account_info(),
        user_token_coin.to_account_info(),
        user_token_pc.to_account_info(),
        user_token_lp.to_account_info(),
        nonce,
        open_time,
        amount_pc_in_pool,
        amount_coin_in_pool,
    )?;

    let liquidity = Calculator::to_u64(
        U128::from(amount_pc_in_pool)
            .checked_mul(amount_coin_in_pool.into())
            .unwrap()
            .integer_sqrt()
            .as_u128(),
    )?;
    let user_lp_amount = liquidity
        .checked_sub(
            (10u64)
                .checked_pow(ctx.accounts.amm_coin_mint.decimals.into()) //lp mint decimal is the same as coin mint decimals
                .unwrap(),
        )
        .unwrap();

    msg!("Transfering {} lp token from payer to pool", user_lp_amount);
    pool.lp_mint_supply = user_lp_amount;
    transfer_lp_token(
        user_wallet.to_account_info(),
        associated_token_program.to_account_info(),
        pool.to_account_info(),
        amm_lp_mint.to_account_info(),
        system_program.to_account_info(),
        token_program.to_account_info(),
        user_token_lp.to_account_info(),
        ctx.accounts.pool_token_lp.to_account_info(),
        user_lp_amount,
    )?;
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
    transfer(
        CpiContext::new(
            token_program.to_account_info(),
            Transfer {
                from: user_token_lp.to_account_info(),
                to: pool_token_lp.to_account_info(),
                authority: user_wallet.to_account_info(),
            },
        ),
        amount,
    )?;
    Ok(())
}

#[inline(never)]
fn cpi_initialize2<'a, 'b, 'c: 'info, 'info>(
    token_program: AccountInfo<'info>,
    associated_token_program: AccountInfo<'info>,
    system_program: AccountInfo<'info>,
    rent: AccountInfo<'info>,
    amm_program: AccountInfo<'info>,
    amm_pool: AccountInfo<'info>,
    amm_authority: AccountInfo<'info>,
    amm_open_orders: AccountInfo<'info>,
    amm_lp_mint: AccountInfo<'info>,
    amm_coin_mint: AccountInfo<'info>,
    amm_pc_mint: AccountInfo<'info>,
    amm_coin_vault: AccountInfo<'info>,
    amm_pc_vault: AccountInfo<'info>,
    amm_target_orders: AccountInfo<'info>,
    amm_config: AccountInfo<'info>,
    create_fee_destination: AccountInfo<'info>,
    market_program: AccountInfo<'info>,
    market: AccountInfo<'info>,
    user_wallet: AccountInfo<'info>,
    user_token_coin: AccountInfo<'info>,
    user_token_pc: AccountInfo<'info>,
    user_token_lp: AccountInfo<'info>,
    nonce: u8,
    open_time: u64,
    init_pc_amount: u64,
    init_coin_amount: u64,
) -> Result<()> {
    let accounts = vec![
        // spl & sys
        AccountMeta::new_readonly(token_program.key(), false),
        AccountMeta::new_readonly(associated_token_program.key(), false),
        AccountMeta::new_readonly(system_program.key(), false),
        AccountMeta::new_readonly(rent.key(), false),
        // amm
        AccountMeta::new(amm_pool.key(), false),
        AccountMeta::new_readonly(amm_authority.key(), false),
        AccountMeta::new(amm_open_orders.key(), false),
        AccountMeta::new(amm_lp_mint.key(), false),
        AccountMeta::new_readonly(amm_coin_mint.key(), false),
        AccountMeta::new_readonly(amm_pc_mint.key(), false),
        AccountMeta::new(amm_coin_vault.key(), false),
        AccountMeta::new(amm_pc_vault.key(), false),
        AccountMeta::new(amm_target_orders.key(), false),
        AccountMeta::new_readonly(amm_config.key(), false),
        AccountMeta::new(create_fee_destination.key(), false),
        // market
        AccountMeta::new_readonly(market_program.key(), false),
        AccountMeta::new_readonly(market.key(), false),
        // user wallet
        AccountMeta::new(user_wallet.key(), true),
        AccountMeta::new(user_token_coin.key(), false),
        AccountMeta::new(user_token_pc.key(), false),
        AccountMeta::new(user_token_lp.key(), false),
    ];

    let mut bytes_data = vec![];
    bytes_data.extend([1]);
    bytes_data.extend(nonce.to_le_bytes());
    bytes_data.extend(open_time.to_le_bytes());
    bytes_data.extend(init_pc_amount.to_le_bytes());
    bytes_data.extend(init_coin_amount.to_le_bytes());

    let account_infos: Vec<AccountInfo> = vec![
        token_program,
        associated_token_program,
        system_program,
        rent,
        amm_pool,
        amm_authority,
        amm_open_orders,
        amm_lp_mint,
        amm_coin_mint,
        amm_pc_mint,
        amm_coin_vault,
        amm_pc_vault,
        amm_target_orders,
        amm_config,
        create_fee_destination,
        market_program,
        market,
        user_wallet,
        user_token_coin,
        user_token_pc,
        user_token_lp,
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
    signer: &[&[&[u8]]; 1],
    amount_max: u64,
) -> Result<()> {
    transfer(
        CpiContext::new(
            token_program,
            Transfer {
                from,
                to,
                authority,
            },
        )
        .with_signer(signer),
        amount_max,
    )?;
    Ok(())
}
