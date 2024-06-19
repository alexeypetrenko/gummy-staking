use borsh::BorshDeserialize;

#[derive(BorshDeserialize, Debug)]
pub struct Pubkey([u8; 32]);
impl AsRef<[u8]> for Pubkey {
    fn as_ref(&self) -> &[u8] {
        &self.0[..]
    }
}
impl Pubkey {
    pub fn to_string(&self) -> String {
        bs58::encode(self.0).into_string()
    }
}

#[derive(BorshDeserialize, Debug)]
pub struct Deposit {
    pub user: Pubkey,
    pub amount: u64,
    pub total_amount: u64,
    pub lock_expires: u32,
    pub referrer: Pubkey,
}