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
    // #TODO
    pub fn calculate_challenge(&self, k: i64, c: i64, x: i64) -> Result<i64, ZKPError> {
        Ok((k - ChaumPedersenProtocol::safe_mul(c, x)?).wrapping_rem(self.ctx.q))
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

    pub fn create_auth_challenge(&self) -> i64 {
        19
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

    fn protocol() -> ChaumPedersenProtocol {
        ChaumPedersenProtocol::new(Context::new(3, 2892, 5980))
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
