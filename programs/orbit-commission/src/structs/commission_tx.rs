
#[account]
pub struct ComishAccount{
    pub preview_address: [u8; 64],
    pub preview_rate: u8,
    pub last_rate_offerer: Pubkey,

    pub funder: Pubkey,
}