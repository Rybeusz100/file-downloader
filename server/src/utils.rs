use hmac::{Hmac, Mac};
use rand::Rng;
use sha2::Sha256;

pub fn generate_jwt_secret() -> Hmac<Sha256> {
    let mut rng = rand::thread_rng();
    let mut key = [0u8; 64];
    rng.fill(&mut key);
    type HmacSha256 = Hmac<Sha256>;
    HmacSha256::new_from_slice(&key).unwrap()
}
