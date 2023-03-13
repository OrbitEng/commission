use anchor_lang::prelude::*;
use orbit_transaction::transaction_struct::OrbitTransactionStruct;

#[account]
pub struct CommissionTransaction{
    pub metadata: OrbitTransactionStruct, // 120

    pub preview_address: String, // 43
    pub preview_rate: u8, // 1
    pub last_rate_offerer: u64, // 8

    pub close_rate: u8, // 1

    pub data_address: String, // 43
    pub num_keys: u64, // 8
    pub key_arr: Vec<Pubkey>, // up to 2048 ; 64 keys
    pub final_decision: BuyerDecisionState, // 1
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq)]
pub enum BuyerDecisionState{
    Null,
    Declined,
    Accept
}