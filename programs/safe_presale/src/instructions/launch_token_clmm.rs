use crate::state::*;
use anchor_lang::prelude::*;
use anchor_lang::solana_program;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::transfer;
use anchor_spl::token::Token;
use anchor_spl::token::Transfer;
use anchor_spl::token_2022::Token2022;
use anchor_spl::token_interface::Mint;
use anchor_spl::token_interface::TokenAccount;

#[derive(Accounts)]
pub struct LaunchTokenClmmCtx<'info> {
    /// Pays to mint the position
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(mut)]
    pub pool: Box<Account<'info, Pool>>,
    /// CHECK: Checked by cpi
    #[account(
        init_if_needed,
        payer = payer,
        associated_token::mint = vault_0_mint,
        associated_token::authority = payer
    )]
    pub token_account_0: Box<InterfaceAccount<'info, TokenAccount>>,
    /// CHECK: Checked by cpi
    #[account(
        init_if_needed,
        payer = payer,
        associated_token::mint = vault_1_mint,
        associated_token::authority = payer,
    )]
    pub token_account_1: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        mut,
        constraint = pool_token_account_0.owner == pool.key(),
        constraint = pool_token_account_0.mint == vault_0_mint.key()
    )]
    pub pool_token_account_0: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        mut,
        constraint = pool_token_account_1.owner == pool.key(),
        constraint = pool_token_account_1.mint == vault_1_mint.key()
    )]
    pub pool_token_account_1: Box<InterfaceAccount<'info, TokenAccount>>,
    /// Sysvar for token mint and ATA creation
    pub rent: Sysvar<'info, Rent>,
    /// Program to create the position manager state account
    pub system_program: Program<'info, System>,
    /// Program to create mint account and mint tokens
    pub token_program: Program<'info, Token>,
    /// Program to create an ATA for receiving position NFT
    pub associated_token_program: Program<'info, AssociatedToken>,
    /// Program to create NFT metadata
    /// CHECK: Metadata program address constraint applied
    pub metadata_program: UncheckedAccount<'info>,
    /// Program to create mint account and mint tokens
    pub token_program_2022: Program<'info, Token2022>,
    /// CHECK: Checked by cpi
    pub vault_0_mint: Box<InterfaceAccount<'info, Mint>>,
    /// CHECK: Checked by cpi
    pub vault_1_mint: Box<InterfaceAccount<'info, Mint>>,
    /// CHECK: Checked by cpi
    pub raydium_program: UncheckedAccount<'info>,
}
pub fn handler<'a, 'b, 'c: 'info, 'info>(
    ctx: Context<'a, 'b, 'c, 'info, LaunchTokenClmmCtx<'info>>,
    liquidity: u128,
    amount_0_max: u64,
    amount_1_max: u64,
    tick_lower_index: i32,
    tick_upper_index: i32,
    tick_array_lower_start_index: i32,
    tick_array_upper_start_index: i32,
) -> Result<()> {
    let pool = &mut ctx.accounts.pool;
    pool.allow_purchase = true;
    let pool_identifier = pool.identifier.to_le_bytes();
    let pool_seed = &[
        POOL_PREFIX.as_bytes(),
        pool_identifier.as_ref(),
        &[pool.bump],
    ];
    let signer = &[&pool_seed[..]];

    //transfer token_0 amount from pool to payer
    transfer_amount(
        ctx.accounts.token_program.to_account_info(),
        ctx.accounts.pool_token_account_0.to_account_info(),
        ctx.accounts.token_account_0.to_account_info(),
        ctx.accounts.pool.to_account_info(),
        signer,
        amount_0_max,
    )?;
    //transfer token_1 amount from pool to payer
    transfer_amount(
        ctx.accounts.token_program.to_account_info(),
        ctx.accounts.pool_token_account_1.to_account_info(),
        ctx.accounts.token_account_1.to_account_info(),
        ctx.accounts.pool.to_account_info(),
        signer,
        amount_1_max,
    )?;

    cpi_open_position(
        &ctx.accounts.payer,
        &ctx.accounts.pool,
        &ctx.accounts.token_account_0,
        &ctx.accounts.token_account_1,
        &ctx.accounts.rent,
        &ctx.accounts.system_program,
        &ctx.accounts.token_program,
        &ctx.accounts.associated_token_program,
        &ctx.accounts.metadata_program,
        &ctx.accounts.token_program_2022,
        &ctx.accounts.vault_0_mint,
        &ctx.accounts.vault_1_mint,
        &ctx.accounts.raydium_program,
        &ctx.remaining_accounts,
        tick_lower_index,
        tick_upper_index,
        tick_array_lower_start_index,
        tick_array_upper_start_index,
        liquidity,
        amount_0_max,
        amount_1_max,
    )?;
    Ok(())
}

