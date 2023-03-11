use anchor_lang::prelude::*;

#[error_code]
pub enum CommissionMarketErrors{
    #[msg("Wrong account to pay")]
    WrongDecisionAccount,
    #[msg("Decision has not been made yet, can not close")]
    UndecidedTransaction,
    #[msg("Can not accept your own rate")]
    InvalidRateAcceptor,
    #[msg("Please confirm delivery first")]
    DidNotConfirmDelivery,
    #[msg("The seller did not commit keys yet")]
    WaitingForSellerData,
    #[msg("The seller for the product does not match the seller given")]
    InvalidSellerForListing,
    #[msg("Could not compute escrow bump")]
    InvalidEscrowBump,
    #[msg("Could not compute auth bump")]
    InvalidAuthBump,
    #[msg("Could not decode private key")]
    CorruptPrivateKeyFormat,
    #[msg("Private and Public keys do not match")]
    IncorrectPrivateKey,
    #[msg("Private and Public keys do not match")]
    IndexOutOfRange,
    #[msg("Can not use discounts on commissions")]
    CannotDiscountCommission,
    #[msg("invalid reflink passed")]
    InvalidReflink,
}