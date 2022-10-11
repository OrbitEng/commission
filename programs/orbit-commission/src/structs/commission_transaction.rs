use anchor_lang::prelude::*;
use orbit_transaction::transaction_struct::OrbitTransactionStruct;

#[account]
pub struct CommissionTransaction{
    pub metadata: OrbitTransactionStruct,

    pub preview_address: String,
    pub preview_rate: u8,
    pub last_rate_offerer: Pubkey,

    pub close_rate: u8,

    pub data_address: String, // 64
    pub num_keys: u64, // 8
    pub key_arr: Vec<Pubkey>, // up to 2048
    pub final_decision: BuyerDecisionState, // 1
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq)]
pub enum BuyerDecisionState{
    Null,
    Declined,
    Accept
}