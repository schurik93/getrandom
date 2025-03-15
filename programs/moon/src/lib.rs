use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount, Mint, MintTo};
use getrandom;

declare_id!("6oiEMKo11VSHtnbu5U8ujSZwkNKugLwwDjVcNnQkG9Zx");

const CONFIG_SPACE: usize = 8 + 8 + 32;

fn get_random_u128() -> Result<u128, getrandom::Error> {
    let mut buf = [0u8; 16];
    getrandom::fill(&mut buf)?;
    Ok(u128::from_ne_bytes(buf))
}

#[program]
pub mod moon_token {
    use super::*;

    pub fn initialize(ctx: Context<InitializeMint>) -> Result<()> {
        let amount = get_random_u128().unwrap_or(10_000_000 * 10u64.pow(9)); // Verwende zufällige Zahl als Amount, fallback auf festen Wert

        anchor_spl::token::mint_to(
            ctx.accounts.into_mint_to_context(),
            amount,
        )?;

        let config = &mut ctx.accounts.config;
        config.transfers_enabled = false;
        config.authority = *ctx.accounts.authority.key;

        Ok(())
    }

    pub fn enable_transfers(ctx: Context<EnableTransfers>) -> Result<()> {
        let config = &mut ctx.accounts.config;
        require!(config.authority == *ctx.accounts.authority.key, CustomError::Unauthorized);
        config.transfers_enabled = true;
        Ok(())
    }

    pub fn transfer(ctx: Context<Transfer>, _amount: u64) -> Result<()> {
        return Err(CustomError::TransfersDisabled.into());
    }
}

#[derive(Accounts)]
pub struct InitializeMint<'info> {
    #[account(init, payer = payer, mint::decimals = 9, mint::authority = authority)]
    pub mint: Account<'info, Mint>, // Angepasste Deklaration
    #[account(init, payer = payer, space = CONFIG_SPACE)]
    pub config: Account<'info, Config>,
    #[account(signer)]
    pub authority: Signer<'info>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct EnableTransfers<'info> {
    #[account(mut, has_one = authority)]
    pub config: Account<'info, Config>,
    #[account(signer)]
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct Transfer<'info> {
    #[account(mut)]
    pub from: Account<'info, TokenAccount>, // Angepasste Deklaration
    #[account(mut)]
    pub to: Account<'info, TokenAccount>,
    #[account(mut)]
    pub config: Account<'info, Config>,
    #[account(signer)]
    pub authority: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

impl<'info> InitializeMint<'info> {
    fn into_mint_to_context(&self) -> CpiContext<'_, '_, '_, 'info, MintTo<'info>> {
        let cpi_accounts = MintTo {
            mint: self.mint.to_account_info().clone(),
            to: self.mint.to_account_info().clone(),
            authority: self.mint.to_account_info().clone(),
        };
        let cpi_program = self.token_program.clone();
        CpiContext::new(cpi_program.to_account_info(), cpi_accounts)
    }
}

#[account]
pub struct Config {
    pub transfers_enabled: bool,
    pub authority: Pubkey,
}

#[error_code]
pub enum CustomError {
    #[msg("Transfers are disabled.")]
    TransfersDisabled,
    #[msg("Unauthorized")]
    Unauthorized,
}