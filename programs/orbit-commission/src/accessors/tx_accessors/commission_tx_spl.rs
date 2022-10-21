use anchor_lang::prelude::*;
use market_accounts::{
    OrbitMarketAccount,
    program::OrbitMarketAccounts
};
use orbit_product::{ListingsStruct, program::OrbitProduct};
use crate::{
    CommissionTransaction,
    BuyerDecisionState, program::OrbitCommissionMarket,
};
use orbit_transaction::{transaction_struct::TransactionState, BuyerOpenTransactions, SellerOpenTransactions, program::OrbitTransaction};
use orbit_product::CommissionProduct;
use anchor_spl::token::{
    TokenAccount,
    Mint,
    Token
};

#[derive(Accounts)]
#[instruction(seller_tx_index: u8)]
pub struct OpenCommissionTransactionSpl<'info>{
    //////////////////////////////////
    /// TX
    #[account(
        init,
        space = 4000,
        payer = buyer_wallet,
        seeds = [
            b"orbit_commission_transaction",
            seller_transactions_log.key().as_ref(),
            [seller_tx_index].as_ref()
        ],
        bump
    )]
    pub commission_transaction: Box<Account<'info, CommissionTransaction>>,

    #[account(
        init,
        token::mint = token_mint,
        token::authority = commission_auth,
        seeds = [
            b"orbit_escrow_account",
            commission_transaction.key().as_ref(),
            buyer_transactions_log.key().as_ref()
        ],
        bump,
        payer = buyer_wallet
    )]
    pub escrow_account: Account<'info, TokenAccount>,

    //////////////////////////////////
    /// PRODUCT
    pub token_mint: Account<'info, Mint>,

    pub commission_product: Box<Account<'info, CommissionProduct>>,
    
    //////////////////////////////////////////////////
    /// BUYER SELLER
    
    /// BUYER
    #[account(
        mut,
        has_one = buyer_wallet
    )]
    pub buyer_transactions_log: Box<Account<'info, BuyerOpenTransactions>>,

    #[account(
        mut,
        constraint = buyer_market_account.wallet == buyer_wallet.key(),
        constraint = buyer_market_account.buyer_commission_transactions == buyer_transactions_log.key()
    )]
    pub buyer_market_account: Box<Account<'info, OrbitMarketAccount>>,
    
    #[account(mut)]
    pub buyer_wallet: Signer<'info>,
    
    /// SELLER
    #[account(
        address = commission_product.metadata.owner_catalog
    )]
    pub seller_listings: Box<Account<'info, ListingsStruct>>,

    #[account(
        mut,
        constraint = seller_transactions_log.seller_wallet == seller_listings.listings_owner
    )]
    pub seller_transactions_log: Box<Account<'info, SellerOpenTransactions>>,

    //////////////////////////////////////////////////
    /// EXTRANEOUS CPI
    #[account(
        seeds = [b"market_authority"],
        bump
    )]
    pub commission_auth: SystemAccount<'info>,

    pub commission_program: Program<'info, OrbitCommissionMarket>,

    pub market_account_program: Program<'info, OrbitMarketAccounts>,

    pub system_program: Program<'info, System>,

    pub token_program: Program<'info, Token>,

    pub product_program: Program<'info, OrbitProduct>,
    
    pub transaction_program: Program<'info, OrbitTransaction>,

    pub rent: Sysvar<'info, Rent>
}


#[derive(Accounts)]
pub struct CloseCommissionTransactionSpl<'info>{
    //////////////////////////////////
    /// TX
    #[account(
        mut,
        constraint =    ((commission_transaction.metadata.transaction_state == TransactionState::BuyerConfirmedProduct) && (commission_transaction.final_decision != BuyerDecisionState::Null)) ||
                        (commission_transaction.metadata.transaction_state == TransactionState::Opened),
    )]
    pub commission_transaction: Box<Account<'info, CommissionTransaction>>,
    
    #[account(
        mut,
        seeds = [
            b"orbit_escrow_account",
            commission_transaction.key().as_ref(),
            buyer_transactions_log.key().as_ref()
        ],
        bump,

        address = commission_transaction.metadata.escrow_account
    )]
    pub escrow_account: Account<'info, TokenAccount>,

    //////////////////////////////////
    /// BUYER SELLER
    
    /// BUYER
    #[account(
        mut,
        constraint = buyer_account.wallet == buyer_transactions_log.buyer_wallet
    )]
    pub buyer_account: Box<Account<'info, OrbitMarketAccount>>,

    #[account(
        mut,
        address = commission_transaction.metadata.buyer
    )]
    pub buyer_transactions_log: Box<Account<'info, BuyerOpenTransactions>>,

    #[account(
        mut,
        token::authority = buyer_transactions_log.buyer_wallet
    )]
    pub buyer_token_account: Account<'info, TokenAccount>,

    /// SELLER
    #[account(
        mut,
        constraint = seller_account.wallet == seller_transactions_log.seller_wallet
    )]
    pub seller_account: Box<Account<'info, OrbitMarketAccount>>,

    #[account(
        mut,
        address = commission_transaction.metadata.seller
    )]
    pub seller_transactions_log: Box<Account<'info, SellerOpenTransactions>>,

    #[account(
        mut,
        token::authority = seller_transactions_log.seller_wallet
    )]
    pub seller_token_account: Account<'info, TokenAccount>,

    
    //////////////////////////////////
    /// CPI AND EXTRANEOUS
    
    #[account(
        seeds = [b"market_authority"],
        bump
    )]
    pub commission_auth: SystemAccount<'info>,
    
    #[account(
        token::authority = Pubkey::new(orbit_addresses::MULTISIG_SIGNER)
    )]
    pub multisig_ata: Account<'info, TokenAccount>,

    pub market_account_program: Program<'info, OrbitMarketAccounts>,
    
    pub commission_program: Program<'info, OrbitCommissionMarket>,

    pub transaction_program: Program<'info, OrbitTransaction>,

    pub token_program: Program<'info, Token>,
    
}

