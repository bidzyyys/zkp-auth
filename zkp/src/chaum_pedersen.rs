use crate::ZKPAuth;
use num_traits::CheckedMul;

pub struct ChaumPedersenProtocol {
    g: i64,
    h: i64,
    q: i64,
}

impl ZKPAuth for ChaumPedersenProtocol {
    fn create_auth_challenge(&self) -> i64 {
        rand::random::<i64>()
    }

    fn verify_auth_challenge(&self, y1: i64, y2: i64, r1: i64, r2: i64, c: i64, s: i64) -> bool {
        let test_left_r1 = self
            .g
            .checked_pow(s.try_into().expect("negative s"))
            .expect("left r1");
        let test_right_r1 = y1
            .checked_pow(c.try_into().expect("negative c"))
            .expect("right r1");
        let test_r1 = test_left_r1.checked_mul(test_right_r1).expect("r1");

        let test_left_r2 = self
            .h
            .checked_pow(s.try_into().expect("negative s"))
            .expect("left r2");
        let test_right_r2 = y2
            .checked_pow(c.try_into().expect("negative c"))
            .expect("right r2");
        let test_r2 = test_left_r2.checked_mul(test_right_r2).expect("r2");

        (test_r1 == r1) && (test_r2 == r2)
    }
}

impl ChaumPedersenProtocol {
    pub fn new(g: i64, h: i64, q: i64) -> Self {
        Self { g, h, q }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn protocol() -> ChaumPedersenProtocol {
        ChaumPedersenProtocol::new(3, 2892, 10009)
    }

    fn init() -> (i64, ChaumPedersenProtocol) {
        (300, protocol())
    }

    #[test]
    fn should_create_proper_challenge() {
        let (_secret, zkp) = init();
        let _c = zkp.create_auth_challenge();
    }

    #[test]
    fn should_accept_proper_challenge() {
        let (_secret, _zkp) = init();
    }

    #[test]
    fn should_reject_invalid_challenge() {
        let (_secret, _zkp) = init();
    }
}
