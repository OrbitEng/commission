use anchor_lang::prelude::*;
use crate::CommissionMarketErrors;
use orbit_catalog::{cpi::{
    accounts::CreateModCatalog,
    create_mod_catalog
}, program::OrbitCatalog};

#[derive(Accounts)]
pub struct CreateCommissionRecentCatalog<'info>{
    #[account(
        seeds = [
            b"recent_catalog"
        ],
        bump
    )]
    pub catalog: SystemAccount<'info>,

    #[account(
        seeds = [
            b"market_auth"
        ],
        bump
    )]
    pub market_auth: SystemAccount<'info>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub catalog_program: Program<'info, OrbitCatalog>,

    pub system_program: Program<'info, System>
}

pub fn recent_commission_catalog_handler(ctx: Context<CreateCommissionRecentCatalog>) -> Result<()>{
    match ctx.bumps.get("market_auth"){
        Some(auth_bump) => {
            create_mod_catalog(
                CpiContext::new_with_signer(
                    ctx.accounts.catalog_program.to_account_info(),
                    CreateModCatalog {
                        catalog: ctx.accounts.catalog.to_account_info(),
                        caller_auth: ctx.accounts.market_auth.to_account_info(),
                        payer: ctx.accounts.payer.to_account_info(),
                        system_program: ctx.accounts.system_program.to_account_info()
                    },
                    &[&[b"market_auth", &[*auth_bump]]]
                )
            ).expect("could not init mod catalog");
            create_mod_catalog(
                CpiContext::new_with_signer(
                    ctx.accounts.catalog_program.to_account_info(),
                    CreateModCatalog {
                        catalog: ctx.accounts.catalog.to_account_info(),
                        caller_auth: ctx.accounts.market_auth.to_account_info(),
                        payer: ctx.accounts.payer.to_account_info(),
                        system_program: ctx.accounts.system_program.to_account_info()
                    },
                    &[&[b"market_auth", &[*auth_bump]]]
                )
            )
        },
        None => err!(CommissionMarketErrors::InvalidAuthBump)
    }
    
}