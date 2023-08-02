pub mod chaum_pedersen;

#[derive(Debug)]
pub enum ZKPError {
    MathOverflow,
    CastError,
}
