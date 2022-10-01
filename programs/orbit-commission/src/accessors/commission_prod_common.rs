use anchor_lang::{
    prelude::*,
    AccountsClose
};
use orbit_catalog::{
    structs::OrbitModCatalogStruct,
    cpi::{
        accounts::EditModCatalog,
        edit_mod_catalog
    }, program::OrbitCatalog
};
use market_accounts::OrbitMarketAccount;
use product::{product_struct::OrbitProduct, product_trait::OrbitProductTrait};
use crate::{CommissionProduct, CommissionMarketErrors, program::OrbitCommissionMarket};

#[derive(Accounts)]
pub struct ListCommissionProduct<'info>{
    
    #[account(
        init,
        space = 200,
        payer = seller_wallet
    )]
    pub commission_product: Box<Account<'info, CommissionProduct>>,

    #[account(
        seeds = [
            b"orbit_account",
            seller_wallet.key().as_ref()
        ],
        bump,
        seeds::program = market_accounts::ID
    )]
    pub seller_account: Box<Account<'info, OrbitMarketAccount>>,

    #[account(
        mut,
        address = seller_account.wallet
    )]
    pub seller_wallet: Signer<'info>,

    pub system_program: Program<'info, System>,

    #[account(
        mut,
        seeds = [
            b"recent_catalog"
        ],
        bump
    )]
    pub recent_catalog: Box<Account<'info, OrbitModCatalogStruct>>,

    #[account(
        seeds = [
            b"market_auth"
        ],
        bump
    )]
    pub market_auth: SystemAccount<'info>,

    pub catalog_program: Program<'info, OrbitCatalog>,

    pub commission_program: Program<'info, OrbitCommissionMarket>,
}

#[derive(Accounts)]
pub struct UnlistCommissionProduct<'info>{

    #[account(mut)]
    pub commission_product: Account<'info, CommissionProduct>,

    #[account(
        address = commission_product.metadata.seller
    )]
    pub seller_account: Account<'info, OrbitMarketAccount>,

    #[account(
        mut,
        address = seller_account.wallet
    )]
    pub seller_wallet: Signer<'info>,
}

impl <'a, 'b> OrbitProductTrait<'a, 'b, ListCommissionProduct<'a>, UnlistCommissionProduct<'b>> for CommissionProduct{
    fn list(ctx: Context<ListCommissionProduct>, prod: OrbitProduct)-> Result<()> {
        if prod.seller != ctx.accounts.seller_account.key() {
            return err!(CommissionMarketErrors::InvalidSellerForListing)
        }
        ctx.accounts.commission_product.metadata = prod;
        match ctx.bumps.get("market_auth"){
            Some(auth_bump) => edit_mod_catalog(
                CpiContext::new_with_signer(
                    ctx.accounts.catalog_program.to_account_info(),
                    EditModCatalog {
                        catalog: ctx.accounts.recent_catalog.to_account_info(),
                        product: ctx.accounts.commission_product.to_account_info(),
                        caller_auth: ctx.accounts.market_auth.to_account_info()
                    },
                    &[&[b"market_auth", &[*auth_bump]]])
            ),
            None => err!(CommissionMarketErrors::InvalidAuthBump)
        }
    }

    fn unlist(ctx: Context<UnlistCommissionProduct>)-> Result<()> {
        ctx.accounts.commission_product.close(ctx.accounts.seller_wallet.to_account_info())
    }
}

#[derive(Accounts, CommonProdUtils)]
pub struct UpdateProductField<'info>{

    #[account(mut)]
    pub commission_product: Account<'info, CommissionProduct>,

    #[account(
        address = commission_product.metadata.seller,
        seeds = [
            b"orbit_account",
            wallet.key().as_ref()
        ],
        bump,
        seeds::program = market_accounts::ID
    )]
    pub seller_account: Account<'info, OrbitMarketAccount>,

    #[account(
        mut,
        address = seller_account.wallet
    )]
    pub wallet: Signer<'info>
}

pub fn change_availability_handler(ctx: Context<UpdateProductField>, available: bool) -> Result<()>{
    ctx.accounts.commission_product.metadata.available = available;
    Ok(())
}