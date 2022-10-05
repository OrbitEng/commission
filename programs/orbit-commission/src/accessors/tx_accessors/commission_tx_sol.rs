use anchor_lang::prelude::*;
use market_accounts::{
    OrbitMarketAccount,
    program::OrbitMarketAccounts
};
use orbit_catalog::OrbitVendorCatalog;
use orbit_multisig::Multisig;
use crate::{
    CommissionTransaction,
    CommissionProduct,
    BuyerDecisionState, program::OrbitCommissionMarket,
};
use transaction::transaction_struct::TransactionState;

#[derive(Accounts)]
pub struct OpenCommissionTransactionSol<'info>{
    #[account(
        init,
        space = 4000,
        payer = buyer_wallet,
    )]
    pub commission_transaction: Box<Account<'info, CommissionTransaction>>,

    #[account(
        constraint = commission_product.metadata.currency == System::id()
    )]
    pub commission_product: Box<Account<'info, CommissionProduct>>,

    #[account(
        constraint = seller_account.wallet == seller_catalog.catalog_owner
    )]
    pub seller_account:Box<Account<'info, OrbitMarketAccount>>,

    #[account(
        address = commission_product.metadata.owner_catalog
    )]
    pub seller_catalog:Box<Account<'info, OrbitVendorCatalog>>,

    #[account(
        seeds = [
            b"orbit_escrow_account",
            commission_transaction.key().as_ref()
        ],
        bump
    )]
    pub escrow_account: SystemAccount<'info>,

    #[account(
        mut,
        seeds = [
            b"orbit_account",
            buyer_wallet.key().as_ref()
        ],
        bump,
        seeds::program = market_accounts::ID
    )]
    pub buyer_account:Box<Account<'info, OrbitMarketAccount>>,

    #[account(
        mut,
        address = buyer_account.wallet
    )]
    pub buyer_wallet: Signer<'info>,

    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
pub struct CloseCommissionTransactionSol<'info>{
    #[account(
        mut,
        constraint =    ((commission_transaction.metadata.transaction_state == TransactionState::BuyerConfirmedProduct) && (commission_transaction.final_decision != BuyerDecisionState::Null)) ||
                        (commission_transaction.metadata.transaction_state == TransactionState::Opened),
    )]
    pub commission_transaction: Box<Account<'info, CommissionTransaction>>,

    #[account(
        constraint = buyer_account.voter_id == commission_transaction.metadata.buyer
    )]
    pub buyer_account:Box<Account<'info, OrbitMarketAccount>>,

    #[account(
        mut,
        address = buyer_account.wallet
    )]
    pub buyer_wallet: SystemAccount<'info>,

    #[account(
        constraint = seller_account.voter_id == commission_transaction.metadata.seller
    )]
    pub seller_account:Box<Account<'info, OrbitMarketAccount>>,

    #[account(
        mut,
        address = seller_account.wallet
    )]
    pub seller_wallet: SystemAccount<'info>,

    #[account(
        seeds = [
            b"orbit_escrow_account",
            commission_transaction.key().as_ref()
        ],
        bump,

        address = commission_transaction.metadata.escrow_account
    )]
    pub escrow_account: SystemAccount<'info>,

    #[account(
        seeds = [b"market_authority"],
        bump
    )]
    pub commission_auth: SystemAccount<'info>,

    #[account(
        address = market_accounts::ID
    )]
    pub market_account_program: Program<'info, OrbitMarketAccounts>,

    pub commission_program: Program<'info, OrbitCommissionMarket>,

    #[account(
        mut,
        address = Pubkey::new(orbit_addresses::MULTISIG_WALLET_ADDRESS)
    )]
    pub multisig_address: Box<Account<'info, Multisig>>,
    
    #[account(
        mut,
        seeds = [
            multisig_address.key().as_ref()
        ],
        bump = multisig_address.nonce
    )]
    pub multisig_wallet: SystemAccount<'info>,
}

#[derive(Accounts)]
pub struct FundEscrowSol<'info>{
    #[account(
        mut,
        constraint = commission_transaction.metadata.transaction_state == TransactionState::SellerConfirmed,
    )]
    pub commission_transaction: Box<Account<'info, CommissionTransaction>>,

    #[account(
        constraint = buyer_account.voter_id == commission_transaction.metadata.buyer,
        seeds = [
            b"orbit_account",
            buyer_wallet.key().as_ref()
        ],
        bump,
        seeds::program = market_accounts::ID
    )]
    pub buyer_account:Box<Account<'info, OrbitMarketAccount>>,

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

    #[account(
        mut,
        address = buyer_account.wallet
    )]
    pub buyer_wallet: Signer<'info>,
}