#[inline(never)]
fn cpi_open_position<'a, 'b, 'c: 'info, 'info>(
    payer: &'b Signer<'info>,
    pool: &'b Box<Account<'info, Pool>>,
    token_account_0: &'b Box<InterfaceAccount<'info, TokenAccount>>,
    token_account_1: &'b Box<InterfaceAccount<'info, TokenAccount>>,
    rent: &'b Sysvar<'info, Rent>,
    system_program: &'b Program<'info, System>,
    token_program: &'b Program<'info, Token>,
    associated_token_program: &'b Program<'info, AssociatedToken>,
    metadata_program: &'b UncheckedAccount<'info>,
    token_program_2022: &'b Program<'info, Token2022>,
    vault_0_mint: &'b Box<InterfaceAccount<'info, Mint>>,
    vault_1_mint: &'b Box<InterfaceAccount<'info, Mint>>,
    raydium_program: &'b UncheckedAccount<'info>,

    remaining_accounts: &'c [AccountInfo<'info>],
    tick_lower_index: i32,
    tick_upper_index: i32,
    tick_array_lower_start_index: i32,
    tick_array_upper_start_index: i32,
    liquidity: u128,
    amount_0_max: u64,
    amount_1_max: u64,
) -> Result<()> {
    let accounts = vec![
        // clmm
        AccountMeta::new(payer.key(), true),
        AccountMeta::new_readonly(pool.key(), false),
        AccountMeta::new(remaining_accounts.get(0).unwrap().key(), true),
        AccountMeta::new(remaining_accounts.get(1).unwrap().key(), false),
        AccountMeta::new(remaining_accounts.get(2).unwrap().key(), false),
        AccountMeta::new(remaining_accounts.get(3).unwrap().key(), false),
        AccountMeta::new(remaining_accounts.get(4).unwrap().key(), false),
        AccountMeta::new(remaining_accounts.get(5).unwrap().key(), false),
        AccountMeta::new(remaining_accounts.get(6).unwrap().key(), false),
        AccountMeta::new(remaining_accounts.get(7).unwrap().key(), false),
        AccountMeta::new(token_account_0.key(), false),
        AccountMeta::new(token_account_1.key(), false),
        AccountMeta::new(remaining_accounts.get(8).unwrap().key(), false),
        AccountMeta::new(remaining_accounts.get(9).unwrap().key(), false),
        // programs
        AccountMeta::new_readonly(rent.key(), false),
        AccountMeta::new_readonly(system_program.key(), false),
        AccountMeta::new_readonly(token_program.key(), false),
        AccountMeta::new_readonly(associated_token_program.key(), false),
        AccountMeta::new_readonly(metadata_program.key(), false),
        AccountMeta::new_readonly(token_program_2022.key(), false),
        // mints
        AccountMeta::new_readonly(vault_0_mint.key(), false),
        AccountMeta::new_readonly(vault_1_mint.key(), false),
    ];

    let mut bytes_data = vec![];
    bytes_data.extend([77, 184, 74, 214, 112, 86, 241, 199]);
    bytes_data.extend(tick_lower_index.to_le_bytes());
    bytes_data.extend(tick_upper_index.to_le_bytes());
    bytes_data.extend(tick_array_lower_start_index.to_le_bytes());
    bytes_data.extend(tick_array_upper_start_index.to_le_bytes());
    bytes_data.extend(liquidity.to_le_bytes());
    bytes_data.extend(amount_0_max.to_le_bytes());
    bytes_data.extend(amount_1_max.to_le_bytes());
    bytes_data.extend([1]);
    //with metadata = create
    bytes_data.extend([0]);
    //option<Baseflag> = false

    let account_infos: Vec<AccountInfo> = vec![
        payer.to_account_info(),
        pool.to_account_info(),
        remaining_accounts.get(0).unwrap().to_account_info(),
        remaining_accounts.get(1).unwrap().to_account_info(),
        remaining_accounts.get(2).unwrap().to_account_info(),
        remaining_accounts.get(3).unwrap().to_account_info(),
        remaining_accounts.get(4).unwrap().to_account_info(),
        remaining_accounts.get(5).unwrap().to_account_info(),
        remaining_accounts.get(6).unwrap().to_account_info(),
        remaining_accounts.get(7).unwrap().to_account_info(),
        token_account_0.to_account_info(),
        token_account_1.to_account_info(),
        remaining_accounts.get(8).unwrap().to_account_info(),
        remaining_accounts.get(9).unwrap().to_account_info(),
        rent.to_account_info(),
        system_program.to_account_info(),
        token_program.to_account_info(),
        associated_token_program.to_account_info(),
        metadata_program.to_account_info(),
        token_program_2022.to_account_info(),
        vault_0_mint.to_account_info(),
        vault_1_mint.to_account_info(),
    ];

    let _invoke = solana_program::program::invoke(
        &solana_program::instruction::Instruction {
            program_id: raydium_program.key(),
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
