pub mod fibs;
pub mod gnuid;
pub mod move_text;
pub mod xgid;

use crate::{Variant, VariantPosition};

pub trait VariantCodec {
    type Error;

    fn encode(position: VariantPosition) -> String;
    fn decode(variant: Variant, input: &str) -> Result<VariantPosition, Self::Error>;
}

#[cfg(test)]
pub(crate) fn assert_roundtrip<C: VariantCodec>(variant: Variant, position: VariantPosition)
where
    C::Error: core::fmt::Display,
{
    let encoded = C::encode(position);
    let decoded = C::decode(variant, &encoded).unwrap_or_else(|err| {
        panic!("codec roundtrip decode failed for {variant}: {err}");
    });
    assert_eq!(decoded, position);
}
