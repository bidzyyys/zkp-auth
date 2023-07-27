use zkp::chaum_pedersen::ChaumPedersenProtocol;

pub fn calculate_challenge(c: i64) -> i64 {
    ChaumPedersenProtocol::calculate_challenge(c)
}
