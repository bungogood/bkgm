use base64::engine::general_purpose;
use base64::Engine;

use crate::codecs::VariantCodec;
use crate::position::{Position, O_BAR, X_BAR};
use crate::{State, Variant, VariantPosition};

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum GnuidError {
    #[error("invalid GNUID payload")]
    InvalidPayload,
    #[error("invalid GNUID field '{0}'")]
    InvalidField(&'static str),
}

pub struct GnuidCodec;

impl VariantCodec for GnuidCodec {
    type Error = GnuidError;

    fn encode(position: VariantPosition) -> String {
        encode(position)
    }

    fn decode(variant: Variant, input: &str) -> Result<VariantPosition, Self::Error> {
        decode(variant, input)
    }
}

pub fn encode(position: VariantPosition) -> String {
    match position {
        VariantPosition::Backgammon(p) => encode_for_position(&p),
        VariantPosition::Nackgammon(p) => encode_for_position(&p),
        VariantPosition::Longgammon(p) => encode_for_position(&p),
        VariantPosition::Hypergammon(p) => encode_for_position(&p),
        VariantPosition::Hypergammon2(p) => encode_for_position(&p),
        VariantPosition::Hypergammon4(p) => encode_for_position(&p),
        VariantPosition::Hypergammon5(p) => encode_for_position(&p),
    }
}

pub fn decode(variant: Variant, id: &str) -> Result<VariantPosition, GnuidError> {
    match variant {
        Variant::Backgammon => decode_for_position::<15>(id).map(VariantPosition::Backgammon),
        Variant::Nackgammon => decode_for_position::<15>(id).map(VariantPosition::Nackgammon),
        Variant::Longgammon => decode_for_position::<15>(id).map(VariantPosition::Longgammon),
        Variant::Hypergammon => decode_for_position::<3>(id).map(VariantPosition::Hypergammon),
        Variant::Hypergammon2 => decode_for_position::<2>(id).map(VariantPosition::Hypergammon2),
        Variant::Hypergammon4 => decode_for_position::<4>(id).map(VariantPosition::Hypergammon4),
        Variant::Hypergammon5 => decode_for_position::<5>(id).map(VariantPosition::Hypergammon5),
    }
}

fn encode_for_position<const N: u8>(position: &Position<N>) -> String {
    let key = encode_key(position);
    let b64 = general_purpose::STANDARD.encode(key);
    b64[..14].to_string()
}

pub fn encode_position<const N: u8>(position: &Position<N>) -> String {
    encode_for_position(position)
}

fn decode_for_position<const N: u8>(id: &str) -> Result<Position<N>, GnuidError> {
    let padded_id = format!("{}==", id);
    let key = general_purpose::STANDARD
        .decode(padded_id)
        .map_err(|_| GnuidError::InvalidPayload)?;
    let key: [u8; 10] = key
        .try_into()
        .map_err(|_| GnuidError::InvalidField("key_length"))?;
    Ok(decode_key(key))
}

pub fn decode_position<const N: u8>(id: &str) -> Result<Position<N>, GnuidError> {
    decode_for_position(id)
}

fn encode_key<const N: u8>(position: &Position<N>) -> [u8; 10] {
    let mut key = [0u8; 10];
    let mut bit_index = 0;

    for point in (O_BAR + 1..X_BAR).rev() {
        for _ in 0..-position.pip(point) {
            key[bit_index / 8] |= 1 << (bit_index % 8);
            bit_index += 1;
        }
        bit_index += 1;
    }
    for _ in 0..position.o_bar() {
        key[bit_index / 8] |= 1 << (bit_index % 8);
        bit_index += 1;
    }
    bit_index += 1;

    for point in O_BAR + 1..X_BAR {
        for _ in 0..position.pip(point) {
            key[bit_index / 8] |= 1 << (bit_index % 8);
            bit_index += 1;
        }
        bit_index += 1;
    }
    for _ in 0..position.x_bar() {
        key[bit_index / 8] |= 1 << (bit_index % 8);
        bit_index += 1;
    }

    key
}

fn decode_key<const N: u8>(key: [u8; 10]) -> Position<N> {
    let mut bit_index = 0;
    let mut pips = [0i8; 26];

    let mut x_pieces = 0;
    let mut o_pieces = 0;

    for point in (O_BAR + 1..X_BAR).rev() {
        while (key[bit_index / 8] >> (bit_index % 8)) & 1 == 1 {
            pips[point] -= 1;
            o_pieces += 1;
            bit_index += 1;
        }
        bit_index += 1;
    }
    while (key[bit_index / 8] >> (bit_index % 8)) & 1 == 1 {
        pips[O_BAR] -= 1;
        o_pieces += 1;
        bit_index += 1;
    }
    bit_index += 1;

    for pip in pips.iter_mut().take(X_BAR).skip(O_BAR + 1) {
        while (key[bit_index / 8] >> (bit_index % 8)) & 1 == 1 {
            *pip += 1;
            x_pieces += 1;
            bit_index += 1;
        }
        bit_index += 1;
    }
    while (key[bit_index / 8] >> (bit_index % 8)) & 1 == 1 {
        pips[X_BAR] += 1;
        x_pieces += 1;
        bit_index += 1;
    }

    Position {
        turn: true,
        pips,
        x_off: N - x_pieces,
        o_off: N - o_pieces,
    }
}

#[cfg(test)]
mod tests {
    use super::{decode, encode, GnuidCodec};
    use crate::codecs::assert_roundtrip;
    use crate::position::Position;
    use crate::Variant;

    #[test]
    fn gnuid_string_roundtrip() {
        let ids = [
            "4HPwATDgc/ABMA",
            "jGfkASjg8wcBMA",
            "zGbiIQgxH/AAWA",
            "zGbiIYCYD3gALA",
        ];
        for id in ids {
            let pos = decode(Variant::Backgammon, id).expect("must decode gnuid");
            let encoded = encode(pos);
            assert_eq!(encoded, id);
        }
    }

    #[test]
    fn gnuid_roundtrip_with_bars_and_off() {
        let mut pips = [0i8; 26];
        pips[25] = 2;
        pips[0] = -1;
        pips[8] = 3;
        pips[6] = 2;
        pips[19] = -2;
        pips[17] = -1;
        let position = Position::<15>::try_from(pips).expect("valid custom board");
        let variant_position = crate::VariantPosition::Backgammon(position);

        let id = encode(variant_position);
        let decoded = decode(Variant::Backgammon, &id).expect("must decode gnuid");
        assert_eq!(encode(decoded), id);
        assert_roundtrip::<GnuidCodec>(Variant::Backgammon, variant_position);
    }
}
