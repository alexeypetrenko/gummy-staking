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

pub const DISCRIMINATOR_DEPOSIT: &[u8] = b"\x3e\xcd\xf2\xaf\xf4\xa9\x88\x34";
// echo -n 'event:Deposit' | shasum  -a256 | cut -c -16 | sed -E 's/(..)/\\x\1/g'
#[derive(BorshDeserialize, Debug)]
pub struct Deposit {
    pub user: Pubkey,
    pub amount: u64,
    pub total_amount: u64,
    pub lock_expires: u32,
    pub referrer: Pubkey,
}

pub const DISCRIMINATOR_WITHDRAW: &[u8] = b"\xc0\xf1\xc9\xd9\x46\x96\x5a\xf7";
#[derive(BorshDeserialize, Debug)]
pub struct Withdraw {
    pub user: Pubkey,
    pub total_amount: u64,
}

pub const DISCRIMINATOR_SET_REFERRER: &[u8] = b"\xbe\xfb\x76\x7f\x49\x7b\x52\xb8";
#[derive(BorshDeserialize, Debug)]
pub struct SetReferrer {
    pub user: Pubkey,
    pub old_referrer: Pubkey,
    pub new_referrer: Pubkey,
}

pub const DISCRIMINATOR_REGISTER_SHORT_REFERRER: &[u8] = b"\x28\x71\xfb\x60\x2c\x5b\xf4\xc1";
#[derive(BorshDeserialize, Debug)]
pub struct RegisterShortReferrer {
    pub full: Pubkey,
    pub short: Vec<u8>,
}

pub const DISCRIMINATOR_ADMIN_REGISTER_SHORT_REFERRER: &[u8] = b"\x69\x8b\x57\xae\x7d\xc8\x06\x6f";
#[derive(BorshDeserialize, Debug)]
pub struct AdminRegisterShortReferrer {
    pub full: Pubkey,
    pub short: Vec<u8>,
    pub initiator: Pubkey,
}

pub const DISCRIMINATOR_ADMIN_DELETE_SHORT_REFERRER: &[u8] = b"\x1c\x12\x1a\x34\x94\xca\xff\x40";
#[derive(BorshDeserialize, Debug)]
pub struct AdminDeleteShortReferrer {
    pub short: Vec<u8>,
    pub initiator: Pubkey,
}

pub const DISCRIMINATOR_ADMIN_EMERGENCY_WITHDRAW: &[u8] = b"\x65\xa3\xa1\x8b\xa3\x9d\x9d\xe3";
#[derive(BorshDeserialize, Debug)]
pub struct AdminEmergencyWithdraw {
    pub user: Pubkey,
    pub total_amount: u64,
    pub initiator: Pubkey,
}