use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Mint, MintTo, Transfer, SetAuthority};
use spl_token::instruction::AuthorityType;

declare_id!("G1swVv4FoHnQVwAdMegnhxfybghfyg5LrnzHCDdHq1bS");

#[program]
pub mod solana_token {
    use super::*;

    pub fn initialize_mint(ctx: Context<Initialize>) -> Result<()> {
        let mint = &ctx.accounts.mint;
        let (mint_pda, bump) = Pubkey::find_program_address(&[b"mint_auth"], ctx.program_id);

        let cpi_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            SetAuthority {
                account_or_mint: mint.to_account_info(),
                current_authority: ctx.accounts.authority.to_account_info(),
            },
        );
        token::set_authority(cpi_ctx, AuthorityType::MintTokens, Some(mint_pda))?;

        Ok(())
    }

    pub fn secret_mint(ctx: Context<SecretMint>, amount: u64) -> Result<()> {
        let mint_state = &mut ctx.accounts.mint_state;
        let mint = &ctx.accounts.mint;
        let destination = &ctx.accounts.destination;

        if mint_state.minted + amount > mint_state.max_supply {
            return Err(ErrorCode::MaxSupplyExceeded.into());
        }

        let (mint_pda, bump) = Pubkey::find_program_address(&[b"mint_auth"], ctx.program_id);

        let signer_seeds: &[&[u8]] = &[b"mint_auth", &[bump][..]];

        let signer_seeds_array = &[signer_seeds]; 

        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            MintTo {
                mint: mint.to_account_info(),
                to: destination.to_account_info(),
                authority: ctx.accounts.mint_authority.to_account_info(),
            },
            signer_seeds_array, 
        );

        token::mint_to(cpi_ctx, amount)?;
        mint_state.minted += amount;

        Ok(())
    }

    pub fn transfer(ctx: Context<TransferTokens>, amount: u64) -> Result<()> {
        let transfer_instruction = Transfer {
            from: ctx.accounts.from.to_account_info(),
            to: ctx.accounts.to.to_account_info(),
            authority: ctx.accounts.from_authority.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(ctx.accounts.token_program.to_account_info(), transfer_instruction);
        token::transfer(cpi_ctx, amount)?;
        Ok(())
    }
}

#[account]
pub struct MintState {
    pub max_supply: u64, 
    pub minted: u64,      
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, payer = authority, space = 8 + 8 + 8)]  
    pub mint: Account<'info, Mint>,
    #[account(mut)]
    pub token_account: Account<'info, TokenAccount>,
    #[account(mut)] // Для оплаты
    pub authority: Signer<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,  
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct SecretMint<'info> {
    #[account(mut)]
    pub mint: Account<'info, Mint>,
    #[account(mut)]
    pub destination: Account<'info, TokenAccount>,
    #[account(mut)]
    pub mint_authority: Signer<'info>,
    pub token_program: Program<'info, Token>,
    #[account(init, payer = mint_authority, space = 8 + 8 + 8)]  
    pub mint_state: Account<'info, MintState>,  
    pub system_program: Program<'info, System>,  
}

#[derive(Accounts)]
pub struct TransferTokens<'info> {
    #[account(mut)]
    pub from: Account<'info, TokenAccount>,
    #[account(mut)]
    pub to: Account<'info, TokenAccount>,
    pub from_authority: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Max supply exceeded")]
    MaxSupplyExceeded,
}
