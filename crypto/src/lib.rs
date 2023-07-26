pub trait ZKPAuth {
    fn create_auth_challenge(&self) -> i64;
    fn verify_auth_challenge(&self, y1: i64, y2: i64, r1: i64, r2: i64, c: i64, s: i64) -> bool;
}

pub mod chaum_pedersen;