#[derive(Accounts)]
pub struct FundEscrowSpl<'info>{
    ////////////////////////////////////////////
    /// TX
    #[account(
        mut,
        constraint = commission_transaction.metadata.transaction_state == TransactionState::SellerConfirmed,
    )]
    pub commission_transaction: Box<Account<'info, CommissionTransaction>>,
    
    #[account(
        mut,
        seeds = [
            b"orbit_escrow_account",
            commission_transaction.key().as_ref(),
            buyer_transactions_log.key().as_ref()
        ],
        bump,
        address = commission_transaction.metadata.escrow_account
    )]
    pub escrow_account: Account<'info, TokenAccount>,

    ////////////////////////////////////////////
    /// BUYER SELLER
    
    /// BUYER
    #[account(
        mut,
        address = commission_transaction.metadata.buyer
    )]
    pub buyer_transactions_log: Box<Account<'info, BuyerOpenTransactions>>,

    #[account(
        mut,
        token::authority = buyer_wallet.key()
    )]
    pub buyer_token_account: Account<'info, TokenAccount>,

    #[account(
        address = buyer_transactions_log.buyer_wallet
    )]
    pub buyer_wallet: Signer<'info>,

    //////////////////////////////////
    /// CPI AND EXTRANEOUS

    pub token_program: Program<'info, Token>
}

#[derive(Accounts)]
pub struct SellerEarlyDeclineSpl<'info>{
    //////////////////////////////////
    /// TX
    #[account(
        mut,
        constraint = commission_transaction.final_decision == BuyerDecisionState::Null
    )]
    pub commission_transaction: Box<Account<'info, CommissionTransaction>>,
    
    #[account(
        mut,
        seeds = [
            b"orbit_escrow_account",
            commission_transaction.key().as_ref(),
            buyer_transactions_log.key().as_ref()
        ],
        bump,
        
        address = commission_transaction.metadata.escrow_account
    )]
    pub escrow_account: Account<'info, TokenAccount>,

    //////////////////////////////////
    /// BUYER SELLER
    
    /// BUYER
    #[account(
        mut,
        constraint = buyer_account.wallet == buyer_transactions_log.buyer_wallet
    )]
    pub buyer_account: Box<Account<'info, OrbitMarketAccount>>,

    #[account(
        mut,
        address = commission_transaction.metadata.buyer
    )]
    pub buyer_transactions_log: Box<Account<'info, BuyerOpenTransactions>>,

    #[account(
        mut,
        token::authority = buyer_transactions_log.buyer_wallet
    )]
    pub buyer_token_account: Account<'info, TokenAccount>,

    /// SELLER

    #[account(
        mut,
        address = commission_transaction.metadata.seller,
        has_one = seller_wallet
    )]
    pub seller_transactions_log: Box<Account<'info, SellerOpenTransactions>>,

    #[account(
        mut,
        token::authority = seller_wallet
    )]
    pub seller_token_account: Account<'info, TokenAccount>,

    pub seller_wallet: Signer<'info>,

    
    //////////////////////////////////
    /// CPI AND EXTRANEOUS
    
    #[account(
        seeds = [b"market_authority"],
        bump
    )]
    pub commission_auth: SystemAccount<'info>,

    pub market_account_program: Program<'info, OrbitMarketAccounts>,
    
    pub commission_program: Program<'info, OrbitCommissionMarket>,

    pub transaction_program: Program<'info, OrbitTransaction>,

    pub token_program: Program<'info, Token>,
    
}