#[derive(Accounts)]
pub struct ListDigitalProductCommission<'info>{
    #[account(
        init,
        space = 200,
        payer = seller_wallet
    )]
    pub digital_product: Box<Account<'info, DigitalProduct>>,

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
            b"recent_commission_catalog"
        ],
        bump
    )]
    pub recent_commission_catalog: Box<Account<'info, OrbitModCatalogStruct>>,

    #[account(
        seeds = [
            b"market_auth"
        ],
        bump
    )]
    pub market_auth: SystemAccount<'info>,

    pub catalog_program: Program<'info, OrbitCatalog>,

    pub digital_program: Program<'info, OrbitDigitalMarket>,
}

pub fn list_commission_handler(ctx: Context<ListDigitalProductCommission>, prod: OrbitProduct)-> Result<()> {
    if prod.seller != ctx.accounts.seller_account.key() {
        return err!(DigitalMarketErrors::InvalidSellerForListing)
    }
    ctx.accounts.digital_product.metadata = prod;
    ctx.accounts.digital_product.digital_product_type = DigitalProductType::Commission;
    match ctx.bumps.get("market_auth"){
        Some(auth_bump) => edit_mod_catalog(
            CpiContext::new_with_signer(
                ctx.accounts.catalog_program.to_account_info(),
                EditModCatalog {
                    catalog: ctx.accounts.recent_commission_catalog.to_account_info(),
                    product: ctx.accounts.digital_product.to_account_info(),
                    caller_auth: ctx.accounts.market_auth.to_account_info()
                },
                &[&[b"market_auth", &[*auth_bump]]])
        ),
        None => err!(DigitalMarketErrors::InvalidAuthBump)
    }
}