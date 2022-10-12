use anchor_lang::prelude::*;
use market_accounts::{
    OrbitMarketAccount,
    program::OrbitMarketAccounts
};
use orbit_product::{ListingsStruct, program::OrbitProduct};
use orbit_product::CommissionProduct;
use crate::{
    CommissionTransaction,
    BuyerDecisionState, program::OrbitCommissionMarket,
};
use orbit_transaction::{transaction_struct::TransactionState, program::OrbitTransaction, BuyerOpenTransactions, SellerOpenTransactions};

#[derive(Accounts)]
#[instruction(seller_tx_index: u8)]
pub struct OpenCommissionTransactionSol<'info>{
    ////////////////////////////////
    /// TX
    #[account(
        init,
        payer = buyer_wallet,
        space = 4000,
        seeds = [
            b"orbit_commission_transaction",
            seller_transactions_log.key().as_ref(),
            [seller_tx_index].as_ref()
        ],
        bump
    )]
    pub commission_transaction: Box<Account<'info, CommissionTransaction>>,
    
    #[account(
        seeds = [
            b"orbit_escrow_account",
            commission_transaction.key().as_ref(),
            buyer_transactions_log.key().as_ref()
        ],
        bump
    )]
    pub escrow_account: SystemAccount<'info>,

    #[account(
        mut,
        constraint = commission_product.metadata.currency == System::id()
    )] 
    pub commission_product: Box<Account<'info, CommissionProduct>>,
    
    //////////////////////////////////////////////////
    /// BUYER SELLER
    
    /// BUYER
    #[account(mut)]
    pub buyer_transactions_log: Box<Account<'info, BuyerOpenTransactions>>,

    #[account(
        mut,
        constraint = buyer_market_account.wallet == buyer_wallet.key(),
        constraint = buyer_market_account.buyer_commission_transactions == buyer_transactions_log.key()
    )]
    pub buyer_market_account: Box<Account<'info, OrbitMarketAccount>>,

    #[account(
        mut,
        address = buyer_transactions_log.buyer_wallet
    )]
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

    /////////////////////////////////
    /// EXTRANEOUS
    
    #[account(
        seeds = [b"market_authority"],
        bump
    )]
    pub commission_auth: SystemAccount<'info>,
    
    pub commission_program: Program<'info, OrbitCommissionMarket>,

    pub transaction_program: Program<'info, OrbitTransaction>,

    pub market_account_program: Program<'info, OrbitMarketAccounts>,
    
    pub product_program: Program<'info, OrbitProduct>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CloseCommissionTransactionSol<'info>{
    ////////////////////////////////////////////
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
            commission_transaction.key().as_ref()
        ],
        bump,
        address = commission_transaction.metadata.escrow_account
    )]
    pub escrow_account: SystemAccount<'info>,

    ///////////////////////////////////////////////////
    /// BUYER SELLER ACCOUNTS
    
    /// BUYER
    #[account(
        mut,
        constraint = buyer_account.buyer_commission_transactions == buyer_transactions_log.key()
    )]
    pub buyer_account: Box<Account<'info, OrbitMarketAccount>>,
    
    #[account(
        mut,
        address = commission_transaction.metadata.buyer,
        has_one = buyer_wallet
    )]
    pub buyer_transactions_log: Box<Account<'info, BuyerOpenTransactions>>,

    #[account(
        mut,
        address = buyer_account.wallet
    )]
    pub buyer_wallet: SystemAccount<'info>,
    
    /// SELLER
    #[account(
        mut,
        constraint = seller_account.seller_commission_transactions == seller_transactions_log.key()
    )]
    pub seller_account: Box<Account<'info, OrbitMarketAccount>>,

    #[account(
        mut,
        address = commission_transaction.metadata.seller,
        has_one = seller_wallet
    )]
    pub seller_transactions_log: Box<Account<'info, SellerOpenTransactions>>,

    #[account(
        mut,
        address = seller_account.wallet
    )]
    pub seller_wallet: SystemAccount<'info>,

    //////////////////////////////////
    /// CPI AND EXTRANEOUS
    
    #[account(
        mut,
        address = Pubkey::new(orbit_addresses::MULTISIG_SIGNER)
    )]
    pub multisig_wallet: SystemAccount<'info>,

    #[account(
        seeds = [b"market_authority"],
        bump
    )]
    pub commission_auth: SystemAccount<'info>,

    pub commission_program: Program<'info, OrbitCommissionMarket>,
    
    pub market_account_program: Program<'info, OrbitMarketAccounts>,

    pub transaction_program: Program<'info, OrbitTransaction>
}

#[derive(Accounts)]
pub struct FundEscrowSol<'info>{
    ////////////////////////////////////////////
    /// TX
    
    #[account(
        mut,
        constraint = commission_transaction.metadata.transaction_state == TransactionState::SellerConfirmed
    )]
    pub commission_transaction: Box<Account<'info, CommissionTransaction>>,
    
    #[account(
        mut,
        seeds = [
            b"orbit_escrow_account",
            commission_transaction.key().as_ref()
        ],
        bump,
        address = commission_transaction.metadata.escrow_account
    )]
    pub escrow_account: SystemAccount<'info>,

    ////////////////////////////////////////////
    /// BUYER SELLER

    /// BUYER
    #[account(
        mut,
        address = commission_transaction.metadata.buyer,
        has_one = buyer_wallet
    )]
    pub buyer_transactions_log: Box<Account<'info, BuyerOpenTransactions>>,

    #[account(mut)]
    pub buyer_wallet: Signer<'info>
}

#[derive(Accounts)]
pub struct SellerEarlyDeclineSol<'info>{
    ////////////////////////////////////////////
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
            commission_transaction.key().as_ref()
        ],
        bump,
        address = commission_transaction.metadata.escrow_account
    )]
    pub escrow_account: SystemAccount<'info>,

    ///////////////////////////////////////////////////
    /// BUYER SELLER ACCOUNTS
    
    /// BUYER
    #[account(
        mut,
        constraint = buyer_account.buyer_commission_transactions == buyer_transactions_log.key()
    )]
    pub buyer_account: Box<Account<'info, OrbitMarketAccount>>,
    
    #[account(
        mut,
        address = commission_transaction.metadata.buyer,
        has_one = buyer_wallet
    )]
    pub buyer_transactions_log: Box<Account<'info, BuyerOpenTransactions>>,

    #[account(
        mut,
        address = buyer_account.wallet
    )]
    pub buyer_wallet: SystemAccount<'info>,
    
    /// SELLER

    #[account(
        mut,
        address = commission_transaction.metadata.seller,
        has_one = seller_wallet
    )]
    pub seller_transactions_log: Box<Account<'info, SellerOpenTransactions>>,

    #[account(mut)]
    pub seller_wallet: Signer<'info>,

    //////////////////////////////////
    /// CPI AND EXTRANEOUS

    #[account(
        seeds = [b"market_authority"],
        bump
    )]
    pub commission_auth: SystemAccount<'info>,

    pub commission_program: Program<'info, OrbitCommissionMarket>,
    
    pub market_account_program: Program<'info, OrbitMarketAccounts>,

    pub transaction_program: Program<'info, OrbitTransaction>
}