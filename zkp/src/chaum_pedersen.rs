use crate::ZKPError;

pub struct ChaumPedersenProtocol {
    ctx: Context,
}

pub struct Context {
    g: i64,
    h: i64,
    q: i64,
}

impl Context {
    pub fn new(g: i64, h: i64, q: i64) -> Self {
        Self { g, h, q }
    }
}

impl ChaumPedersenProtocol {
    pub fn new(ctx: Context) -> Self {
        Self { ctx }
    }

    pub fn calculate_challenge(&self, k: i64, c: i64, x: i64) -> Result<i64, ZKPError> {
        Ok(
            (ChaumPedersenProtocol::safe_sub(k, ChaumPedersenProtocol::safe_mul(c, x)?)?)
                % self.ctx.q,
        )
    }

    fn safe_sub(l: i64, value: i64) -> Result<i64, ZKPError> {
        l.checked_sub(value)
            .map_or_else(|| Err(ZKPError::MathOverflow), Ok)
    }
    fn safe_pow(base: i64, exp: i64) -> Result<i64, ZKPError> {
        base.checked_pow(exp.try_into().map_err(|_| ZKPError::CastError)?)
            .map_or_else(|| Err(ZKPError::MathOverflow), Ok)
    }

    fn safe_mul(l: i64, r: i64) -> Result<i64, ZKPError> {
        l.checked_mul(r)
            .map_or_else(|| Err(ZKPError::MathOverflow), Ok)
    }
    pub fn calculate_registration_data(&self, x: i64) -> Result<(i64, i64), ZKPError> {
        let y1 = ChaumPedersenProtocol::safe_pow(self.ctx.g, x)?;
        let y2 = ChaumPedersenProtocol::safe_pow(self.ctx.h, x)?;

        Ok((y1, y2))
    }

    pub fn calculate_login_challenge_data(&self, k: i64) -> Result<(i64, i64), ZKPError> {
        self.calculate_registration_data(k)
    }

    pub fn create_auth_challenge(&self) -> i64 {
        2
        // rand::random::<i64>()
    }

    pub fn verify_auth_challenge(
        &self,
        y1: i64,
        y2: i64,
        r1: i64,
        r2: i64,
        c: i64,
        s: i64,
    ) -> Result<bool, ZKPError> {
        let test_left_r1 = ChaumPedersenProtocol::safe_pow(self.ctx.g, s)?;
        let test_right_r1 = ChaumPedersenProtocol::safe_pow(y1, c)?;
        let test_r1 = ChaumPedersenProtocol::safe_mul(test_left_r1, test_right_r1)?;

        let test_left_r2 = ChaumPedersenProtocol::safe_pow(self.ctx.h, s)?;
        let test_right_r2 = ChaumPedersenProtocol::safe_pow(y2, c)?;
        let test_r2 = ChaumPedersenProtocol::safe_mul(test_left_r2, test_right_r2)?;

        Ok((test_r1 == r1) && (test_r2 == r2))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ZKP_G: i64 = 3;
    const ZKP_H: i64 = 5;
    const ZKP_Q: i64 = 10009;
    const CHALLENGE_K: i64 = 27;

    fn protocol() -> ChaumPedersenProtocol {
        ChaumPedersenProtocol::new(Context::new(ZKP_G, ZKP_H, ZKP_Q))
    }

    fn init() -> (i64, ChaumPedersenProtocol) {
        (9, protocol())
    }

    #[test]
    fn should_calculate_proper_registration_data() {
        let (secret, zkp) = init();
        let (y1, y2) = zkp.calculate_registration_data(secret).unwrap();
        assert_eq!(y1, ZKP_G.checked_pow(secret.try_into().unwrap()).unwrap());
        assert_eq!(y2, ZKP_H.checked_pow(secret.try_into().unwrap()).unwrap());
    }

    #[test]
    fn should_calculate_proper_login_challenge_data() {
        let (_secret, zkp) = init();
        let (r1, r2) = zkp.calculate_login_challenge_data(CHALLENGE_K).unwrap();
        assert_eq!(
            r1,
            ZKP_G.checked_pow(CHALLENGE_K.try_into().unwrap()).unwrap()
        );
        assert_eq!(
            r2,
            ZKP_H.checked_pow(CHALLENGE_K.try_into().unwrap()).unwrap()
        );
    }

    #[test]
    fn should_calculate_proper_auth_challenge() {
        let (secret, zkp) = init();
        let challenge_c = zkp.create_auth_challenge();
        let challenge_s = zkp
            .calculate_challenge(CHALLENGE_K, challenge_c, secret)
            .unwrap();
        assert_eq!(
            challenge_s,
            CHALLENGE_K
                .checked_sub(challenge_c.checked_mul(secret).unwrap())
                .unwrap()
                % ZKP_Q
        );
    }

    #[test]
    fn should_accept_valid_challenge() {
        let (secret, zkp) = init();
        let (y1, y2) = zkp.calculate_registration_data(secret).unwrap();
        let (r1, r2) = zkp.calculate_login_challenge_data(CHALLENGE_K).unwrap();
        let challenge_c = zkp.create_auth_challenge();
        let challenge_s = zkp
            .calculate_challenge(CHALLENGE_K, challenge_c, secret)
            .unwrap();
        assert!(zkp
            .verify_auth_challenge(y1, y2, r1, r2, challenge_c, challenge_s)
            .unwrap());
    }

    #[test]
    fn should_reject_invalid_challenge() {
        let (secret, zkp) = init();
        let (y1, y2) = zkp.calculate_registration_data(secret).unwrap();
        let (r1, r2) = zkp.calculate_login_challenge_data(CHALLENGE_K).unwrap();
        let challenge_c = zkp.create_auth_challenge();
        let invalid_challenge_s = zkp
            .calculate_challenge(CHALLENGE_K, challenge_c, secret.checked_add(1).unwrap())
            .unwrap();
        assert!(!zkp
            .verify_auth_challenge(y1, y2, r1, r2, challenge_c, invalid_challenge_s)
            .unwrap());
    }
}
